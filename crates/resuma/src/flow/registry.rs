//! Global registries for Resuma Flow `#[load]` and `#[submit]` handlers.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::core::{Result, ResumaError};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde_json::Value;

use super::request::FlowRequest;

pub type LoadFuture = Pin<Box<dyn Future<Output = Result<Value>> + Send>>;
pub type LoadFn = fn(FlowRequest) -> LoadFuture;

pub type SubmitFuture = Pin<Box<dyn Future<Output = Result<Value>> + Send>>;
pub type SubmitFn = fn(Value, FlowRequest) -> SubmitFuture;

static LOADS: Lazy<RwLock<HashMap<String, LoadFn>>> = Lazy::new(|| RwLock::new(HashMap::new()));
static SUBMITS: Lazy<RwLock<HashMap<String, SubmitFn>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub fn register_loader(name: &str, f: LoadFn) {
    let mut reg = LOADS.write();
    if reg.contains_key(name) {
        panic!("loader `{name}` registered twice");
    }
    reg.insert(name.to_string(), f);
}

pub fn register_submit(name: &str, f: SubmitFn) {
    let mut reg = SUBMITS.write();
    if reg.contains_key(name) {
        panic!("submit handler `{name}` registered twice");
    }
    reg.insert(name.to_string(), f);
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
        None => Err(ResumaError::UnknownLoader(name.to_string())),
    }
}

pub async fn dispatch_submit(name: &str, data: Value, req: FlowRequest) -> Result<Value> {
    match get_submit(name) {
        Some(f) => f(data, req).await,
        None => Err(ResumaError::UnknownSubmit(name.to_string())),
    }
}
