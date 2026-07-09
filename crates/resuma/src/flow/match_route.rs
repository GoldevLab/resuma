//! URL pattern matching for Resuma Flow pages.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct RouteMatch {
    pub params: BTreeMap<String, String>,
}

/// Normalize a URL path for static route lookup (strip trailing slash except root).
pub fn normalize_lookup_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return "/".to_string();
    }
    trimmed.trim_end_matches('/').to_string()
}

/// Match a URL path against a Resuma Flow pattern.
///
/// Patterns use:
///   * `/users/:id` — named param
///   * `/docs/*rest` — catch-all suffix
///
/// **Note:** Path params are percent-decoded per segment (e.g. `%2F` → `/`).
/// If your handler uses a param for filesystem access, validate it explicitly —
/// never trust it raw.
pub fn match_route(pattern: &str, path: &str) -> Option<RouteMatch> {
    let pattern_parts: Vec<&str> = pattern
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let path_parts: Vec<&str> = path
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    if pattern == "/" && (path == "/" || path.is_empty()) {
        return Some(RouteMatch::default());
    }

    let mut params = BTreeMap::new();
    let mut pi = 0;
    let mut ui = 0;

    while pi < pattern_parts.len() && ui < path_parts.len() {
        let seg = pattern_parts[pi];
        if let Some(name) = seg.strip_prefix(':') {
            params.insert(name.to_string(), decode_param_segment(path_parts[ui]));
            pi += 1;
            ui += 1;
        } else if let Some(name) = seg.strip_prefix('*') {
            let rest = path_parts[ui..]
                .iter()
                .map(|s| decode_param_segment(s))
                .collect::<Vec<_>>()
                .join("/");
            params.insert(name.to_string(), rest);
            return Some(RouteMatch { params });
        } else if seg == path_parts[ui] {
            pi += 1;
            ui += 1;
        } else {
            return None;
        }
    }

    if pi == pattern_parts.len() && ui == path_parts.len() {
        Some(RouteMatch { params })
    } else {
        None
    }
}

fn decode_param_segment(segment: &str) -> String {
    let bytes = segment.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(s) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(byte) = u8::from_str_radix(s, 16) {
                    out.push(byte);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_static() {
        assert!(match_route("/about", "/about").is_some());
    }

    #[test]
    fn matches_param() {
        let m = match_route("/users/:id", "/users/42").unwrap();
        assert_eq!(m.params.get("id").map(String::as_str), Some("42"));
    }

    #[test]
    fn catch_all_captures_remaining_segments() {
        let m = match_route("/docs/*rest", "/docs/a/b/c").unwrap();
        assert_eq!(m.params.get("rest").map(String::as_str), Some("a/b/c"));
    }

    #[test]
    fn rejects_wrong_length_and_static_mismatch() {
        assert!(match_route("/users/:id", "/users").is_none());
        assert!(match_route("/users/:id", "/users/1/2").is_none());
        assert!(match_route("/about", "/contact").is_none());
    }

    #[test]
    fn root_pattern_matches_root_only() {
        assert!(match_route("/", "/").is_some());
        assert!(match_route("/", "").is_some());
        assert!(match_route("/", "/x").is_none());
    }

    #[test]
    fn decodes_percent_encoded_param() {
        let m = match_route("/files/:name", "/files/hello%20world").unwrap();
        assert_eq!(
            m.params.get("name").map(String::as_str),
            Some("hello world")
        );
    }

    #[test]
    fn trailing_slash_normalized() {
        assert!(match_route("/about", "/about/").is_some());
    }

    #[test]
    fn normalize_lookup_strips_trailing_slash() {
        assert_eq!(normalize_lookup_path("/about/"), "/about");
        assert_eq!(normalize_lookup_path("/"), "/");
        assert_eq!(normalize_lookup_path(""), "/");
    }
}
