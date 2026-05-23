//! View Transitions API helpers.

use super::view::{Attr, AttrValue, Child, Element, View};

/// Wrap content in a View Transition boundary (`document.startViewTransition`).
pub fn with_view_transition(name: impl Into<String>, children: Vec<Child>) -> View {
    View::Element(Element {
        tag: "div".into(),
        attrs: vec![Attr {
            name: "data-r-vt".into(),
            value: AttrValue::Static(name.into()),
        }],
        children,
        dom_id: None,
    })
}
