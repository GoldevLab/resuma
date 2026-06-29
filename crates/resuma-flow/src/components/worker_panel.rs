//! Worker controls — pause / resume / replay hooks.

use resuma::core::view::{Attr, AttrValue, Child, Element, View};

/// Minimal control panel wired to execution HTTP routes.
pub fn worker_panel(id: impl Into<String>) -> View {
    worker_panel_auth(id, None)
}

/// Same as [`worker_panel`] with graph-scoped access token for production auth.
pub fn worker_panel_auth(id: impl Into<String>, access_token: Option<String>) -> View {
    let id = id.into();
    let mut attrs = vec![
        Attr {
            name: "class".into(),
            value: AttrValue::Static("r-worker-panel".into()),
        },
        Attr {
            name: "data-r-worker-panel".into(),
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
        children: vec![
            Child::View(View::Element(Element {
                tag: "button".into(),
                attrs: vec![
                    Attr {
                        name: "type".into(),
                        value: AttrValue::Static("button".into()),
                    },
                    Attr {
                        name: "data-r-worker-pause".into(),
                        value: AttrValue::Static("true".into()),
                    },
                ],
                children: vec![Child::Text("Pause".into())],
                dom_id: None,
            })),
            Child::View(View::Element(Element {
                tag: "button".into(),
                attrs: vec![
                    Attr {
                        name: "type".into(),
                        value: AttrValue::Static("button".into()),
                    },
                    Attr {
                        name: "data-r-worker-resume".into(),
                        value: AttrValue::Static("true".into()),
                    },
                ],
                children: vec![Child::Text("Resume".into())],
                dom_id: None,
            })),
            Child::View(View::Element(Element {
                tag: "button".into(),
                attrs: vec![
                    Attr {
                        name: "type".into(),
                        value: AttrValue::Static("button".into()),
                    },
                    Attr {
                        name: "data-r-worker-cancel".into(),
                        value: AttrValue::Static("true".into()),
                    },
                ],
                children: vec![Child::Text("Cancel".into())],
                dom_id: None,
            })),
            Child::View(View::Element(Element {
                tag: "button".into(),
                attrs: vec![
                    Attr {
                        name: "type".into(),
                        value: AttrValue::Static("button".into()),
                    },
                    Attr {
                        name: "data-r-worker-replay".into(),
                        value: AttrValue::Static("true".into()),
                    },
                ],
                children: vec![Child::Text("Replay".into())],
                dom_id: None,
            })),
        ],
        dom_id: None,
    })
}
