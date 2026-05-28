//! Redirect helpers for `#[submit]` and `#[server]` handlers (PRG / post-action navigation).

use crate::core::{FlowRequest, Result, ResumaError};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Redirect as AxumRedirect, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Query-string key used by [`redirect_with_flash`] and [`flash_message`].
pub const FLASH_KEY: &str = "flash";

/// Redirect target returned from a submit or server action.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Redirect {
    pub to: String,
}

impl Redirect {
    pub fn to(path: impl Into<String>) -> Self {
        Self { to: path.into() }
    }

    pub fn into_response(self) -> axum::response::Response {
        redirect_response(&self.to)
    }
}

impl Serialize for Redirect {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Redirect", 1)?;
        s.serialize_field("redirect", &self.to)?;
        s.end()
    }
}

/// Build a redirect value for `#[submit]` / `#[server]` return types.
pub fn redirect(path: impl Into<String>) -> Redirect {
    Redirect::to(path)
}

/// Build a PRG redirect that carries a one-shot flash message as a query param.
///
/// Stateless: the message survives a 303 redirect (no-JS) and SPA navigation
/// alike, and is read on the target page with [`flash_message`]. No session or
/// cookie storage required.
///
/// ```ignore
/// #[submit]
/// async fn create(req: &FlowRequest) -> Redirect {
///     // ...persist...
///     redirect_with_flash("/items", "Item created")
/// }
/// // On the target page:
/// if let Some(msg) = flash_message(req) { /* render banner */ }
/// ```
pub fn redirect_with_flash(path: impl Into<String>, message: impl AsRef<str>) -> Redirect {
    Redirect::to(append_flash(&path.into(), message.as_ref()))
}

/// Read a one-shot flash message set by [`redirect_with_flash`] from the request query.
pub fn flash_message(req: &FlowRequest) -> Option<String> {
    req.query_param(FLASH_KEY)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn append_flash(path: &str, message: &str) -> String {
    let encoded = serde_urlencoded::to_string([(FLASH_KEY, message)])
        .unwrap_or_else(|_| format!("{FLASH_KEY}="));
    let sep = if path.contains('?') { '&' } else { '?' };
    format!("{path}{sep}{encoded}")
}

/// Extract a same-origin redirect path from a serialized handler result.
pub fn extract_redirect(value: &Value) -> Option<String> {
    let path = value.get("redirect")?.as_str()?;
    validate_redirect_path(path).ok().map(str::to_string)
}

/// Reject open redirects — only root-relative paths are allowed.
pub fn validate_redirect_path(path: &str) -> Result<&str> {
    if !path.starts_with('/') || path.starts_with("//") {
        return Err(ResumaError::Other(format!(
            "invalid redirect path `{path}` (must start with `/`, not `//`)"
        )));
    }
    Ok(path)
}

/// HTTP 303 See Other — standard PRG response for form submits without JavaScript.
pub fn redirect_response(path: &str) -> Response {
    match validate_redirect_path(path) {
        Ok(loc) => AxumRedirect::to(loc).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

/// Attach `Location` on JSON responses when a redirect hint is present.
pub fn redirect_json_headers(path: &str) -> Option<[(header::HeaderName, String); 1]> {
    validate_redirect_path(path)
        .ok()
        .map(|loc| [(header::LOCATION, loc.to_string())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_root_relative_paths() {
        assert_eq!(validate_redirect_path("/items").unwrap(), "/items");
        assert_eq!(
            validate_redirect_path("/items/42?created=1").unwrap(),
            "/items/42?created=1"
        );
    }

    #[test]
    fn rejects_open_redirects() {
        assert!(validate_redirect_path("https://evil.test").is_err());
        assert!(validate_redirect_path("//evil.test").is_err());
    }

    #[test]
    fn extracts_redirect_field() {
        assert_eq!(
            extract_redirect(&json!({ "redirect": "/done" })),
            Some("/done".into())
        );
        assert_eq!(extract_redirect(&json!({ "ok": true })), None);
    }

    #[test]
    fn flash_appends_query_param() {
        assert_eq!(append_flash("/items", "Saved!"), "/items?flash=Saved%21");
        assert_eq!(
            append_flash("/items?page=2", "Saved!"),
            "/items?page=2&flash=Saved%21"
        );
    }

    #[test]
    fn flash_roundtrips_through_request() {
        let redirect = redirect_with_flash("/items", "Item created");
        // Same-origin path is preserved and validates.
        assert!(redirect.to.starts_with("/items?flash="));
        let query = crate::flow::request::parse_query(redirect.to.split_once('?').map(|x| x.1));
        let req = FlowRequest::from_parts(
            "GET",
            "/items",
            Default::default(),
            Default::default(),
            query,
        );
        assert_eq!(flash_message(&req).as_deref(), Some("Item created"));
    }
}
