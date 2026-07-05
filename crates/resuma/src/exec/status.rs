//! Execution layer ops snapshot — queues, graphs, workers, scheduler.

use serde::Serialize;

use super::engine::FlowEngine;
use super::queue_disk;
use super::scheduler;
use super::workers;

/// `GET /_resuma/status` — operational snapshot for monitoring / readiness.
#[derive(Debug, Clone, Serialize)]
pub struct ExecStatus {
    pub ok: bool,
    pub uptime_ms: u64,
    pub workers: WorkersStatus,
    pub graphs: GraphsStatus,
    pub queues: Vec<queue_disk::QueueStats>,
    pub scheduler: scheduler::SchedulerStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkersStatus {
    pub registered: usize,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphsStatus {
    pub active: usize,
    pub running: usize,
    pub paused: usize,
}

static STARTED_MS: once_cell::sync::Lazy<u64> = once_cell::sync::Lazy::new(super::id::now_ms);

/// Build the current execution-layer status snapshot.
pub fn snapshot() -> ExecStatus {
    let graph_counts = FlowEngine::graph_counts();
    let queue_names = queue_disk::list_queues();
    let mut queues: Vec<_> = queue_names.iter().map(|q| queue_disk::stats(q)).collect();
    if queues.is_empty() {
        queues.push(queue_disk::stats("default"));
    }
    let names = workers::list_worker_names();
    ExecStatus {
        ok: true,
        uptime_ms: super::id::now_ms().saturating_sub(*STARTED_MS),
        workers: WorkersStatus {
            registered: names.len(),
            names,
        },
        graphs: GraphsStatus {
            active: graph_counts.active,
            running: graph_counts.running,
            paused: graph_counts.paused,
        },
        queues,
        scheduler: scheduler::stats(),
    }
}

/// True when the execution layer is healthy enough for traffic.
pub fn is_ready() -> bool {
    snapshot().ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_includes_queues() {
        let snap = snapshot();
        assert!(snap.ok);
        assert!(!snap.queues.is_empty());
    }
}
