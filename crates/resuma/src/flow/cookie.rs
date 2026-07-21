//! Cookie helpers for session auth and PRG responses.
//!
//! Prefer setting session cookies via [`crate::Redirect::with_cookie`] on
//! `#[submit]` / `#[server]` returns so the browser receives `Set-Cookie`
//! (HttpOnly) instead of `document.cookie`.

use std::fmt;

/// SameSite attribute for [`CookieOptions`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SameSite {
    #[default]
    Lax,
    Strict,
    None,
}

impl fmt::Display for SameSite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Lax => "Lax",
            Self::Strict => "Strict",
            Self::None => "None",
        })
    }
}

/// Options for [`set_cookie`].
#[derive(Debug, Clone)]
pub struct CookieOptions {
    pub path: String,
    pub max_age: Option<i64>,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: SameSite,
}

impl Default for CookieOptions {
    fn default() -> Self {
        Self {
            path: "/".into(),
            max_age: None,
            http_only: true,
            secure: false,
            same_site: SameSite::Lax,
        }
    }
}

impl CookieOptions {
    pub fn session(max_age_secs: i64) -> Self {
        Self {
            max_age: Some(max_age_secs),
            ..Default::default()
        }
    }

    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = same_site;
        self
    }

    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }
}

/// Build a `Set-Cookie` header value (name=value + attributes).
pub fn set_cookie(name: &str, value: &str, opts: CookieOptions) -> String {
    let mut out = format!("{name}={value}; Path={}", opts.path);
    if let Some(max_age) = opts.max_age {
        out.push_str(&format!("; Max-Age={max_age}"));
    }
    if opts.http_only {
        out.push_str("; HttpOnly");
    }
    out.push_str(&format!("; SameSite={}", opts.same_site));
    if opts.secure || matches!(opts.same_site, SameSite::None) {
        out.push_str("; Secure");
    }
    out
}

/// Expire a cookie (Max-Age=0), HttpOnly + SameSite=Lax by default.
pub fn clear_cookie(name: &str) -> String {
    set_cookie(
        name,
        "",
        CookieOptions {
            max_age: Some(0),
            ..Default::default()
        },
    )
}

/// Read one cookie value from a raw `Cookie` request header.
pub fn cookie_value(raw: &str, name: &str) -> Option<String> {
    raw.split(';').find_map(|part| {
        let (k, v) = part.trim().split_once('=')?;
        if k == name {
            Some(percent_decode(v))
        } else {
            None
        }
    })
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((h << 4 | l) as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_cookie_http_only_session() {
        let c = set_cookie("sid", "abc", CookieOptions::session(3600));
        assert!(c.contains("sid=abc"));
        assert!(c.contains("HttpOnly"));
        assert!(c.contains("Max-Age=3600"));
        assert!(c.contains("SameSite=Lax"));
    }

    #[test]
    fn clear_cookie_expires() {
        let c = clear_cookie("sid");
        assert!(c.contains("Max-Age=0"));
        assert!(c.contains("HttpOnly"));
    }

    #[test]
    fn parses_cookie_header() {
        let raw = "a=1; sid=token%2Dvalue; b=2";
        assert_eq!(cookie_value(raw, "sid").as_deref(), Some("token-value"));
    }
}
