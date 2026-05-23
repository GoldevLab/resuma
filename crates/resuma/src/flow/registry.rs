//! Global registries for Resuma Flow `#[load]` and `#[submit]` handlers.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use crate::core::{ResumaError, Result};
use serde_json::Value;

use super::request::FlowRequest;

pub type LoadFuture = Pin<Box<dyn Future<Output = Result<Value>> + Send>>;
pub type LoadFn = fn(FlowRequest) -> LoadFuture;

pub type SubmitFuture = Pin<Box<dyn Future<Output = Result<Value>> + Send>>;
pub type SubmitFn = fn(Value, FlowRequest) -> SubmitFuture;

static LOADS: Lazy<RwLock<HashMap<String, LoadFn>>> = Lazy::new(|| RwLock::new(HashMap::new()));
static SUBMITS: Lazy<RwLock<HashMap<String, SubmitFn>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub fn register_loader(name: &str, f: LoadFn) {
    LOADS.write().insert(name.to_string(), f);
}

pub fn register_submit(name: &str, f: SubmitFn) {
    SUBMITS.write().insert(name.to_string(), f);
}

pub fn get_loader(name: &str) -> Option<LoadFn> {
    LOADS.read().get(name).copied()
}

pub fn get_submit(name: &str) -> Option<SubmitFn> {
    SUBMITS.read().get(name).copied()
}

pub async fn dispatch_load(name: &str, req: FlowRequest) -> Result<Value> {
    match get_loader(name) {
        Some(f) => f(req).await,
        None => Err(ResumaError::Other(format!("unknown loader `{name}`"))),
    }
}

pub async fn dispatch_submit(name: &str, data: Value, req: FlowRequest) -> Result<Value> {
    match get_submit(name) {
        Some(f) => f(data, req).await,
        None => Err(ResumaError::Other(format!("unknown submit `{name}`"))),
    }
}
