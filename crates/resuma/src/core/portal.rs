//! Portals — render children into a remote DOM target on the client.

use super::view::{Attr, AttrValue, Child, Element, View};

/// Portal content projected to `#target` (or `[data-r-portal-target="target"]`).
pub fn portal(target: impl Into<String>, children: Vec<Child>) -> View {
    View::Element(Element {
        tag: "template".into(),
        attrs: vec![Attr {
            name: "data-r-portal".into(),
            value: AttrValue::Static(target.into()),
        }],
        children,
        dom_id: None,
    })
}
