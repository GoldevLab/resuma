//! Live ops dashboard — polls `exec_status` server action or `GET /_resuma/status`.

use resuma::core::view::{Attr, AttrValue, Child, Element, View};
use resuma::exec::ExecStatus;
use resuma::prelude::*;

/// Ops dashboard with default 5s polling.
pub fn flow_dashboard() -> View {
    flow_dashboard_poll(5000, None)
}

/// Dashboard with custom poll interval (milliseconds).
pub fn flow_dashboard_poll(poll_ms: u32, initial: Option<ExecStatus>) -> View {
    let poll = poll_ms.to_string();
    let init_ref = initial.as_ref();
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
    if let Some(snap) = init_ref.and_then(|s| serde_json::to_string(s).ok()) {
        attrs.push(Attr {
            name: "data-r-flow-dashboard-init".into(),
            value: AttrValue::Static(snap),
        });
    }
    View::Element(Element {
        tag: "div".into(),
        attrs,
        children: vec![Child::View(dashboard_shell(init_ref))],
        dom_id: None,
    })
}

/// SSR snapshot + live polling (recommended for production pages).
pub fn flow_dashboard_live(initial: ExecStatus) -> View {
    flow_dashboard_poll(5000, Some(initial))
}

fn dashboard_shell(initial: Option<&ExecStatus>) -> View {
    let children = match initial {
        Some(status) => vec![Child::View(dashboard_ssr(status))],
        None => vec![],
    };
    View::Element(Element {
        tag: "div".into(),
        attrs: vec![Attr {
            name: "data-r-flow-dashboard-root".into(),
            value: AttrValue::Static("true".into()),
        }],
        children,
        dom_id: None,
    })
}

fn format_uptime(ms: u64) -> String {
    let s = ms / 1000;
    let h = s / 3600;
    let m = (s % 3600) / 60;
    let sec = s % 60;
    if h > 0 {
        format!("{h}h {m}m")
    } else if m > 0 {
        format!("{m}m {sec}s")
    } else {
        format!("{sec}s")
    }
}

fn dashboard_ssr(status: &ExecStatus) -> View {
    let pending: usize = status.queues.iter().map(|q| q.pending).sum();
    let processing: usize = status.queues.iter().map(|q| q.processing).sum();
    let badge_class = if pending > 10 {
        "r-flow-dash__badge r-flow-dash__badge--warn"
    } else if status.ok {
        "r-flow-dash__badge"
    } else {
        "r-flow-dash__badge r-flow-dash__badge--err"
    };
    let health = if status.ok { "healthy" } else { "degraded" };
    let badge = format!("{health} · uptime {}", format_uptime(status.uptime_ms));

    let worker_chips: Vec<View> = if status.workers.names.is_empty() {
        vec![view! { <span class="r-flow-dash__chip">"none registered"</span> }]
    } else {
        status
            .workers
            .names
            .iter()
            .map(|name| view! { <span class="r-flow-dash__chip">{name.clone()}</span> })
            .collect()
    };

    let queue_rows: Vec<View> = status
        .queues
        .iter()
        .map(|q| {
            let total = q.pending + q.processing + q.done + q.failed;
            let total = if total == 0 { 1 } else { total };
            let pct = ((q.processing as f64 / total as f64) * 100.0).round() as u32;
            view! {
                <tr>
                    <td>{q.queue.clone()}</td>
                    <td>{q.pending.to_string()}</td>
                    <td>{q.processing.to_string()}</td>
                    <td>{q.done.to_string()}</td>
                    <td>{q.failed.to_string()}</td>
                    <td>
                        <div class="r-flow-dash__bar">
                            <svg viewBox="0 0 100 6" preserveAspectRatio="none" aria-hidden="true">
                                <rect class="r-flow-dash__bar-fill" width={pct.to_string()} height="6" rx="3" />
                            </svg>
                        </div>
                    </td>
                </tr>
            }
        })
        .collect();

    let sched = format!(
        "{} enabled · {} total · {} due now",
        status.scheduler.enabled, status.scheduler.total, status.scheduler.due
    );

    view! {
        <>
            <header class="r-flow-dash__header">
                <h2 class="r-flow-dash__title">"Resuma OS"</h2>
                <span class={badge_class.to_string()}>{badge}</span>
            </header>
            <div class="r-flow-dash__grid">
                <div class="r-flow-dash__stat">
                    <p class="r-flow-dash__stat-label">"Workers"</p>
                    <p class="r-flow-dash__stat-value">{status.workers.registered.to_string()}</p>
                </div>
                <div class="r-flow-dash__stat">
                    <p class="r-flow-dash__stat-label">"Graphs running"</p>
                    <p class="r-flow-dash__stat-value">{status.graphs.running.to_string()}</p>
                </div>
                <div class="r-flow-dash__stat">
                    <p class="r-flow-dash__stat-label">"Graphs paused"</p>
                    <p class="r-flow-dash__stat-value">{status.graphs.paused.to_string()}</p>
                </div>
                <div class="r-flow-dash__stat">
                    <p class="r-flow-dash__stat-label">"Queue pending"</p>
                    <p class="r-flow-dash__stat-value">{pending.to_string()}</p>
                </div>
                <div class="r-flow-dash__stat">
                    <p class="r-flow-dash__stat-label">"Processing"</p>
                    <p class="r-flow-dash__stat-value">{processing.to_string()}</p>
                </div>
                <div class="r-flow-dash__stat">
                    <p class="r-flow-dash__stat-label">"Scheduler due"</p>
                    <p class="r-flow-dash__stat-value">{status.scheduler.due.to_string()}</p>
                </div>
            </div>
            <section class="r-flow-dash__section">
                <h3>"Workers"</h3>
                <div class="r-flow-dash__chips">{worker_chips}</div>
            </section>
            <section class="r-flow-dash__section">
                <h3>"Queues"</h3>
                <table class="r-flow-dash__table">
                    <thead>
                        <tr>
                            <th>"Name"</th>
                            <th>"Pending"</th>
                            <th>"Active"</th>
                            <th>"Done"</th>
                            <th>"Failed"</th>
                            <th>"Load"</th>
                        </tr>
                    </thead>
                    <tbody>{queue_rows}</tbody>
                </table>
            </section>
            <section class="r-flow-dash__section">
                <h3>"Scheduler"</h3>
                <p class="r-flow-dash__meta">{sched}</p>
            </section>
        </>
    }
}
