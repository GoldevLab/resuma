//! Progressive-enhancement `<Form>` helper for `#[submit]` handlers.

use resuma_core::view::{Attr, AttrValue, Child, Element, View};

/// Build a `<form>` wired to a Resuma Flow `#[submit]` handler.
///
/// Renders `method="POST"` and `action="/_resuma/submit/<name>"` so the form
/// works without JavaScript. The client runtime intercepts submit when loaded.
pub fn form(submit_name: &str, attrs: Vec<(String, AttrValue)>, children: Vec<Child>) -> View {
    let mut element = Element {
        tag: "form".into(),
        attrs: vec![
            Attr {
                name: "method".into(),
                value: AttrValue::Static("POST".into()),
            },
            Attr {
                name: "action".into(),
                value: AttrValue::Static(format!("/_resuma/submit/{submit_name}")),
            },
            Attr {
                name: "data-r-submit".into(),
                value: AttrValue::Static(submit_name.into()),
            },
        ],
        children,
        dom_id: None,
    };
    for (name, value) in attrs {
        element.attrs.push(Attr { name, value });
    }
    View::Element(element)
}
