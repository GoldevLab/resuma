//! Worker registry + execution context (`ctx.emit`, `ctx.tool`, `ctx.ai`, `ctx.state`).

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

use crate::core::Result;

use super::durable;
use super::events::{emit, SharedEventBus};
use super::graph;
use super::resources::Resources;
use super::state::StateStore;
use super::tools;
use super::types::{GraphId, NodeId, NodeKind};

pub type WorkerFuture = Pin<Box<dyn Future<Output = Result<Value>> + Send>>;
pub type WorkerFn = fn(Value, WorkerContext) -> WorkerFuture;

#[derive(Debug, Clone)]
pub struct WorkerMeta {
    pub intent: String,
    pub resources: Resources,
}

struct WorkerEntry {
    meta: WorkerMeta,
    run: WorkerFn,
}

static WORKERS: Lazy<RwLock<HashMap<String, WorkerEntry>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Register a named worker handler.
pub fn register_worker(name: &str, meta: WorkerMeta, run: WorkerFn) {
    WORKERS
        .write()
        .insert(name.to_string(), WorkerEntry { meta, run });
}

pub fn get_worker(name: &str) -> Option<(WorkerMeta, WorkerFn)> {
    WORKERS.read().get(name).map(|e| (e.meta.clone(), e.run))
}

/// Names of all registered workers (for ops / status).
pub fn list_worker_names() -> Vec<String> {
    let mut names: Vec<_> = WORKERS.read().keys().cloned().collect();
    names.sort();
    names
}

/// True when at least one worker handler was registered.
pub fn has_registered_workers() -> bool {
    !WORKERS.read().is_empty()
}

/// Per-invocation context passed to worker `run` closures.
#[derive(Clone)]
pub struct WorkerContext {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    bus: SharedEventBus,
    state: Arc<StateStore>,
    snapshot: Arc<RwLock<super::types::GraphSnapshot>>,
    cancel: CancellationToken,
}

impl WorkerContext {
    pub(crate) fn new(
        graph_id: GraphId,
        bus: SharedEventBus,
        state: Arc<StateStore>,
        snapshot: Arc<RwLock<super::types::GraphSnapshot>>,
        cancel: CancellationToken,
    ) -> Self {
        Self {
            graph_id,
            node_id: NodeId::new("worker"),
            bus,
            state,
            snapshot,
            cancel,
        }
    }

    /// Clone the cancellation token (e.g. for custom async branches).
    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel.clone()
    }

    /// Returns `Err(Cancelled)` when execution was paused.
    pub fn check_cancelled(&self) -> Result<()> {
        super::cancel::check(&self.cancel)
    }

    pub fn emit(&self, event: super::types::WorkerEvent) {
        graph::apply_event(&mut self.snapshot.write(), &event);
        self.bus.emit(event);
    }

    pub fn log(&self, message: impl Into<String>) {
        self.emit(emit::log(self.node_id.clone(), message));
    }

    pub fn progress(&self, value: u8) {
        self.emit(emit::progress(self.node_id.clone(), value));
    }

    pub async fn tool(&self, name: &str, args: Value) -> Result<Value> {
        self.check_cancelled()?;
        self.emit(emit::tool_call(NodeId::new(name), name, Some(args.clone())));
        let started = super::id::now_ms();
        let out = super::cancel::run_cancellable(&self.cancel, tools::dispatch(name, args)).await;
        let duration = super::id::now_ms().saturating_sub(started);
        match &out {
            Ok(_) => self.emit(emit::node_done(NodeId::new(name), duration)),
            Err(e) => self.emit(emit::node_failed(NodeId::new(name), e.to_string())),
        }
        out
    }

    pub async fn ai(&self, prompt: impl Into<String>, data: &Value) -> Result<Value> {
        let prompt = prompt.into();
        self.emit(emit::ai_thinking(self.node_id.clone(), &prompt));
        self.tool("ai", json!({ "prompt": prompt, "data": data }))
            .await
    }

    pub fn state(&self) -> &StateStore {
        &self.state
    }

    pub fn state_get(&self, key: &str) -> Option<Value> {
        self.state.get(key)
    }

    pub fn state_set(&self, key: impl Into<String>, value: Value) {
        self.state.set(key, value);
        let _ = durable::save_checkpoint(&self.graph_id, &self.state);
    }
}

/// Builder for registering workers fluently from app code.
#[derive(Debug, Default)]
pub struct WorkerRegistry {
    entries: Vec<(String, WorkerMeta, WorkerFn)>,
}

impl WorkerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(mut self, name: impl Into<String>, meta: WorkerMeta, run: WorkerFn) -> Self {
        self.entries.push((name.into(), meta, run));
        self
    }

    pub fn install(self) {
        for (name, meta, run) in self.entries {
            register_worker(&name, meta, run);
        }
    }
}

/// Convenience for worker node start event.
pub fn emit_worker_start(ctx: &WorkerContext) {
    ctx.emit(emit::node_start(ctx.node_id.clone(), NodeKind::Worker));
}
