//! Typed extractors for Resuma Flow loaders, submits, and server actions.
//!
//! ```ignore
//! #[load]
//! async fn user(id: Path<u64>, q: Query<SearchParams>) -> UserData { ... }
//!
//! #[submit]
//! async fn save(form: CreateUser, q: Query<RedirectTo>) -> Result<(), SubmitError> { ... }
//! ```

use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::core::{FlowRequest, Result, ResumaError};

/// Extract route params (`:id`, `:slug`, …) into a typed value.
///
/// For a single param route, `Path<u64>` reads the lone value. For multiple
/// params, use a struct with `#[derive(Deserialize)]` and field names matching
/// the route pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path<T>(pub T);

/// Extract query-string parameters into a typed value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query<T>(pub T);

/// Deserialize route params from a request.
pub trait FromFlowRequest: Sized {
    fn from_request(req: &FlowRequest) -> Result<Self>;
}

impl<T> FromFlowRequest for Path<T>
where
    T: DeserializeOwned,
{
    fn from_request(req: &FlowRequest) -> Result<Self> {
        if req.params.is_empty() {
            return Err(ResumaError::Validation(
                "Path extractor: no route params on this request".into(),
            ));
        }

        if req.params.len() == 1 {
            let raw = req.params.values().next().expect("checked len");
            if let Ok(parsed) = parse_scalar::<T>(raw) {
                return Ok(Path(parsed));
            }
        }

        let map = string_map_to_json(&req.params);
        serde_json::from_value(Value::Object(map))
            .map(Path)
            .map_err(|e| {
                ResumaError::Validation(format!(
                    "Path extractor: could not decode params into `{}`: {e}",
                    std::any::type_name::<T>()
                ))
            })
    }
}

impl<T> FromFlowRequest for Query<T>
where
    T: DeserializeOwned,
{
    fn from_request(req: &FlowRequest) -> Result<Self> {
        if req.query.is_empty() {
            return Err(ResumaError::Validation(
                "Query extractor: no query params on this request".into(),
            ));
        }

        if req.query.len() == 1 {
            let raw = req.query.values().next().expect("checked len");
            if let Ok(parsed) = parse_scalar::<T>(raw) {
                return Ok(Query(parsed));
            }
        }

        let map = string_map_to_json(&req.query);
        serde_json::from_value(Value::Object(map))
            .map(Query)
            .map_err(|e| {
                ResumaError::Validation(format!(
                    "Query extractor: could not decode query into `{}`: {e}",
                    std::any::type_name::<T>()
                ))
            })
    }
}

impl FromFlowRequest for FlowRequest {
    fn from_request(req: &FlowRequest) -> Result<Self> {
        Ok(req.clone())
    }
}

fn string_map_to_json(map: &std::collections::BTreeMap<String, String>) -> Map<String, Value> {
    map.iter()
        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
        .collect()
}

fn parse_scalar<T: DeserializeOwned>(raw: &str) -> Result<T> {
    if let Ok(v) = serde_json::from_str::<T>(raw) {
        return Ok(v);
    }
    serde_json::from_value(Value::String(raw.to_string())).map_err(|e| {
        ResumaError::Validation(format!(
            "Could not parse `{raw}` into `{}`: {e}",
            std::any::type_name::<T>()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn path_single_param_u64() {
        let mut params = BTreeMap::new();
        params.insert("id".into(), "42".into());
        let req =
            FlowRequest::from_parts("GET", "/users/42", BTreeMap::new(), params, BTreeMap::new());
        let Path(id): Path<u64> = Path::from_request(&req).unwrap();
        assert_eq!(id, 42);
    }

    #[test]
    fn query_single_param() {
        let mut query = BTreeMap::new();
        query.insert("q".into(), "hello".into());
        let req = FlowRequest::from_parts("GET", "/", BTreeMap::new(), BTreeMap::new(), query);
        let Query(q): Query<String> = Query::from_request(&req).unwrap();
        assert_eq!(q, "hello");
    }
}
