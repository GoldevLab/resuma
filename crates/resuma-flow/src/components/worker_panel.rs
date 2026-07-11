//! Worker controls — pause / resume / replay hooks.

use resuma::core::view::{Attr, AttrValue, Child, Element, View};

/// Minimal control panel wired to execution HTTP routes.
pub fn worker_panel(id: impl Into<String>) -> View {
    worker_panel_auth(id, None)
}

fn control_btn(class: &'static str, data_attr: &'static str, label: &'static str) -> View {
    View::Element(Element {
        tag: "button".into(),
        attrs: vec![
            Attr {
                name: "type".into(),
                value: AttrValue::Static("button".into()),
            },
            Attr {
                name: "class".into(),
                value: AttrValue::Static(class.into()),
            },
            Attr {
                name: data_attr.into(),
                value: AttrValue::Static("true".into()),
            },
        ],
        children: vec![Child::Text(label.into())],
        dom_id: None,
    })
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
                tag: "div".into(),
                attrs: vec![Attr {
                    name: "class".into(),
                    value: AttrValue::Static("r-worker-panel__actions".into()),
                }],
                children: vec![
                    Child::View(control_btn(
                        "r-flow-control r-flow-control--ghost r-flow-control--pause",
                        "data-r-worker-pause",
                        "Pause",
                    )),
                    Child::View(control_btn(
                        "r-flow-control r-flow-control--ghost r-flow-control--resume",
                        "data-r-worker-resume",
                        "Resume",
                    )),
                    Child::View(control_btn(
                        "r-flow-control r-flow-control--danger",
                        "data-r-worker-cancel",
                        "Cancel",
                    )),
                    Child::View(control_btn(
                        "r-flow-control r-flow-control--ghost r-flow-control--replay",
                        "data-r-worker-replay",
                        "Replay",
                    )),
                ],
                dom_id: None,
            })),
            Child::View(View::Element(Element {
                tag: "p".into(),
                attrs: vec![
                    Attr {
                        name: "class".into(),
                        value: AttrValue::Static("r-worker-panel__status".into()),
                    },
                    Attr {
                        name: "data-r-worker-status".into(),
                        value: AttrValue::Static("true".into()),
                    },
                    Attr {
                        name: "aria-live".into(),
                        value: AttrValue::Static("polite".into()),
                    },
                ],
                children: vec![],
                dom_id: None,
            })),
        ],
        dom_id: None,
    })
}
