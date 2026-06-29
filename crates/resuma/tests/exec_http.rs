//! HTTP integration tests for the execution layer.

use std::net::SocketAddr;

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use resuma::exec::{configure_exec_security, ExecSecurityConfig};
use resuma::prelude::*;
use tower::ServiceExt;

fn test_connect_info() -> ConnectInfo<SocketAddr> {
    ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345)))
}

#[tokio::test]
async fn exec_status_requires_api_key_when_configured() {
    configure_exec_security(ExecSecurityConfig {
        api_key: Some("test-exec-api-key-32chars-min!!!!".into()),
        public: false,
        ..ExecSecurityConfig::from_env()
    });

    let app = ResumaApp::new().into_router();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/_resuma/status")
                .extension(test_connect_info())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn exec_status_accepts_bearer_api_key() {
    configure_exec_security(ExecSecurityConfig {
        api_key: Some("test-exec-api-key-32chars-min!!!!".into()),
        public: false,
        ..ExecSecurityConfig::from_env()
    });

    let app = ResumaApp::new().into_router();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/_resuma/status")
                .header("authorization", "Bearer test-exec-api-key-32chars-min!!!!")
                .extension(test_connect_info())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn exec_worker_rejects_without_api_key_in_production_mode() {
    configure_exec_security(ExecSecurityConfig {
        api_key: Some("test-exec-api-key-32chars-min!!!!".into()),
        public: false,
        ..ExecSecurityConfig::from_env()
    });

    let app = ResumaApp::new().into_router();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_resuma/worker/unknown")
                .header("content-type", "application/json")
                .extension(test_connect_info())
                .body(Body::from(r#"{"input":{}}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
