//! Live execution graph (`<div data-r-flow-graph>`).

use resuma::core::view::{Attr, AttrValue, Child, Element, View};

/// Render a mount point for the client-side flow graph widget.
///
/// Subscribes to `GET /_resuma/graph/{id}/events` (SSE) via `@resuma/flow` runtime.
pub fn flow_graph(id: impl Into<String>, live: bool) -> View {
    flow_graph_auth(id, live, None)
}

/// Same as [`flow_graph`] with a scoped access token from `StartWorkerResponse` for production.
pub fn flow_graph_auth(id: impl Into<String>, live: bool, access_token: Option<String>) -> View {
    let id = id.into();
    let mut attrs = vec![
        Attr {
            name: "class".into(),
            value: AttrValue::Static("r-flow-graph".into()),
        },
        Attr {
            name: "data-r-flow-graph".into(),
            value: AttrValue::Static(id),
        },
        Attr {
            name: "data-r-flow-graph-live".into(),
            value: AttrValue::Static(if live { "true" } else { "false" }.into()),
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
        children: vec![
            Child::View(View::Element(Element {
                tag: "div".into(),
                attrs: vec![
                    Attr {
                        name: "class".into(),
                        value: AttrValue::Static("r-flow-graph__track".into()),
                    },
                    Attr {
                        name: "data-r-flow-graph-track".into(),
                        value: AttrValue::Static("true".into()),
                    },
                ],
                children: vec![Child::Text("…".into())],
                dom_id: None,
            })),
            Child::View(View::Element(Element {
                tag: "p".into(),
                attrs: vec![
                    Attr {
                        name: "class".into(),
                        value: AttrValue::Static("r-flow-graph__status".into()),
                    },
                    Attr {
                        name: "data-r-flow-graph-status".into(),
                        value: AttrValue::Static("true".into()),
                    },
                ],
                children: vec![Child::Text("Loading graph…".into())],
                dom_id: None,
            })),
        ],
        dom_id: None,
    })
}
