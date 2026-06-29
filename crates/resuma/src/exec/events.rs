//! In-process event bus with replay log (SSE source of truth).

use std::sync::Arc;

use parking_lot::RwLock;
use tokio::sync::broadcast;

use super::types::WorkerEvent;

const BUS_CAPACITY: usize = 256;

/// Per-graph event bus: append-only log + live subscribers.
#[derive(Debug)]
pub struct EventBus {
    tx: broadcast::Sender<WorkerEvent>,
    log: RwLock<Vec<WorkerEvent>>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(BUS_CAPACITY);
        Self {
            tx,
            log: RwLock::new(Vec::new()),
        }
    }

    pub fn emit(&self, event: WorkerEvent) {
        self.log.write().push(event.clone());
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WorkerEvent> {
        self.tx.subscribe()
    }

    pub fn history(&self) -> Vec<WorkerEvent> {
        self.log.read().clone()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Build events with current timestamp.
pub mod emit {
    use serde_json::Value;

    use super::super::id;
    use super::super::types::{GraphId, NodeId, NodeKind, WorkerEvent};

    pub fn log(node: NodeId, message: impl Into<String>) -> WorkerEvent {
        WorkerEvent::Log {
            message: message.into(),
            node,
            timestamp_ms: id::now_ms(),
        }
    }

    pub fn progress(node: NodeId, value: u8) -> WorkerEvent {
        WorkerEvent::Progress {
            value: value.min(100),
            node,
            timestamp_ms: id::now_ms(),
        }
    }

    pub fn ai_thinking(node: NodeId, content: impl Into<String>) -> WorkerEvent {
        WorkerEvent::AiThinking {
            content: content.into(),
            node,
            timestamp_ms: id::now_ms(),
        }
    }

    pub fn tool_call(node: NodeId, tool: impl Into<String>, args: Option<Value>) -> WorkerEvent {
        WorkerEvent::ToolCall {
            tool: tool.into(),
            args,
            node,
            timestamp_ms: id::now_ms(),
        }
    }

    pub fn node_start(node: NodeId, kind: NodeKind) -> WorkerEvent {
        WorkerEvent::NodeStart {
            node,
            kind,
            timestamp_ms: id::now_ms(),
        }
    }

    pub fn node_done(node: NodeId, duration_ms: u64) -> WorkerEvent {
        WorkerEvent::NodeDone {
            node,
            duration_ms,
            timestamp_ms: id::now_ms(),
        }
    }

    pub fn node_failed(node: NodeId, error: impl Into<String>) -> WorkerEvent {
        WorkerEvent::NodeFailed {
            node,
            error: error.into(),
            timestamp_ms: id::now_ms(),
        }
    }

    pub fn result(data: Value) -> WorkerEvent {
        WorkerEvent::Result {
            data,
            timestamp_ms: id::now_ms(),
        }
    }

    pub fn graph_done(graph_id: GraphId) -> WorkerEvent {
        WorkerEvent::GraphDone {
            graph_id,
            timestamp_ms: id::now_ms(),
        }
    }
}

pub type SharedEventBus = Arc<EventBus>;
