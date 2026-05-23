//! Navigation link with active-state styling.

use crate::view::{Attr, AttrValue, Child, Element, View};

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
        ],
        children,
        dom_id: None,
    })
}

fn paths_match(href: &str, current: &str) -> bool {
    if href == current {
        return true;
    }
    if href != "/" && current.starts_with(href) {
        return current
            .as_bytes()
            .get(href.len())
            .is_none_or(|b| *b == b'/');
    }
    false
}
