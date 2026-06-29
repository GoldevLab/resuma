//! Resuma OS bootstrap — configure durable storage, queues, node pool from env.

use super::durable;
use super::node::{self, NodePool};
use super::queue;
use super::scheduler;
use super::webhooks;

/// Initialize the execution layer (call once before serving).
pub async fn init() {
    init_storage();
    init_queue();
    init_scheduler();
    init_webhooks();
    init_node_pool();
    init_security();
    super::actions::register_builtin_actions();
    queue::start_consumers().await;
    scheduler::start().await;
    super::tools::init_http_client();
}

fn init_security() {
    super::security::configure(super::security::ExecSecurityConfig::from_env());
    let root = data_root();
    crate::server::rate_limit_disk::configure(format!("{root}/rate-limit"));
}

fn data_root() -> String {
    std::env::var("RESUMA_DATA_DIR").unwrap_or_else(|_| ".resuma".into())
}

fn init_storage() {
    durable::configure(format!("{}/durable", data_root()));
}

fn init_queue() {
    queue::configure_disk(format!("{}/queue", data_root()));
}

fn init_scheduler() {
    scheduler::configure(format!("{}/scheduler", data_root()));
}

fn init_webhooks() {
    webhooks::configure(format!("{}/webhooks", data_root()));
    webhooks::init_from_env();
}

fn init_node_pool() {
    let parallel = std::env::var("RESUMA_NODE_PARALLEL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);
    node::configure_pool(NodePool {
        parallel_limit: parallel,
    });
}
