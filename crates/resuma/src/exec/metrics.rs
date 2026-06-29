//! Prometheus metrics — counters and gauges for the execution layer.

use std::sync::atomic::{AtomicU64, Ordering};

use super::engine::FlowEngine;
use super::status;

static GRAPHS_STARTED: AtomicU64 = AtomicU64::new(0);
static GRAPHS_COMPLETED: AtomicU64 = AtomicU64::new(0);
static GRAPHS_FAILED: AtomicU64 = AtomicU64::new(0);
static GRAPHS_PAUSED: AtomicU64 = AtomicU64::new(0);
static WEBHOOKS_SENT: AtomicU64 = AtomicU64::new(0);
static WEBHOOKS_FAILED: AtomicU64 = AtomicU64::new(0);

/// Record a graph execution start.
pub fn inc_graph_started() {
    GRAPHS_STARTED.fetch_add(1, Ordering::Relaxed);
}

/// Record successful graph completion.
pub fn inc_graph_completed() {
    GRAPHS_COMPLETED.fetch_add(1, Ordering::Relaxed);
}

/// Record failed graph execution.
pub fn inc_graph_failed() {
    GRAPHS_FAILED.fetch_add(1, Ordering::Relaxed);
}

/// Record paused / cancelled graph.
pub fn inc_graph_paused() {
    GRAPHS_PAUSED.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_webhook_sent() {
    WEBHOOKS_SENT.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_webhook_failed() {
    WEBHOOKS_FAILED.fetch_add(1, Ordering::Relaxed);
}

fn load(counter: &AtomicU64) -> u64 {
    counter.load(Ordering::Relaxed)
}

/// Render Prometheus text exposition format (`GET /_resuma/metrics`).
pub fn prometheus_text() -> String {
    let snap = status::snapshot();
    let mut out = String::with_capacity(2048);

    writeln_metric(
        &mut out,
        "resuma_exec_up",
        "gauge",
        "1",
        &[],
        "Resuma execution layer is running",
    );
    writeln_metric(
        &mut out,
        "resuma_exec_uptime_seconds",
        "gauge",
        &(snap.uptime_ms / 1000).to_string(),
        &[],
        "Process uptime since exec init",
    );
    writeln_metric(
        &mut out,
        "resuma_exec_workers_registered",
        "gauge",
        &snap.workers.registered.to_string(),
        &[],
        "Registered worker count",
    );
    writeln_metric(
        &mut out,
        "resuma_exec_graphs_active",
        "gauge",
        &snap.graphs.running.to_string(),
        &[("status", "running")],
        "Live graphs by status",
    );
    writeln_metric(
        &mut out,
        "resuma_exec_graphs_active",
        "gauge",
        &snap.graphs.paused.to_string(),
        &[("status", "paused")],
        "Live graphs by status",
    );
    writeln_metric(
        &mut out,
        "resuma_exec_graphs_active",
        "gauge",
        &snap.graphs.active.to_string(),
        &[("status", "total")],
        "Live graphs by status",
    );

    let counts = FlowEngine::graph_counts();
    let _ = counts;

    writeln_metric(
        &mut out,
        "resuma_exec_graphs_total",
        "counter",
        &load(&GRAPHS_STARTED).to_string(),
        &[("status", "started")],
        "Graph executions started",
    );
    writeln_metric(
        &mut out,
        "resuma_exec_graphs_total",
        "counter",
        &load(&GRAPHS_COMPLETED).to_string(),
        &[("status", "completed")],
        "Graph executions completed",
    );
    writeln_metric(
        &mut out,
        "resuma_exec_graphs_total",
        "counter",
        &load(&GRAPHS_FAILED).to_string(),
        &[("status", "failed")],
        "Graph executions failed",
    );
    writeln_metric(
        &mut out,
        "resuma_exec_graphs_total",
        "counter",
        &load(&GRAPHS_PAUSED).to_string(),
        &[("status", "paused")],
        "Graph executions paused",
    );

    for q in &snap.queues {
        for (state, value) in [
            ("pending", q.pending),
            ("processing", q.processing),
            ("done", q.done),
            ("failed", q.failed),
        ] {
            writeln_metric(
                &mut out,
                "resuma_exec_queue_jobs",
                "gauge",
                &value.to_string(),
                &[("queue", &q.queue), ("state", state)],
                "Queue depth by state",
            );
        }
    }

    writeln_metric(
        &mut out,
        "resuma_exec_scheduler_jobs",
        "gauge",
        &snap.scheduler.total.to_string(),
        &[("state", "total")],
        "Scheduled jobs",
    );
    writeln_metric(
        &mut out,
        "resuma_exec_scheduler_jobs",
        "gauge",
        &snap.scheduler.enabled.to_string(),
        &[("state", "enabled")],
        "Scheduled jobs",
    );
    writeln_metric(
        &mut out,
        "resuma_exec_scheduler_jobs",
        "gauge",
        &snap.scheduler.due.to_string(),
        &[("state", "due")],
        "Scheduled jobs due now",
    );

    writeln_metric(
        &mut out,
        "resuma_exec_webhooks_total",
        "counter",
        &load(&WEBHOOKS_SENT).to_string(),
        &[("result", "success")],
        "Webhook deliveries",
    );
    writeln_metric(
        &mut out,
        "resuma_exec_webhooks_total",
        "counter",
        &load(&WEBHOOKS_FAILED).to_string(),
        &[("result", "failed")],
        "Webhook deliveries",
    );

    out
}

fn writeln_metric(
    out: &mut String,
    name: &str,
    kind: &str,
    value: &str,
    labels: &[(&str, &str)],
    help: &str,
) {
    use std::fmt::Write;
    let _ = writeln!(out, "# HELP {name} {help}");
    let _ = writeln!(out, "# TYPE {name} {kind}");
    if labels.is_empty() {
        let _ = writeln!(out, "{name} {value}");
    } else {
        let label_str: String = labels
            .iter()
            .map(|(k, v)| format!("{k}=\"{}\"", escape_label(v)))
            .collect::<Vec<_>>()
            .join(",");
        let _ = writeln!(out, "{name}{{{label_str}}} {value}");
    }
}

fn escape_label(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prometheus_contains_help_and_type() {
        let text = prometheus_text();
        assert!(text.contains("# HELP resuma_exec_up"));
        assert!(text.contains("# TYPE resuma_exec_graphs_total counter"));
        assert!(text.contains("resuma_exec_workers_registered"));
    }
}
