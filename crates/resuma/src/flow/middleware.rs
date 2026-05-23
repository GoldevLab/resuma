//! Global middleware pipeline for Resuma Flow requests.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use crate::core::Result;

use super::request::FlowRequest;

pub type MiddlewareFuture = Pin<Box<dyn Future<Output = Result<FlowRequest>> + Send>>;
pub type MiddlewareFn = fn(FlowRequest) -> MiddlewareFuture;

static ORDER: AtomicUsize = AtomicUsize::new(0);
static MIDDLEWARE: Lazy<RwLock<Vec<(usize, MiddlewareFn)>>> =
    Lazy::new(|| RwLock::new(Vec::new()));

/// Register a middleware handler. Order follows registration (via `#[ctor]` link order).
pub fn register_middleware(f: MiddlewareFn) {
    let order = ORDER.fetch_add(1, Ordering::SeqCst);
    MIDDLEWARE.write().push((order, f));
    MIDDLEWARE.write().sort_by_key(|(o, _)| *o);
}

/// Run all registered middleware against a request.
pub async fn run_middleware(req: FlowRequest) -> Result<FlowRequest> {
    let chain: Vec<MiddlewareFn> = MIDDLEWARE.read().iter().map(|(_, f)| *f).collect();
    let mut current = req;
    for mw in chain {
        current = mw(current).await?;
    }
    Ok(current)
}
