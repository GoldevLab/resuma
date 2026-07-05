//! Production ops helpers shared by `ResumaApp` and `FlowApp`:
//! health/readiness probes, request-id + latency tracing, and graceful shutdown.

use std::time::Instant;

use axum::body::Body;
use axum::http::{HeaderName, HeaderValue, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

/// Header carrying a per-request correlation id (read from the client if present,
/// otherwise generated). Mirrored back on the response.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Liveness probe path. Returns `200 OK` as soon as the process can serve HTTP.
pub const HEALTH_PATH: &str = "/health";

/// Readiness probe path. Returns `200 OK` when the app is ready to receive traffic.
pub const READY_PATH: &str = "/ready";

/// Liveness handler — the process is up and the axum stack is serving.
pub async fn health() -> Response {
    (StatusCode::OK, "ok").into_response()
}

/// Readiness handler — safe default returns ready; apps with external
/// dependencies (DB, cache) can register their own `/ready` page to override.
pub async fn ready() -> Response {
    (StatusCode::OK, "ready").into_response()
}

/// Generate or propagate a request id, emit a tracing span with method/path/
/// status/latency, and echo `x-request-id` on the response.
pub async fn request_id_middleware(mut req: Request<Body>, next: Next) -> Response {
    super::page_cache::clear_request_staging();
    super::request_path::clear_request_staging();

    let incoming = req
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty() && s.len() <= 128)
        .map(|s| s.to_string());
    let request_id = incoming.unwrap_or_else(super::security::random_token);

    let method = req.method().clone();
    let path = req.uri().path().to_string();

    // Make the id available to handlers via request extensions.
    req.extensions_mut().insert(RequestId(request_id.clone()));
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        req.headers_mut()
            .insert(HeaderName::from_static("x-request-id"), value);
    }

    let started = Instant::now();
    // Isolate staged page metadata (CSRF token, CSP nonce, cache headers) per
    // request task so concurrent requests on the same worker thread cannot
    // clobber each other's staging mid-render.
    let mut res = super::page_cache::scope_page_staging(crate::flow::runtime::scope_flow_runtime(
        next.run(req),
    ))
    .await;
    let latency_ms = started.elapsed().as_millis();
    let status = res.status().as_u16();

    tracing::info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        status,
        latency_ms,
        "request"
    );

    if let Ok(value) = HeaderValue::from_str(&request_id) {
        res.headers_mut()
            .insert(HeaderName::from_static("x-request-id"), value);
    }

    super::page_cache::clear_request_staging();
    super::request_path::clear_request_staging();
    res
}

/// Per-request correlation id stored in request extensions.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

/// Resolve when the process receives a shutdown signal (`Ctrl+C` on all
/// platforms, plus `SIGTERM` on Unix for Fly.io / Kubernetes rolling deploys).
pub async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }

    tracing::info!("shutdown signal received; draining connections");
}
