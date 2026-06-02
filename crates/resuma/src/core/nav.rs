//! Navigation link with active-state styling.

use super::view::{Attr, AttrValue, Child, Element, View};

/// Render an `<a>` that adds `active_class` when `href` matches `current_path`.
pub fn nav_link(
    href: impl Into<String>,
    current_path: &str,
    active_class: impl Into<String>,
    class: impl Into<String>,
    children: Vec<Child>,
) -> View {
    let href = href.into();
    let active_class = active_class.into();
    let class = class.into();
    let is_active = paths_match(&href, current_path);
    let merged_class = if is_active && !active_class.is_empty() {
        format!("{class} {active_class}")
    } else {
        class
    };

    View::Element(Element {
        tag: "a".into(),
        attrs: vec![
            Attr {
                name: "href".into(),
                value: AttrValue::Static(href),
            },
            Attr {
                name: "class".into(),
                value: AttrValue::Static(merged_class),
            },
            Attr {
                name: "data-r-nav".into(),
                value: AttrValue::Static("true".into()),
            },
            Attr {
                name: "data-r-active-class".into(),
                value: AttrValue::Static(active_class.clone()),
            },
        ],
        children,
        dom_id: None,
    })
}

fn paths_match(href: &str, current: &str) -> bool {
    if href == current {
        return true;
    }
    let (href_path, href_query) = split_path_query(href);
    let (cur_path, cur_query) = split_path_query(current);
    if href_query.is_some() {
        return href_path == cur_path && href_query == cur_query;
    }
    if href_path == cur_path {
        return true;
    }
    if href_path != "/" && cur_path.starts_with(href_path) {
        return cur_path
            .as_bytes()
            .get(href_path.len())
            .is_none_or(|b| *b == b'/');
    }
    false
}

fn split_path_query(s: &str) -> (&str, Option<&str>) {
    match s.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (s, None),
    }
}

#[cfg(test)]
mod tests {
    use super::paths_match;

    #[test]
    fn path_only_active_with_query_on_current() {
        assert!(paths_match("/reservar", "/reservar?fecha=2026-06-02"));
    }

    #[test]
    fn query_href_requires_exact_match() {
        assert!(paths_match("/book?fecha=1", "/book?fecha=1"));
        assert!(!paths_match("/book?fecha=1", "/book?fecha=2"));
        assert!(!paths_match("/book?fecha=1", "/book"));
    }
}
