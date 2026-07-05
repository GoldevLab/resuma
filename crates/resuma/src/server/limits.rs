//! Server-level DoS protections: request timeout and in-flight concurrency cap.
//!
//! These bound resources against slow/abusive clients (e.g. Slowloris-style
//! stalls or request floods) without depending on an upstream proxy.
//!
//! - `RESUMA_REQUEST_TIMEOUT_SECS` — max seconds to *produce a response*
//!   (default `30`; `0` disables). The timeout covers handler execution, body
//!   read and origin/CSRF checks. It does **not** limit long-lived response
//!   bodies (SSE/WebSocket resolve their response immediately, then stream), so
//!   those keep working.
//! - `RESUMA_MAX_CONCURRENT` — max concurrent in-flight requests (default:
//!   unset = unlimited). Set behind untrusted networks to cap peak memory.

use std::net::SocketAddr;
use std::time::Duration;

use axum::Router;

const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;

/// Request timeout, or `None` when explicitly disabled with `0`.
pub fn request_timeout() -> Option<Duration> {
    let secs = std::env::var("RESUMA_REQUEST_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_REQUEST_TIMEOUT_SECS);
    (secs > 0).then(|| Duration::from_secs(secs))
}

/// Global in-flight concurrency cap, or `None` when unset.
pub fn max_concurrency() -> Option<usize> {
    std::env::var("RESUMA_MAX_CONCURRENT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| *n > 0)
}

/// Layer request-timeout and (optional) concurrency limits onto the router.
pub fn apply_server_limits(mut router: Router) -> Router {
    if let Some(timeout) = request_timeout() {
        router = router.layer(tower_http::timeout::TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            timeout,
        ));
    }
    if let Some(limit) = max_concurrency() {
        router = router.layer(tower::limit::GlobalConcurrencyLimitLayer::new(limit));
    }
    router
}

/// Emit a loud warning when binding to a non-loopback interface without
/// production hardening (`RESUMA_ENV=production`).
///
/// Outside production, internal error messages are returned verbatim to clients
/// and the origin check is relaxed — safe for local dev, dangerous when exposed.
pub fn warn_if_exposed_without_hardening(bound: SocketAddr, production: bool) {
    if production || bound.ip().is_loopback() {
        return;
    }
    tracing::warn!(
        addr = %bound,
        "resuma is bound to a non-loopback address without RESUMA_ENV=production: \
         internal error details are exposed to clients and origin checks are relaxed. \
         Set RESUMA_ENV=production before serving public traffic."
    );
    eprintln!(
        "[resuma] WARNING: listening on {bound} without RESUMA_ENV=production — \
         error messages are not sanitized and origin checks are relaxed. \
         Set RESUMA_ENV=production for public deployments."
    );
}
