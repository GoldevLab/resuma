//! Progressive-enhancement `<Form>` helper for `#[submit]` handlers.

use resuma_core::view::{Attr, AttrValue, Child, Element, View};
use resuma_server::page_csrf;
use resuma_server::CSRF_FIELD;

/// Build a `<form>` wired to a Resuma Flow `#[submit]` handler.
///
/// Renders `method="POST"` and `action="/_resuma/submit/<name>"` so the form
/// works without JavaScript. The client runtime intercepts submit when loaded.
/// Includes a CSRF hidden field when a page token is staged.
pub fn form(submit_name: &str, attrs: Vec<(String, AttrValue)>, children: Vec<Child>) -> View {
    let csrf = page_csrf();
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
    if !csrf.is_empty() {
        element.children.insert(
            0,
            Child::View(View::Element(Element {
                tag: "input".into(),
                attrs: vec![
                    Attr {
                        name: "type".into(),
                        value: AttrValue::Static("hidden".into()),
                    },
                    Attr {
                        name: "name".into(),
                        value: AttrValue::Static(CSRF_FIELD.into()),
                    },
                    Attr {
                        name: "value".into(),
                        value: AttrValue::Static(csrf),
                    },
                ],
                children: vec![],
                dom_id: None,
            })),
        );
    }
    for (name, value) in attrs {
        element.attrs.push(Attr { name, value });
    }
    View::Element(element)
}
