//! HTTP request helpers for Resuma Flow.

pub use crate::core::FlowRequest;

use std::collections::BTreeMap;

/// Parse query strings into a sorted map for [`FlowRequest::query`].
pub fn parse_query(query: Option<&str>) -> BTreeMap<String, String> {
    query
        .and_then(|q| serde_urlencoded::from_str::<Vec<(String, String)>>(q).ok())
        .map(|pairs| pairs.into_iter().collect())
        .unwrap_or_default()
}

/// Build a [`FlowRequest`] from an axum HTTP request and route params.
pub fn from_http_request(
    req: &axum::http::Request<axum::body::Body>,
    path: &str,
    params: BTreeMap<String, String>,
) -> FlowRequest {
    from_http(
        req.method().as_str(),
        path,
        req.headers(),
        params,
        parse_query(req.uri().query()),
    )
}

/// Build a [`FlowRequest`] from plain HTTP parts.
pub fn from_http(
    method: &str,
    path: &str,
    headers: &axum::http::HeaderMap,
    params: BTreeMap<String, String>,
    query: BTreeMap<String, String>,
) -> FlowRequest {
    FlowRequest::from_parts(
        method,
        path,
        headers
            .iter()
            .filter_map(|(k, v)| {
                v.to_str()
                    .ok()
                    .map(|s| (k.as_str().to_string(), s.to_string()))
            })
            .collect(),
        params,
        query,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_query_decodes_pairs() {
        let q = parse_query(Some("a=1&b=two"));
        assert_eq!(q.get("a").map(String::as_str), Some("1"));
        assert_eq!(q.get("b").map(String::as_str), Some("two"));
    }
}
