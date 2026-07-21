//! Redirect helpers for `#[submit]` and `#[server]` handlers (PRG / post-action navigation).

use crate::core::{FlowRequest, Result, ResumaError};
use crate::flow::cookie::{self, CookieOptions};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Redirect as AxumRedirect, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Query-string key used by [`redirect_with_flash`] and [`flash_message`].
pub const FLASH_KEY: &str = "flash";

/// Internal JSON key for typed redirects (avoids colliding with app fields named `redirect`).
pub const REDIRECT_KEY: &str = "__resuma_redirect";

/// Internal JSON key for `Set-Cookie` values — stripped before the body reaches the client.
pub const COOKIES_KEY: &str = "__resuma_cookies";

/// Redirect target returned from a submit or server action.
///
/// Attach session cookies with [`Redirect::with_cookie`] / [`Redirect::with_session_cookie`]
/// so no-JS PRG and JSON submits both receive `Set-Cookie` (HttpOnly-capable).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect {
    pub to: String,
    cookies: Vec<String>,
}

impl Redirect {
    pub fn to(path: impl Into<String>) -> Self {
        Self {
            to: path.into(),
            cookies: Vec::new(),
        }
    }

    /// Append a raw `Set-Cookie` header value (from [`cookie::set_cookie`] / [`cookie::clear_cookie`]).
    pub fn with_cookie(mut self, set_cookie_value: impl Into<String>) -> Self {
        self.cookies.push(set_cookie_value.into());
        self
    }

    /// Convenience: HttpOnly session cookie (`Path=/`, `SameSite=Lax`).
    pub fn with_session_cookie(self, name: &str, value: &str, max_age_secs: i64) -> Self {
        self.with_cookie(cookie::set_cookie(
            name,
            value,
            CookieOptions::session(max_age_secs),
        ))
    }

    /// Clear a cookie on this redirect (e.g. logout).
    pub fn clear_cookie(self, name: &str) -> Self {
        self.with_cookie(cookie::clear_cookie(name))
    }

    pub fn cookies(&self) -> &[String] {
        &self.cookies
    }

    pub fn into_response(self) -> Response {
        redirect_response_with_cookies(&self.to, &self.cookies)
    }
}

impl Serialize for Redirect {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let fields = 1 + usize::from(!self.cookies.is_empty());
        let mut s = serializer.serialize_struct("Redirect", fields)?;
        s.serialize_field(REDIRECT_KEY, &self.to)?;
        if !self.cookies.is_empty() {
            s.serialize_field(COOKIES_KEY, &self.cookies)?;
        }
        s.end()
    }
}

impl<'de> Deserialize<'de> for Redirect {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let to = value
            .get(REDIRECT_KEY)
            .or_else(|| value.get("redirect"))
            .or_else(|| value.get("to"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde::de::Error::missing_field(REDIRECT_KEY))?
            .to_string();
        let cookies = value
            .get(COOKIES_KEY)
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self { to, cookies })
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
///
/// Prefers [`REDIRECT_KEY`]. Bare `"redirect"` is only accepted when it is the
/// sole application field (legacy `Redirect` JSON) — payloads like
/// `{ "token": "…", "redirect": "/dashboard" }` are **not** treated as navigation
/// (auth footgun).
pub fn extract_redirect(value: &Value) -> Option<String> {
    if let Some(path) = value.get(REDIRECT_KEY).and_then(|v| v.as_str()) {
        return validate_redirect_path(path).ok();
    }
    let obj = value.as_object()?;
    let path = obj.get("redirect")?.as_str()?;
    let allowed = obj
        .keys()
        .all(|k| k == "redirect" || k == COOKIES_KEY || k == "cookies");
    if !allowed {
        return None;
    }
    validate_redirect_path(path).ok()
}

/// Take `Set-Cookie` values embedded by [`Redirect`] serialization and remove them
/// from the JSON body (cookies must only appear as HTTP headers).
pub fn take_response_cookies(value: &mut Value) -> Vec<String> {
    let Some(obj) = value.as_object_mut() else {
        return Vec::new();
    };
    let raw = obj.remove(COOKIES_KEY).or_else(|| obj.remove("cookies"));
    match raw {
        Some(Value::Array(items)) => items
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .filter(|s| !s.is_empty() && HeaderValue::from_str(s).is_ok())
            .collect(),
        Some(Value::String(s)) if !s.is_empty() && HeaderValue::from_str(&s).is_ok() => {
            vec![s]
        }
        _ => Vec::new(),
    }
}

/// Scrub internal redirect/cookie keys from a submit/action JSON `value`.
pub fn scrub_resuma_meta(value: &mut Value) {
    if let Some(obj) = value.as_object_mut() {
        obj.remove(REDIRECT_KEY);
        obj.remove(COOKIES_KEY);
        // Legacy sole-key redirect shape: drop after extraction so clients use top-level `redirect`.
        if obj.len() == 1 && obj.contains_key("redirect") {
            obj.remove("redirect");
        }
    }
}

/// Extract redirect + cookies and scrub meta keys. Used by submit/action HTTP layers.
pub fn prepare_navigation(value: &mut Value) -> (Option<String>, Vec<String>) {
    let cookies = take_response_cookies(value);
    let redirect = extract_redirect(value);
    scrub_resuma_meta(value);
    (redirect, cookies)
}

/// Append `Set-Cookie` headers onto an HTTP response.
pub fn attach_set_cookies(res: &mut Response, cookies: &[String]) {
    for c in cookies {
        if let Ok(val) = HeaderValue::from_str(c) {
            res.headers_mut().append(header::SET_COOKIE, val);
        }
    }
}

/// Reject open redirects — only root-relative paths are allowed.
///
/// Percent-decodes the path and rejects encoded slashes (`%2f`), backslashes,
/// protocol-relative targets (`//`), and control characters.
pub fn validate_redirect_path(path: &str) -> Result<String> {
    if path.is_empty() {
        return Err(ResumaError::Other("invalid redirect path (empty)".into()));
    }
    if path.contains('\\') || path.contains('\0') {
        return Err(ResumaError::Other(
            "invalid redirect path (backslash or null byte)".into(),
        ));
    }
    let lower = path.to_ascii_lowercase();
    if lower.contains("%2f") || lower.contains("%5c") {
        return Err(ResumaError::Other(
            "invalid redirect path (encoded slash or backslash)".into(),
        ));
    }
    if !path.starts_with('/') || path.starts_with("//") {
        return Err(ResumaError::Other(format!(
            "invalid redirect path `{path}` (must start with `/`, not `//`)"
        )));
    }

    let decoded = percent_decode_path(path)?;
    if !decoded.starts_with('/') || decoded.starts_with("//") || decoded.contains("//") {
        return Err(ResumaError::Other(format!(
            "invalid redirect path `{path}` (open redirect after decode)"
        )));
    }
    if decoded.chars().any(|c| c.is_control()) {
        return Err(ResumaError::Other(
            "invalid redirect path (control character)".into(),
        ));
    }
    Ok(decoded)
}

fn percent_decode_path(path: &str) -> Result<String> {
    let bytes = path.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() {
                return Err(ResumaError::Other(
                    "invalid redirect path (truncated percent-encoding)".into(),
                ));
            }
            let h1 = bytes[i + 1];
            let h2 = bytes[i + 2];
            let hex = [h1, h2];
            let s = std::str::from_utf8(&hex)
                .map_err(|_| ResumaError::Other("invalid redirect path encoding".into()))?;
            let byte = u8::from_str_radix(s, 16)
                .map_err(|_| ResumaError::Other("invalid redirect path encoding".into()))?;
            if byte == 0 {
                return Err(ResumaError::Other(
                    "invalid redirect path (null byte)".into(),
                ));
            }
            out.push(byte);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| ResumaError::Other("invalid redirect path utf-8".into()))
}

/// HTTP 303 See Other — standard PRG response for form submits without JavaScript.
pub fn redirect_response(path: &str) -> Response {
    redirect_response_with_cookies(path, &[])
}

/// HTTP 303 with optional `Set-Cookie` headers.
pub fn redirect_response_with_cookies(path: &str, cookies: &[String]) -> Response {
    match validate_redirect_path(path) {
        Ok(loc) => {
            let mut res = AxumRedirect::to(&loc).into_response();
            // Axum Redirect defaults to 303 for `to` in recent versions; ensure SEE_OTHER.
            *res.status_mut() = StatusCode::SEE_OTHER;
            attach_set_cookies(&mut res, cookies);
            res
        }
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

/// Attach `Location` on JSON responses when a redirect hint is present.
pub fn redirect_json_headers(path: &str) -> Option<[(header::HeaderName, String); 1]> {
    validate_redirect_path(path)
        .ok()
        .map(|loc| [(header::LOCATION, loc)])
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
        assert!(validate_redirect_path("/%2f%2fevil.com").is_err());
        assert!(validate_redirect_path("/%2F%2Fevil.com").is_err());
        assert!(validate_redirect_path("/foo\\@evil.com").is_err());
    }

    #[test]
    fn extracts_typed_redirect() {
        assert_eq!(
            extract_redirect(&json!({ REDIRECT_KEY: "/done" })),
            Some("/done".into())
        );
        assert_eq!(
            extract_redirect(&json!({ "redirect": "/done" })),
            Some("/done".into())
        );
        assert_eq!(extract_redirect(&json!({ "ok": true })), None);
    }

    #[test]
    fn ignores_redirect_amid_app_fields() {
        assert_eq!(
            extract_redirect(&json!({
                "token": "abc",
                "redirect": "/dashboard"
            })),
            None
        );
    }

    #[test]
    fn serialize_roundtrip_with_cookies() {
        let r = Redirect::to("/dashboard").with_session_cookie("sid", "t1", 60);
        let mut v = serde_json::to_value(&r).unwrap();
        assert_eq!(extract_redirect(&v).as_deref(), Some("/dashboard"));
        let cookies = take_response_cookies(&mut v);
        assert_eq!(cookies.len(), 1);
        assert!(cookies[0].contains("sid=t1"));
        assert!(cookies[0].contains("HttpOnly"));
        scrub_resuma_meta(&mut v);
        assert!(v.as_object().map(|o| o.is_empty()).unwrap_or(false));
    }

    #[test]
    fn prepare_navigation_strips_secrets_from_body() {
        let r = Redirect::to("/a").with_cookie("sid=secret; Path=/; HttpOnly");
        let mut v = serde_json::to_value(&r).unwrap();
        let (loc, cookies) = prepare_navigation(&mut v);
        assert_eq!(loc.as_deref(), Some("/a"));
        assert_eq!(cookies.len(), 1);
        assert!(v.get(COOKIES_KEY).is_none());
        assert!(v.get(REDIRECT_KEY).is_none());
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

    #[test]
    fn legacy_redirect_json_still_works() {
        // Pre-1.2.16 Serialize shape
        let v = json!({ "redirect": "/items?created=1" });
        assert_eq!(extract_redirect(&v).as_deref(), Some("/items?created=1"));
    }
}
