//! Streaming SSR slot — placeholder replaced by a later chunk in streaming responses.

use super::view::{Attr, AttrValue, Child, Element, View};

/// Placeholder for a loader-driven region in streaming SSR.
pub fn stream_slot(name: impl Into<String>) -> View {
    View::Element(Element {
        tag: "template".into(),
        attrs: vec![Attr {
            name: "data-r-stream".into(),
            value: AttrValue::Static(name.into()),
        }],
        children: vec![Child::Text("Loading…".into())],
        dom_id: None,
    })
}

/// Resolved chunk emitted after a streamed loader completes.
pub fn stream_chunk(name: impl Into<String>, html: impl Into<String>) -> View {
    View::Element(Element {
        tag: "template".into(),
        attrs: vec![Attr {
            name: "data-r-stream-chunk".into(),
            value: AttrValue::Static(name.into()),
        }],
        children: vec![Child::View(View::Raw(html.into()))],
        dom_id: None,
    })
}
