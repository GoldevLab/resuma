//! Global registry for `#[server]` actions.

use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::pin::Pin;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use resuma_core::{FlowRequest, ResumaError, Result};
use serde_json::Value;

pub type ActionFuture = Pin<Box<dyn Future<Output = Result<Value>> + Send>>;
pub type ActionFn = fn(Vec<Value>, FlowRequest) -> ActionFuture;

pub type ActionMiddlewareFuture = Pin<Box<dyn Future<Output = Result<FlowRequest>> + Send>>;
pub type ActionMiddlewareFn = fn(FlowRequest) -> ActionMiddlewareFuture;

static REGISTRY: Lazy<RwLock<HashMap<String, ActionFn>>> = Lazy::new(|| RwLock::new(HashMap::new()));
static ACTION_MIDDLEWARE: Lazy<RwLock<Option<ActionMiddlewareFn>>> =
    Lazy::new(|| RwLock::new(None));

/// Register a global middleware pipeline for `/_resuma/action/*` requests.
pub fn set_action_middleware(f: ActionMiddlewareFn) {
    *ACTION_MIDDLEWARE.write() = Some(f);
}

pub fn register_server_action(name: &str, f: ActionFn) {
    REGISTRY.write().insert(name.to_string(), f);
}

pub fn get_action(name: &str) -> Option<ActionFn> {
    REGISTRY.read().get(name).copied()
}

pub async fn dispatch(name: &str, args: Vec<Value>, mut req: FlowRequest) -> Result<Value> {
    let middleware = *ACTION_MIDDLEWARE.read();
    if let Some(mw) = middleware {
        req = mw(req).await?;
    }
    match get_action(name) {
        Some(f) => f(args, req).await,
        None => Err(ResumaError::UnknownAction(name.to_string())),
    }
}

/// Parse query string into a map.
pub fn parse_query(query: &str) -> BTreeMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.to_string();
            let value = parts.next().unwrap_or("").to_string();
            if key.is_empty() {
                None
            } else {
                Some((key, value))
            }
        })
        .collect()
}
