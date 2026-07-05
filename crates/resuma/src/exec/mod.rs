//! Resuma Execution Layer — planner, resources, graph runtime, queue, durable storage.
//!
//! Resuma OS runs entirely on your infrastructure:
//!
//! * **Backend** — Tokio + Axum (`resuma dev`, Fly.io, Docker)
//! * **Node** — isolated worker tasks with timeouts (`exec::node`)
//! * **Queue** — async job dispatch (`POST /_resuma/queue/:name`)
//! * **Durable** — `.resuma/durable/` KV + checkpoints
//! * **Tools** — real HTTP fetch + OpenAI-compatible AI proxy
//! * **Browser** — live graph UI via `resuma-flow` + SSE

pub mod actions;
pub mod cancel;
pub mod config;
pub mod cron;
pub mod durable;
pub mod engine;
pub mod events;
pub mod graph;
pub mod id;
pub mod metrics;
pub mod node;
pub mod planner;
pub mod queue;
pub mod queue_disk;
pub mod resources;
pub mod routes;
pub mod runner;
pub mod runtime;
pub mod scheduler;
pub mod security;
pub mod ssrf;
pub mod state;
pub mod status;
pub mod tools;
pub mod webhooks;
pub mod workers;

#[cfg(test)]
mod tests;

pub mod types;

pub use config::init as init_exec;
pub use engine::{FlowEngine, GraphCounts};
pub use events::EventBus;
pub use metrics::prometheus_text;
pub use node::{configure_pool as configure_node_pool, NodePool};
pub use planner::{plan, PlannerHints};
pub use queue::{enqueue, queue_stats, register_queue, EnqueueResponse, QueueMessage};
pub use queue_disk::QueueStats;
pub use resources::{resolve as resolve_resources, ResourceLevel, ResourceProfile, Resources};
pub use routes::attach_exec_routes;
pub use runtime::{route as route_runtime, RuntimeTarget};
pub use scheduler::{
    create as create_schedule, list_response as list_schedules, remove as remove_schedule,
    CreateScheduleBody, ScheduleJob, ScheduleListResponse, SchedulerStats,
};
pub use security::{configure as configure_exec_security, ExecSecurityConfig, GRAPH_TOKEN_HEADER};
pub use state::StateStore;
pub use status::{snapshot as exec_status, ExecStatus, GraphsStatus, WorkersStatus};
pub use tools::{dispatch as dispatch_tool, register_tool};
pub use types::{
    ExecutionPlan, ExecutionStrategy, GraphEdge, GraphId, GraphNodeSnapshot, GraphSnapshot,
    GraphStatus, NodeId, NodeKind, NodeStatus, RuntimeChoice, StartWorkerResponse, WorkerEvent,
};
pub use webhooks::{register as register_webhook, RegisterWebhookBody, WebhookListResponse};
pub use workers::{
    has_registered_workers, register_worker, WorkerContext, WorkerMeta, WorkerRegistry,
};

/// True when `/_resuma/*` exec admin routes should be mounted on the HTTP router.
///
/// Routes are omitted for purely static apps unless workers are registered or
/// `RESUMA_EXEC_ENABLED=1` is set explicitly.
pub fn exec_routes_enabled() -> bool {
    workers::has_registered_workers()
        || matches!(
            std::env::var("RESUMA_EXEC_ENABLED").as_deref(),
            Ok("1") | Ok("true") | Ok("TRUE")
        )
}

pub use durable::{
    get as durable_get, load_checkpoint, load_events, load_execution_record, load_graph,
    persist_events, persist_graph, save_checkpoint, save_execution_record, set as durable_set,
    ExecutionRecord,
};
