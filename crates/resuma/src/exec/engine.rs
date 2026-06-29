//! Flow engine — start workers, hold graph executions, pause/resume hooks.

use std::collections::HashMap;
use std::sync::Arc;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde_json::Value;
use tokio::task;
use tokio_util::sync::CancellationToken;

use crate::core::{Result, ResumaError};

use super::cancel;
use super::durable::{self, ExecutionRecord};
use super::events::{emit, EventBus, SharedEventBus};
use super::graph;
use super::planner::{self, PlannerHints};
use super::resources::{self, ResourceProfile};
use super::runner;
use super::state::StateStore;
use super::types::{GraphId, GraphSnapshot, GraphStatus, StartWorkerResponse};
use super::workers::{self, emit_worker_start, WorkerContext, WorkerFn};

static GRAPHS: Lazy<RwLock<HashMap<String, Arc<GraphExecution>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

static RESUME_LOCKS: Lazy<RwLock<HashMap<String, Arc<tokio::sync::Mutex<()>>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

fn resume_lock(id: &str) -> Arc<tokio::sync::Mutex<()>> {
    let mut map = RESUME_LOCKS.write();
    map.entry(id.to_string())
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone()
}

/// Live graph execution state.
pub struct GraphExecution {
    pub snapshot: Arc<RwLock<GraphSnapshot>>,
    pub bus: SharedEventBus,
    pub state: Arc<StateStore>,
    pub worker: String,
    pub input: Value,
    pub profile: ResourceProfile,
    pub plan: super::types::ExecutionPlan,
    pub cancel: CancellationToken,
}

/// Start a registered worker; returns immediately while execution runs in background.
pub struct FlowEngine;

/// Live graph counts for ops / status.
#[derive(Debug, Clone, Copy, Default)]
pub struct GraphCounts {
    pub active: usize,
    pub running: usize,
    pub paused: usize,
}

impl FlowEngine {
    pub fn graph_counts() -> GraphCounts {
        let graphs = GRAPHS.read();
        let mut running = 0usize;
        let mut paused = 0usize;
        for g in graphs.values() {
            match g.snapshot.read().status {
                GraphStatus::Running => running += 1,
                GraphStatus::Paused => paused += 1,
                _ => {}
            }
        }
        GraphCounts {
            active: graphs.len(),
            running,
            paused,
        }
    }

    pub async fn start(name: &str, input: Value) -> Result<StartWorkerResponse> {
        let (meta, run) =
            workers::get_worker(name).ok_or_else(|| ResumaError::UnknownWorker(name.to_string()))?;

        let hints = PlannerHints {
            use_ai: meta.intent.to_lowercase().contains("ai"),
            tools: Vec::new(),
        };
        let plan = planner::plan(&meta.intent, hints);
        let profile = resources::resolve(&meta.resources, &plan);

        let graph_id = GraphId::new();
        let access_token = super::security::issue_graph_token(&graph_id)?;
        super::metrics::inc_graph_started();
        spawn_execution(
            graph_id.clone(),
            name.to_string(),
            meta.intent.clone(),
            input,
            plan.clone(),
            profile.clone(),
            run,
            None,
            None,
        )
        .await?;

        Ok(StartWorkerResponse {
            graph_id,
            plan,
            access_token: Some(access_token),
        })
    }

    /// Resume a paused graph from durable checkpoint.
    pub async fn resume(id: &GraphId) -> Result<StartWorkerResponse> {
        let lock = resume_lock(&id.0);
        let _guard = lock.lock().await;

        let record = durable::load_execution_record(id)
            .ok_or_else(|| ResumaError::UnknownGraph(id.0.clone()))?;
        if record.cancelled {
            return Err(ResumaError::validation("graph was cancelled"));
        }
        if !record.paused {
            return Err(ResumaError::validation("graph is not paused"));
        }

        if let Some(exec) = GRAPHS.read().get(&id.0) {
            if exec.snapshot.read().status == GraphStatus::Running {
                return Err(ResumaError::validation("graph already running"));
            }
        }

        let (_, run) = workers::get_worker(&record.worker).ok_or_else(|| {
            ResumaError::UnknownWorker(record.worker.clone())
        })?;

        let snapshot = durable::load_graph(id)
            .ok_or_else(|| ResumaError::UnknownGraph(id.0.clone()))?;
        let state = durable::load_checkpoint(id).unwrap_or_default();

        spawn_execution(
            id.clone(),
            record.worker.clone(),
            snapshot.intent.clone(),
            record.input.clone(),
            record.plan.clone(),
            record.profile.clone(),
            run,
            Some(snapshot),
            Some(state),
        )
        .await?;

        let mut record = record;
        record.paused = false;
        let _ = durable::save_execution_record(&record);

        Ok(StartWorkerResponse {
            graph_id: id.clone(),
            plan: record.plan,
            access_token: durable::load_graph_token(id),
        })
    }

    pub fn snapshot(id: &GraphId) -> Option<GraphSnapshot> {
        GRAPHS
            .read()
            .get(&id.0)
            .map(|g| g.snapshot.read().clone())
            .or_else(|| durable::load_graph(id))
    }

    pub fn bus(id: &GraphId) -> Option<SharedEventBus> {
        GRAPHS.read().get(&id.0).map(|g| g.bus.clone())
    }

    pub fn replay(id: &GraphId) -> Option<Vec<super::types::WorkerEvent>> {
        GRAPHS
            .read()
            .get(&id.0)
            .map(|g| g.bus.history())
            .or_else(|| durable::load_events(id))
    }

    /// Pause and **cancel** the in-flight worker (cooperative abort, resumable).
    pub fn pause(id: &GraphId) -> Result<()> {
        let exec = GRAPHS
            .read()
            .get(&id.0)
            .cloned()
            .or_else(|| restore_exec_from_durable(id))
            .ok_or_else(|| ResumaError::UnknownGraph(id.0.clone()))?;

        {
            let snap = exec.snapshot.read();
            match snap.status {
                GraphStatus::Running => {}
                GraphStatus::Paused => return Ok(()),
                GraphStatus::Done | GraphStatus::Failed => {
                    return Err(ResumaError::validation("cannot pause finished graph"));
                }
            }
        }

        // Signal cancellation first so run_on_node / map-reduce stop promptly.
        exec.cancel.cancel();

        {
            let mut snap = exec.snapshot.write();
            snap.status = GraphStatus::Paused;
            let _ = durable::persist_graph(&snap);
        }
        let _ = durable::save_checkpoint(id, &exec.state);
        let _ = durable::persist_events(id, &exec.bus.history());

        let record = ExecutionRecord {
            graph_id: id.clone(),
            worker: exec.worker.clone(),
            input: exec.input.clone(),
            plan: exec.plan.clone(),
            profile: exec.profile.clone(),
            paused: true,
            cancelled: false,
        };
        let _ = durable::save_execution_record(&record);

        GRAPHS.write().insert(id.0.clone(), exec);
        Ok(())
    }

    /// Cancel a graph permanently (not resumable). Running workers are aborted.
    pub fn cancel(id: &GraphId) -> Result<()> {
        let exec = GRAPHS
            .read()
            .get(&id.0)
            .cloned()
            .or_else(|| restore_exec_from_durable(id))
            .ok_or_else(|| ResumaError::UnknownGraph(id.0.clone()))?;

        {
            let snap = exec.snapshot.read();
            match snap.status {
                GraphStatus::Done | GraphStatus::Failed => {
                    return Err(ResumaError::validation("cannot cancel finished graph"));
                }
                GraphStatus::Paused => {
                    let mut snap = exec.snapshot.write();
                    graph::mark_failed(&mut snap);
                    let _ = durable::persist_graph(&snap);
                    let record = ExecutionRecord {
                        graph_id: id.clone(),
                        worker: exec.worker.clone(),
                        input: exec.input.clone(),
                        plan: exec.plan.clone(),
                        profile: exec.profile.clone(),
                        paused: false,
                        cancelled: true,
                    };
                    let _ = durable::save_execution_record(&record);
                    super::metrics::inc_graph_failed();
                    super::webhooks::notify_failed(
                        &snap,
                        0,
                        "cancelled by operator".into(),
                    );
                    return Ok(());
                }
                GraphStatus::Running => {}
            }
        }

        exec.cancel.cancel();

        let record = ExecutionRecord {
            graph_id: id.clone(),
            worker: exec.worker.clone(),
            input: exec.input.clone(),
            plan: exec.plan.clone(),
            profile: exec.profile.clone(),
            paused: false,
            cancelled: true,
        };
        let _ = durable::save_execution_record(&record);

        GRAPHS.write().insert(id.0.clone(), exec);
        Ok(())
    }
}

async fn spawn_execution(
    graph_id: GraphId,
    worker_name: String,
    intent: String,
    input: Value,
    plan: super::types::ExecutionPlan,
    profile: ResourceProfile,
    run: WorkerFn,
    existing_snapshot: Option<GraphSnapshot>,
    existing_state: Option<StateStore>,
) -> Result<()> {
    let snapshot = existing_snapshot.unwrap_or_else(|| {
        graph::materialize(graph_id.clone(), &worker_name, &intent, &plan)
    });
    let bus = Arc::new(EventBus::new());

    if let Some(events) = durable::load_events(&graph_id) {
        for event in events {
            bus.emit(event);
        }
    }

    let state = Arc::new(existing_state.unwrap_or_default());
    let snap_arc = Arc::new(RwLock::new(snapshot));
    let cancel = cancel::new_scope();

    let exec = Arc::new(GraphExecution {
        snapshot: snap_arc.clone(),
        bus: bus.clone(),
        state: state.clone(),
        worker: worker_name.clone(),
        input: input.clone(),
        profile: profile.clone(),
        plan: plan.clone(),
        cancel: cancel.clone(),
    });

    GRAPHS.write().insert(graph_id.0.clone(), exec);

    let record = ExecutionRecord {
        graph_id: graph_id.clone(),
        worker: worker_name.clone(),
        input: input.clone(),
        plan: plan.clone(),
        profile: profile.clone(),
        paused: false,
        cancelled: false,
    };
    let _ = durable::save_execution_record(&record);

    let gid = graph_id.clone();
    task::spawn(async move {
        run_worker(
            gid,
            worker_name,
            input,
            run,
            bus,
            state,
            snap_arc,
            profile,
            plan,
            cancel,
        )
        .await;
    });

    Ok(())
}

fn restore_exec_from_durable(id: &GraphId) -> Option<Arc<GraphExecution>> {
    let record = durable::load_execution_record(id)?;
    let snapshot = durable::load_graph(id)?;
    let state = durable::load_checkpoint(id).unwrap_or_default();
    let bus = Arc::new(EventBus::new());
    if let Some(events) = durable::load_events(id) {
        for event in events {
            bus.emit(event);
        }
    }
    Some(Arc::new(GraphExecution {
        snapshot: Arc::new(RwLock::new(snapshot)),
        bus,
        state: Arc::new(state),
        worker: record.worker,
        input: record.input,
        profile: record.profile,
        plan: record.plan,
        cancel: cancel::new_scope(),
    }))
}

async fn run_worker(
    graph_id: GraphId,
    _name: String,
    input: Value,
    run: WorkerFn,
    bus: SharedEventBus,
    state: Arc<StateStore>,
    snapshot: Arc<RwLock<GraphSnapshot>>,
    profile: ResourceProfile,
    plan: super::types::ExecutionPlan,
    cancel: CancellationToken,
) {
    {
        let mut snap = snapshot.write();
        graph::mark_running(&mut snap);
        let _ = durable::persist_graph(&snap);
    }

    let ctx = WorkerContext::new(
        graph_id.clone(),
        bus.clone(),
        state.clone(),
        snapshot.clone(),
        cancel.clone(),
    );
    emit_worker_start(&ctx);
    ctx.log("execution started");

    let started = super::id::now_ms();
    let result = runner::run_with_plan(
        &plan,
        input,
        run,
        graph_id.clone(),
        bus.clone(),
        state.clone(),
        snapshot.clone(),
        profile,
        cancel.clone(),
    )
    .await;

    match result {
        Ok(value) => {
            bus.emit(emit::result(value.clone()));
            let duration = super::id::now_ms().saturating_sub(started);
            bus.emit(emit::node_done(ctx.node_id.clone(), duration));
            let mut snap = snapshot.write();
            graph::mark_done(&mut snap);
            let _ = durable::persist_graph(&snap);
            let _ = durable::persist_events(&graph_id, &bus.history());
            let final_snap = snap.clone();
            drop(snap);
            super::metrics::inc_graph_completed();
            super::webhooks::notify_done(&final_snap, duration, Some(value));
            bus.emit(emit::graph_done(graph_id));
        }
        Err(ResumaError::Cancelled) => {
            let duration = super::id::now_ms().saturating_sub(started);
            let hard_cancel = durable::load_execution_record(&graph_id)
                .map(|r| r.cancelled)
                .unwrap_or(false);
            if hard_cancel {
                ctx.log("execution cancelled");
                let mut snap = snapshot.write();
                graph::mark_failed(&mut snap);
                let _ = durable::persist_graph(&snap);
                let _ = durable::persist_events(&graph_id, &bus.history());
                let final_snap = snap.clone();
                drop(snap);
                super::metrics::inc_graph_failed();
                super::webhooks::notify_failed(
                    &final_snap,
                    duration,
                    "cancelled by operator".into(),
                );
                bus.emit(emit::graph_done(graph_id));
            } else {
                ctx.log("execution paused (cancelled)");
                let mut snap = snapshot.write();
                if snap.status != GraphStatus::Paused {
                    snap.status = GraphStatus::Paused;
                }
                let _ = durable::persist_graph(&snap);
                let _ = durable::persist_events(&graph_id, &bus.history());
                let final_snap = snap.clone();
                drop(snap);
                super::metrics::inc_graph_paused();
                super::webhooks::notify_paused(&final_snap, duration);
            }
        }
        Err(e) => {
            bus.emit(emit::node_failed(ctx.node_id.clone(), e.to_string()));
            let duration = super::id::now_ms().saturating_sub(started);
            let mut snap = snapshot.write();
            graph::mark_failed(&mut snap);
            let _ = durable::persist_graph(&snap);
            let _ = durable::persist_events(&graph_id, &bus.history());
            let final_snap = snap.clone();
            let err = e.to_string();
            drop(snap);
            super::metrics::inc_graph_failed();
            super::webhooks::notify_failed(&final_snap, duration, err);
            bus.emit(emit::graph_done(graph_id));
        }
    }
}
