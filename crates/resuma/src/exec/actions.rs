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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::security::{configure, ExecSecurityConfig};

    #[tokio::test]
    async fn exec_status_action_requires_auth_when_api_key_set() {
        configure(ExecSecurityConfig {
            api_key: Some("super-secret-key-for-tests-only".into()),
            public: false,
            ..ExecSecurityConfig::from_env()
        });
        let err = exec_status_action(vec![], FlowRequest::default())
            .await
            .expect_err("unauthenticated");
        assert!(matches!(err, crate::core::ResumaError::Unauthorized));
    }

    #[tokio::test]
    async fn exec_status_action_allows_authenticated_session() {
        configure(ExecSecurityConfig {
            api_key: Some("super-secret-key-for-tests-only".into()),
            public: false,
            ..ExecSecurityConfig::from_env()
        });
        let mut req = FlowRequest::default();
        req.set_extension("authenticated", serde_json::json!(true));
        let out = exec_status_action(vec![], req).await.expect("ok");
        assert!(out.get("ok").and_then(|v| v.as_bool()).unwrap_or(false));
    }
}
