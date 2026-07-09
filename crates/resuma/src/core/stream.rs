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
///
/// `html` must be produced by [`crate::ssr::render_view`] (escaped `view!` output).
/// Never pass raw user input — the client injects this via `createContextualFragment`.
pub fn stream_chunk(name: impl Into<String>, html: impl Into<String>) -> View {
    let name = name.into();
    if let Err(e) = crate::server::security::validate_chunk_id(&name) {
        tracing::warn!(stream = %name, error = %e, "invalid stream chunk name");
        return View::empty();
    }
    View::Element(Element {
        tag: "template".into(),
        attrs: vec![Attr {
            name: "data-r-stream-chunk".into(),
            value: AttrValue::Static(name),
        }],
        children: vec![Child::View(View::Raw(html.into()))],
        dom_id: None,
    })
}
