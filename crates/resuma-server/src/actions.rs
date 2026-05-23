//! Global registry for `#[server]` actions.
//!
//! `#[server]` macro emits a `ctor` that calls `register_server_action`. At
//! runtime, the `/_resuma/action/:name` route dispatches the call.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use resuma_core::{ResumaError, Result};
use serde_json::Value;

pub type ActionFuture = Pin<Box<dyn Future<Output = Result<Value>> + Send>>;
pub type ActionFn = fn(Vec<Value>) -> ActionFuture;

static REGISTRY: Lazy<RwLock<HashMap<String, ActionFn>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub fn register_server_action(name: &str, f: ActionFn) {
    REGISTRY.write().insert(name.to_string(), f);
}

pub fn get_action(name: &str) -> Option<ActionFn> {
    REGISTRY.read().get(name).copied()
}

pub async fn dispatch(name: &str, args: Vec<Value>) -> Result<Value> {
    match get_action(name) {
        Some(f) => f(args).await,
        None => Err(ResumaError::UnknownAction(name.to_string())),
    }
}
