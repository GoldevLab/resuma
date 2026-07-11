//! Full execution view — graph + controls + event stream in one layout.

use resuma::core::view::{Attr, AttrValue, Child, Element, View};

use super::event_stream::event_stream_auth;
use super::flow_graph::flow_graph_auth;
use super::styles::flow_styles;
use super::worker_panel::worker_panel_auth;

/// Combined execution panel: live graph, worker controls, and event timeline.
pub fn flow_execution(graph_id: impl Into<String>, live: bool) -> View {
    flow_execution_auth(graph_id, live, None)
}

/// Same as [`flow_execution`] with graph-scoped access token.
pub fn flow_execution_auth(
    graph_id: impl Into<String>,
    live: bool,
    access_token: Option<String>,
) -> View {
    flow_execution_auth_opts(graph_id, live, access_token, true)
}

/// Same as [`flow_execution_auth`] but omits embedded [`flow_styles`] (for dynamic HTML injection).
///
/// Include [`super::flow_styles_link`] once on the page, or call `flow_styles()` in the layout.
pub fn flow_execution_panel_auth(
    graph_id: impl Into<String>,
    live: bool,
    access_token: Option<String>,
) -> View {
    flow_execution_auth_opts(graph_id, live, access_token, false)
}

fn flow_execution_auth_opts(
    graph_id: impl Into<String>,
    live: bool,
    access_token: Option<String>,
    include_styles: bool,
) -> View {
    let id = graph_id.into();
    let mut children = Vec::new();
    if include_styles {
        children.push(Child::View(flow_styles()));
    }
    children.extend([
        Child::View(View::Element(Element {
            tag: "div".into(),
            attrs: vec![Attr {
                name: "class".into(),
                value: AttrValue::Static("r-flow-exec__panel".into()),
            }],
            children: vec![
                Child::View(View::Element(Element {
                    tag: "h3".into(),
                    attrs: vec![],
                    children: vec![Child::Text("Execution graph".into())],
                    dom_id: None,
                })),
                Child::View(flow_graph_auth(id.clone(), live, access_token.clone())),
            ],
            dom_id: None,
        })),
        Child::View(View::Element(Element {
            tag: "aside".into(),
            attrs: vec![Attr {
                name: "class".into(),
                value: AttrValue::Static("r-flow-exec__side".into()),
            }],
            children: vec![
                Child::View(View::Element(Element {
                    tag: "div".into(),
                    attrs: vec![Attr {
                        name: "class".into(),
                        value: AttrValue::Static("r-flow-exec__panel".into()),
                    }],
                    children: vec![
                        Child::View(View::Element(Element {
                            tag: "h3".into(),
                            attrs: vec![],
                            children: vec![Child::Text("Controls".into())],
                            dom_id: None,
                        })),
                        Child::View(worker_panel_auth(id.clone(), access_token.clone())),
                    ],
                    dom_id: None,
                })),
                Child::View(View::Element(Element {
                    tag: "div".into(),
                    attrs: vec![Attr {
                        name: "class".into(),
                        value: AttrValue::Static("r-flow-exec__panel".into()),
                    }],
                    children: vec![
                        Child::View(View::Element(Element {
                            tag: "h3".into(),
                            attrs: vec![],
                            children: vec![Child::Text("Event stream".into())],
                            dom_id: None,
                        })),
                        Child::View(event_stream_auth(id.clone(), access_token)),
                    ],
                    dom_id: None,
                })),
            ],
            dom_id: None,
        })),
    ]);
    View::Element(Element {
        tag: "div".into(),
        attrs: vec![
            Attr {
                name: "class".into(),
                value: AttrValue::Static("r-flow-exec".into()),
            },
            Attr {
                name: "data-r-flow-execution".into(),
                value: AttrValue::Static(id),
            },
        ],
        children,
        dom_id: None,
    })
}

/// Ops page: system dashboard + optional active execution panel.
pub fn flow_ops_page(initial_status: resuma::exec::ExecStatus) -> View {
    View::Element(Element {
        tag: "div".into(),
        attrs: vec![Attr {
            name: "class".into(),
            value: AttrValue::Static("r-flow-ops-page".into()),
        }],
        children: vec![
            Child::View(flow_styles()),
            Child::View(super::flow_dashboard::flow_dashboard_live(initial_status)),
        ],
        dom_id: None,
    })
}
