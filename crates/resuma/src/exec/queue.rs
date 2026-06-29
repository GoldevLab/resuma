//! Job queue — disk-backed, multi-process safe (atomic file claim).

use std::collections::HashMap;
use std::time::Duration;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{mpsc, Mutex};
use tokio::task;
use tracing::{error, info};

use crate::core::Result;

use super::engine::FlowEngine;
use super::id;
use super::queue_disk;
use super::types::{GraphId, GraphStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMessage {
    pub id: String,
    pub worker: String,
    #[serde(default)]
    pub input: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnqueueResponse {
    pub message_id: String,
    pub queue: String,
}

static WAKES: Lazy<RwLock<HashMap<String, mpsc::Sender<()>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

static STARTED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

fn poll_interval_ms() -> u64 {
    std::env::var("RESUMA_QUEUE_POLL_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200)
}

/// Configure file-backed queue storage root.
pub fn configure_disk(root: impl AsRef<std::path::Path>) {
    queue_disk::configure(root);
}

/// Start disk pollers for all queues (multi-process workers).
pub async fn start_consumers() {
    let mut started = STARTED.lock().await;
    if *started {
        return;
    }
    *started = true;

    register_queue("default");

    for name in queue_disk::list_queues() {
        if name != "default" {
            register_queue(&name);
        }
    }
}

/// Register a named queue and spawn its disk consumer loop.
pub fn register_queue(name: &str) {
    if WAKES.read().contains_key(name) {
        return;
    }
    let (tx, rx) = mpsc::channel(512);
    WAKES.write().insert(name.to_string(), tx);
    let qname = name.to_string();
    task::spawn(async move {
        disk_consumer_loop(&qname, rx).await;
    });
}

/// Enqueue a worker job (persisted to disk; any Resuma process may claim it).
pub async fn enqueue(queue: &str, worker: &str, input: Value) -> Result<EnqueueResponse> {
    register_queue(queue);
    let msg = QueueMessage {
        id: format!("m_{}", id::next_id()),
        worker: worker.to_string(),
        input,
    };
    let mid = msg.id.clone();
    queue_disk::persist_pending(queue, &msg)?;
    wake(queue);
    Ok(EnqueueResponse {
        message_id: mid,
        queue: queue.to_string(),
    })
}

/// Queue depth stats (`pending`, `processing`, …).
pub fn queue_stats(queue: &str) -> queue_disk::QueueStats {
    queue_disk::stats(queue)
}

fn wake(queue: &str) {
    if let Some(tx) = WAKES.read().get(queue) {
        let _ = tx.try_send(());
    }
}

async fn disk_consumer_loop(queue: &str, mut wake_rx: mpsc::Receiver<()>) {
    let recovered = queue_disk::recover_processing(queue);
    if recovered > 0 {
        info!(queue = %queue, recovered, "re-queued stuck processing jobs");
    }
    info!(queue = %queue, "resuma disk queue consumer started (multi-process)");

    let poll = Duration::from_millis(poll_interval_ms());
    loop {
        drain_claimed(queue).await;
        tokio::select! {
            _ = wake_rx.recv() => {}
            _ = tokio::time::sleep(poll) => {}
        }
    }
}

async fn drain_claimed(queue: &str) {
    while let Some(msg) = queue_disk::claim_next(queue) {
        process_message(queue, msg).await;
    }
}

async fn process_message(queue: &str, msg: QueueMessage) {
    info!(
        queue = %queue,
        worker = %msg.worker,
        message_id = %msg.id,
        "claimed queued job from disk"
    );
    match FlowEngine::start(&msg.worker, msg.input.clone()).await {
        Ok(started) => {
            info!(
                graph_id = %started.graph_id.0,
                worker = %msg.worker,
                "queued worker started"
            );
            let queue = queue.to_string();
            let msg_id = msg.id.clone();
            let graph_id = started.graph_id;
            tokio::spawn(async move {
                let success = wait_graph_terminal(&graph_id).await;
                let _ = queue_disk::complete(&queue, &msg_id, success);
            });
        }
        Err(e) => {
            error!(error = %e, worker = %msg.worker, "queued worker failed to start");
            let _ = queue_disk::complete(queue, &msg.id, false);
        }
    }
}

async fn wait_graph_terminal(graph_id: &GraphId) -> bool {
    let poll = Duration::from_millis(100);
    loop {
        match FlowEngine::snapshot(graph_id) {
            Some(snap) => match snap.status {
                GraphStatus::Done => return true,
                GraphStatus::Failed => return false,
                GraphStatus::Running | GraphStatus::Paused => {}
            },
            None => return false,
        }
        tokio::time::sleep(poll).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn enqueue_persists_for_disk_consumer() {
        let _guard = super::queue_disk::test_queue_lock().lock();
        let root = std::env::temp_dir().join(format!("resuma-qe-{}", id::next_id()));
        configure_disk(&root);
        register_queue("test");
        let resp = enqueue("test", "missing_worker", json!({}))
            .await
            .expect("enqueue");
        let s = queue_stats("test");
        assert!(s.pending >= 1 || s.processing >= 1);
        assert!(!resp.message_id.is_empty());
    }
}
