//! Resuma Node — isolated execution with timeout + cancellation.

use std::time::Duration;

use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

use crate::core::{Result, ResumaError};

use super::cancel;
use super::resources::ResourceProfile;
use super::workers::{WorkerContext, WorkerFn};
use serde_json::Value;

/// Run a worker on this node with timeout and pause/cancel support.
///
/// `timeout_secs == 0` disables the wall-clock timeout (cooperative cancel only).
pub async fn run_on_node(
    profile: &ResourceProfile,
    input: Value,
    ctx: WorkerContext,
    run: WorkerFn,
    cancel: &CancellationToken,
) -> Result<Value> {
    let work = cancel::run_cancellable(cancel, run(input, ctx));
    if profile.timeout_secs == 0 {
        return work.await;
    }
    let secs = profile.timeout_secs;
    match timeout(Duration::from_secs(secs), work).await {
        Ok(r) => r,
        Err(_) => {
            // The timed-out future is dropped, but any tasks it spawned may
            // still be running — signal the scope so they stop cooperatively.
            cancel.cancel();
            Err(ResumaError::Other(format!(
                "worker exceeded timeout ({}s)",
                secs
            )))
        }
    }
}

/// In-process node pool size hint (parallel worker executions).
#[derive(Debug, Clone)]
pub struct NodePool {
    pub parallel_limit: u32,
}

impl Default for NodePool {
    fn default() -> Self {
        Self { parallel_limit: 4 }
    }
}

impl From<&ResourceProfile> for NodePool {
    fn from(p: &ResourceProfile) -> Self {
        Self {
            parallel_limit: p.parallel_limit.max(1),
        }
    }
}

/// Shared node pool config (process-wide).
static POOL: once_cell::sync::Lazy<std::sync::Arc<parking_lot::RwLock<NodePool>>> =
    once_cell::sync::Lazy::new(|| {
        std::sync::Arc::new(parking_lot::RwLock::new(NodePool::default()))
    });

pub fn configure_pool(pool: NodePool) {
    *POOL.write() = pool;
}

pub fn pool() -> NodePool {
    POOL.read().clone()
}
