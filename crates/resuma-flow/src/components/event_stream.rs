//! Live event timeline (`<div data-r-event-stream>`).

use resuma::core::view::{Attr, AttrValue, Child, Element, View};

/// Render a mount point for the client-side event stream widget.
pub fn event_stream(id: impl Into<String>) -> View {
    event_stream_auth(id, None)
}

/// Same as [`event_stream`] with graph-scoped access token for production auth.
pub fn event_stream_auth(id: impl Into<String>, access_token: Option<String>) -> View {
    let id = id.into();
    let mut attrs = vec![
        Attr {
            name: "class".into(),
            value: AttrValue::Static("r-event-stream".into()),
        },
        Attr {
            name: "data-r-event-stream".into(),
            value: AttrValue::Static(id),
        },
    ];
    if let Some(token) = access_token.filter(|t| !t.is_empty()) {
        attrs.push(Attr {
            name: "data-r-graph-token".into(),
            value: AttrValue::Static(token),
        });
    }
    View::Element(Element {
        tag: "div".into(),
        attrs,
        children: vec![Child::View(View::Element(Element {
            tag: "div".into(),
            attrs: vec![
                Attr {
                    name: "class".into(),
                    value: AttrValue::Static("r-event-stream__viewport".into()),
                },
                Attr {
                    name: "data-r-event-stream-viewport".into(),
                    value: AttrValue::Static("true".into()),
                },
            ],
            children: vec![Child::View(View::Element(Element {
                tag: "ul".into(),
                attrs: vec![Attr {
                    name: "class".into(),
                    value: AttrValue::Static("r-event-stream-list".into()),
                }],
                children: vec![],
                dom_id: None,
            }))],
            dom_id: None,
        }))],
        dom_id: None,
    })
}
