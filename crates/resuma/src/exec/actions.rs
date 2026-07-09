//! Built-in server actions for the execution layer (Flow dashboard polling).

use crate::core::{FlowRequest, Result};
use crate::server::register_server_action;
use serde_json::Value;

/// Register `exec_status` action — returns [`super::status::ExecStatus`] as JSON.
/// Callable from the browser via `__resuma.action("exec_status", [])` (CSRF + middleware).
pub fn register_builtin_actions() {
    register_server_action("exec_status", exec_status_action);
}

fn exec_status_action(
    _args: Vec<Value>,
    req: FlowRequest,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value>> + Send>> {
    Box::pin(async move {
        super::security::guard_exec_status_action(&req)?;
        Ok(serde_json::to_value(super::status::snapshot())?)
    })
}
