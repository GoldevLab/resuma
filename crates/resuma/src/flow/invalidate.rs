//! Stable loader invalidation — re-run server `#[load]` handlers via SPA navigation.

use crate::core::view::{Child, View};

use super::nav::build_query_href;

/// Build an href that triggers SPA navigation and re-runs server loaders for `path`.
///
/// Appends `_r` (cache-bust) so repeated invalidations refetch. Use with
/// `NavLink`, `loader_refresh_input`, or `js! { await __resuma.invalidate("/path"); }`.
pub fn invalidate_href(path: &str, query: &[(&str, &str)]) -> String {
    let mut pairs: Vec<(&str, &str)> = query.to_vec();
    pairs.push(("_r", "1"));
    build_query_href(path, &pairs)
}

/// Same as [`invalidate_href`] but uses a unique `_r` timestamp on each call.
pub fn invalidate_href_now(path: &str, query: &[(&str, &str)]) -> String {
    let bust = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_else(|_| "0".into());
    let mut owned: Vec<(String, String)> = query
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    owned.push(("_r".into(), bust));
    let refs: Vec<(&str, &str)> = owned
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    build_query_href(path, &refs)
}

/// `<a data-r-nav>` that navigates to [`invalidate_href_now`] for the current path + optional query.
pub fn invalidate_link(
    path: &str,
    query: &[(&str, &str)],
    label: impl Into<String>,
) -> View {
    let href = invalidate_href_now(path, query);
    View::Element(crate::core::view::Element {
        tag: "a".into(),
        attrs: vec![
            crate::core::view::Attr {
                name: "href".into(),
                value: crate::core::view::AttrValue::Static(href),
            },
            crate::core::view::Attr {
                name: "data-r-nav".into(),
                value: crate::core::view::AttrValue::Static("true".into()),
            },
        ],
        children: vec![Child::Text(label.into())],
        dom_id: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalidate_href_adds_cache_bust_param() {
        let href = invalidate_href("/users", &[("q", "a")]);
        assert!(href.starts_with("/users?"));
        assert!(href.contains("q=a"));
        assert!(href.contains("_r=1"));
    }
}
