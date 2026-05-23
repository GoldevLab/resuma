//! URL pattern matching for Resuma Flow pages.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct RouteMatch {
    pub params: BTreeMap<String, String>,
}

/// Match a URL path against a Resuma Flow pattern.
///
/// Patterns use:
///   * `/users/:id` — named param
///   * `/docs/*rest` — catch-all suffix
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
            params.insert(name.to_string(), path_parts[ui].to_string());
            pi += 1;
            ui += 1;
        } else if let Some(name) = seg.strip_prefix('*') {
            let rest = path_parts[ui..].join("/");
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
}
