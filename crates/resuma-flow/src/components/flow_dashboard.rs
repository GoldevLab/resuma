//! Live ops dashboard — polls `exec_status` server action or `GET /_resuma/status`.

use resuma::core::view::{Attr, AttrValue, Child, Element, View};
use resuma::exec::ExecStatus;

/// Ops dashboard with default 5s polling.
pub fn flow_dashboard() -> View {
    flow_dashboard_poll(5000, None)
}

/// Dashboard with custom poll interval (milliseconds).
pub fn flow_dashboard_poll(poll_ms: u32, initial: Option<ExecStatus>) -> View {
    let poll = poll_ms.to_string();
    let mut attrs = vec![
        Attr {
            name: "class".into(),
            value: AttrValue::Static("r-flow-dash".into()),
        },
        Attr {
            name: "data-r-flow-dashboard".into(),
            value: AttrValue::Static("true".into()),
        },
        Attr {
            name: "data-r-flow-dashboard-poll".into(),
            value: AttrValue::Static(poll),
        },
    ];
    if let Some(snap) = initial.and_then(|s| serde_json::to_string(&s).ok()) {
        attrs.push(Attr {
            name: "data-r-flow-dashboard-init".into(),
            value: AttrValue::Static(snap),
        });
    }
    View::Element(Element {
        tag: "div".into(),
        attrs,
        children: vec![Child::View(dashboard_shell())],
        dom_id: None,
    })
}

/// SSR snapshot + live polling (recommended for production pages).
pub fn flow_dashboard_live(initial: ExecStatus) -> View {
    flow_dashboard_poll(5000, Some(initial))
}

fn dashboard_shell() -> View {
    View::Element(Element {
        tag: "div".into(),
        attrs: vec![Attr {
            name: "data-r-flow-dashboard-root".into(),
            value: AttrValue::Static("true".into()),
        }],
        children: vec![Child::Text(String::new())],
        dom_id: None,
    })
}
