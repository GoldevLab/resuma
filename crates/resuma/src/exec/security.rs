//! Production security for the execution layer — API keys, graph tokens, input limits.

use axum::http::HeaderMap;

use crate::core::{FlowRequest, Result, ResumaError};
use crate::server::security::{
    self, check_rate_limit, client_ip_from_parts, validate_csrf, validate_origin, verify_secret,
};

use super::durable;
use super::types::GraphId;

/// Header for admin exec operations (`POST /_resuma/worker`, `POST /_resuma/queue`).
pub const EXEC_API_HEADER: &str = "x-resuma-exec-key";
/// Header or query param for graph-scoped access (SSE cannot send custom headers).
pub const GRAPH_TOKEN_HEADER: &str = "x-resuma-graph-token";
pub const GRAPH_TOKEN_QUERY: &str = "token";

static CONFIG: once_cell::sync::Lazy<parking_lot::RwLock<ExecSecurityConfig>> =
    once_cell::sync::Lazy::new(|| parking_lot::RwLock::new(ExecSecurityConfig::from_env()));

/// Execution-layer security settings (env-driven).
#[derive(Debug, Clone)]
pub struct ExecSecurityConfig {
    /// Shared secret for worker/queue admin routes. Required in production unless `public`.
    pub api_key: Option<String>,
    /// Allow unauthenticated exec routes (dev only; ignored when `api_key` is set in production).
    pub public: bool,
    /// Max worker/queue POSTs per IP per minute.
    pub workers_per_minute: u32,
    /// Max graph read/SSE requests per IP per minute.
    pub graph_reads_per_minute: u32,
    /// Max graph control POSTs (pause/resume/cancel) per IP per minute.
    pub graph_controls_per_minute: u32,
    /// Max serialized JSON input bytes for worker/queue bodies.
    pub max_input_bytes: usize,
    /// Max JSON nesting depth for worker/queue bodies.
    pub max_input_depth: u32,
    /// Require `Origin` or `Referer` on exec mutations when CSRF is enabled.
    pub strict_origin: bool,
    /// Allow unauthenticated `GET /_resuma/metrics` (scrape behind VPC only).
    pub metrics_public: bool,
}

impl ExecSecurityConfig {
    pub fn from_env() -> Self {
        let production = matches!(
            std::env::var("RESUMA_ENV").as_deref(),
            Ok("production") | Ok("prod")
        );
        let api_key = std::env::var("RESUMA_EXEC_API_KEY")
            .ok()
            .filter(|k| !k.is_empty());
        let public = matches!(
            std::env::var("RESUMA_EXEC_PUBLIC").as_deref(),
            Ok("1") | Ok("true") | Ok("TRUE")
        );
        Self {
            api_key,
            public: public && !production,
            workers_per_minute: env_u32("RESUMA_RATE_EXEC_WORKERS", 30),
            graph_reads_per_minute: env_u32("RESUMA_RATE_EXEC_GRAPH", 180),
            graph_controls_per_minute: env_u32("RESUMA_RATE_EXEC_CONTROL", 60),
            max_input_bytes: std::env::var("RESUMA_EXEC_MAX_INPUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(512 * 1024),
            max_input_depth: std::env::var("RESUMA_EXEC_MAX_DEPTH")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(32),
            strict_origin: production
                && !matches!(
                    std::env::var("RESUMA_EXEC_STRICT_ORIGIN").as_deref(),
                    Ok("0") | Ok("false") | Ok("FALSE")
                ),
            metrics_public: matches!(
                std::env::var("RESUMA_METRICS_PUBLIC").as_deref(),
                Ok("1") | Ok("true") | Ok("TRUE")
            ),
        }
    }

    /// True when admin routes require a configured API key.
    ///
    /// Fail-closed by default: routes require `RESUMA_EXEC_API_KEY` unless
    /// `RESUMA_EXEC_PUBLIC=1` is explicitly set (dev only; ignored in production).
    pub fn requires_api_key(&self) -> bool {
        !self.public
    }

    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }
}

fn env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Override exec security config (call before `init_exec()`).
pub fn configure(config: ExecSecurityConfig) {
    *CONFIG.write() = config;
}

pub fn config() -> ExecSecurityConfig {
    CONFIG.read().clone()
}

/// Issue and persist a graph-scoped access token.
pub fn issue_graph_token(graph_id: &GraphId) -> Result<String> {
    let token = security::random_token();
    durable::save_graph_token(graph_id, &token)?;
    Ok(token)
}

/// Validate a graph-scoped token (constant-time).
pub fn validate_graph_token(graph_id: &GraphId, token: Option<&str>) -> bool {
    let Some(expected) = durable::load_graph_token(graph_id) else {
        return false;
    };
    let Some(provided) = token.filter(|t| t.len() >= 16) else {
        return false;
    };
    verify_secret(&expected, provided)
}

fn header_str(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    header_str(headers, "authorization").and_then(|v| {
        v.strip_prefix("Bearer ")
            .or_else(|| v.strip_prefix("bearer "))
            .map(|t| t.trim().to_string())
    })
}

/// Extract admin API key from `Authorization: Bearer` or [`EXEC_API_HEADER`].
pub fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    bearer_token(headers).or_else(|| header_str(headers, EXEC_API_HEADER))
}

/// Extract graph token from header (query param handled by route handlers).
pub fn extract_graph_token(headers: &HeaderMap) -> Option<String> {
    header_str(headers, GRAPH_TOKEN_HEADER)
}

fn api_key_valid(headers: &HeaderMap) -> bool {
    let cfg = config();
    let Some(expected) = cfg.api_key() else {
        return !cfg.requires_api_key();
    };
    let Some(provided) = extract_api_key(headers) else {
        return false;
    };
    verify_secret(expected, &provided)
}

/// Like [`api_key_valid`] but never satisfied by public mode: a key must be
/// configured *and* match. Used for per-graph routes where public mode must
/// not grant access.
fn api_key_valid_strict(headers: &HeaderMap) -> bool {
    let cfg = config();
    let Some(expected) = cfg.api_key() else {
        return false;
    };
    let Some(provided) = extract_api_key(headers) else {
        return false;
    };
    verify_secret(expected, &provided)
}

/// Guard `exec_status` server action — same trust as admin HTTP routes.
pub fn guard_exec_status_action(req: &FlowRequest) -> Result<()> {
    let cfg = config();
    if !cfg.requires_api_key() {
        return Ok(());
    }
    if req.is_authenticated() || req.has_role("admin") {
        return Ok(());
    }
    if api_key_valid_from_request(req) {
        return Ok(());
    }
    Err(ResumaError::Unauthorized)
}

fn api_key_valid_from_request(req: &FlowRequest) -> bool {
    let cfg = config();
    let Some(expected) = cfg.api_key() else {
        return !cfg.requires_api_key();
    };
    let provided = req
        .header("authorization")
        .and_then(|v| {
            v.strip_prefix("Bearer ")
                .or_else(|| v.strip_prefix("bearer "))
        })
        .map(str::trim)
        .or_else(|| req.header(EXEC_API_HEADER));
    let Some(provided) = provided else {
        return false;
    };
    verify_secret(expected, provided)
}

/// Guard Prometheus metrics scrape (`GET /_resuma/metrics`).
pub fn guard_metrics(headers: &HeaderMap, ip: &str) -> Result<()> {
    let cfg = config();
    if cfg.metrics_public {
        return Ok(());
    }
    check_rate_limit(ip, "exec:metrics", cfg.graph_reads_per_minute)?;
    if cfg.requires_api_key() && !api_key_valid(headers) {
        return Err(ResumaError::Unauthorized);
    }
    Ok(())
}

/// Guard read-only admin routes (`GET /_resuma/status`, list endpoints).
pub fn guard_admin_read(headers: &HeaderMap, ip: &str) -> Result<()> {
    let cfg = config();
    check_rate_limit(ip, "exec:admin", cfg.graph_reads_per_minute)?;
    if cfg.requires_api_key() && !api_key_valid(headers) {
        return Err(ResumaError::Unauthorized);
    }
    Ok(())
}

/// Guard admin routes: worker start + queue enqueue.
pub fn guard_admin(
    headers: &HeaderMap,
    host: &str,
    ip: &str,
    form_csrf: Option<&str>,
) -> Result<()> {
    let cfg = config();
    check_rate_limit(ip, "exec:worker", cfg.workers_per_minute)?;
    if cfg.requires_api_key() && !api_key_valid(headers) {
        return Err(ResumaError::Unauthorized);
    }
    if cfg.strict_origin {
        validate_origin_strict(headers, host)?;
    } else {
        validate_origin(headers, host)?;
    }
    let sec = security::config();
    if sec.csrf {
        validate_csrf(headers, form_csrf)?;
    }
    Ok(())
}

/// Guard graph read routes (GET snapshot, replay, SSE).
///
/// A valid graph token or API key is always required — public mode only
/// relaxes worker/queue admin routes, never per-graph access.
pub fn guard_graph_read(
    headers: &HeaderMap,
    host: &str,
    ip: &str,
    graph_id: &GraphId,
    query_token: Option<&str>,
) -> Result<()> {
    let cfg = config();
    let _ = host;
    check_rate_limit(ip, "exec:graph", cfg.graph_reads_per_minute)?;
    if graph_access_granted(headers, graph_id, query_token) {
        return Ok(());
    }
    Err(ResumaError::Unauthorized)
}

/// Guard graph control routes (pause, resume, cancel).
pub fn guard_graph_control(
    headers: &HeaderMap,
    host: &str,
    ip: &str,
    graph_id: &GraphId,
    query_token: Option<&str>,
    form_csrf: Option<&str>,
) -> Result<()> {
    let cfg = config();
    check_rate_limit(ip, "exec:control", cfg.graph_controls_per_minute)?;
    if !graph_access_granted(headers, graph_id, query_token) {
        return Err(ResumaError::Unauthorized);
    }
    if cfg.strict_origin {
        validate_origin_strict(headers, host)?;
    } else {
        validate_origin(headers, host)?;
    }
    let sec = security::config();
    if sec.csrf {
        validate_csrf(headers, form_csrf)?;
    }
    Ok(())
}

fn graph_access_granted(
    headers: &HeaderMap,
    graph_id: &GraphId,
    query_token: Option<&str>,
) -> bool {
    if api_key_valid_strict(headers) {
        return true;
    }
    let header_token = extract_graph_token(headers);
    let token = header_token.as_deref().or(query_token);
    validate_graph_token(graph_id, token)
}

/// Reject mutations when both `Origin` and `Referer` are absent (production hardening).
fn validate_origin_strict(headers: &HeaderMap, host: &str) -> Result<()> {
    let has_origin = headers.get("origin").is_some();
    let has_referer = headers.get("referer").is_some();
    if !has_origin && !has_referer {
        return Err(ResumaError::Forbidden("origin or referer required".into()));
    }
    validate_origin(headers, host)
}

/// Validate worker/queue resource names (path segments).
pub fn validate_resource_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 64 {
        return Err(ResumaError::validation("invalid resource name length"));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ResumaError::validation(
            "resource name must be alphanumeric, dash, or underscore",
        ));
    }
    Ok(())
}

/// Validate a scheduler/webhook resource id used to build on-disk paths.
///
/// Rejects path separators, `..`, and any character outside `[A-Za-z0-9_-]`, so a
/// percent-decoded path param such as `..%2f..%2fsecret` can never escape the jobs dir.
pub fn validate_schedule_id(id: &str) -> Result<()> {
    if id.is_empty() || id.len() > 128 {
        return Err(ResumaError::validation("invalid schedule id length"));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ResumaError::validation("invalid schedule id characters"));
    }
    Ok(())
}

/// Validate graph id format (unguessable token ids).
pub fn validate_graph_id(id: &str) -> Result<()> {
    if id.len() < 16 || id.len() > 128 {
        return Err(ResumaError::validation("invalid graph id"));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ResumaError::validation("invalid graph id characters"));
    }
    Ok(())
}

/// Validate JSON input size and nesting depth.
pub fn validate_input(value: &serde_json::Value) -> Result<()> {
    let cfg = config();
    let serialized = serde_json::to_string(value).map_err(ResumaError::Serde)?;
    if serialized.len() > cfg.max_input_bytes {
        return Err(ResumaError::PayloadTooLarge);
    }
    if json_depth(value) > cfg.max_input_depth {
        return Err(ResumaError::validation("JSON nesting too deep"));
    }
    Ok(())
}

fn json_depth(value: &serde_json::Value) -> u32 {
    match value {
        serde_json::Value::Object(map) => 1 + map.values().map(json_depth).max().unwrap_or(0),
        serde_json::Value::Array(items) => 1 + items.iter().map(json_depth).max().unwrap_or(0),
        _ => 1,
    }
}

/// Helper: client IP from request parts.
pub fn client_ip(headers: &HeaderMap, connect: Option<std::net::SocketAddr>) -> String {
    client_ip_from_parts(headers, connect)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn resource_name_rejects_traversal() {
        assert!(validate_resource_name("../etc").is_err());
        assert!(validate_resource_name("valid_worker").is_ok());
    }

    #[test]
    fn schedule_id_rejects_traversal() {
        // Percent-decoded traversal payloads must be rejected before touching disk.
        assert!(validate_schedule_id("../../etc/passwd").is_err());
        assert!(validate_schedule_id("..%2f..%2fsecret").is_err());
        assert!(validate_schedule_id("a/b").is_err());
        assert!(validate_schedule_id("a.b").is_err());
        assert!(validate_schedule_id("").is_err());
        assert!(validate_schedule_id("s_0123456789abcdef0123456789abcdef").is_ok());
    }

    #[test]
    fn requires_api_key_fail_closed_by_default() {
        let _guard = TEST_LOCK.lock().unwrap();
        configure(ExecSecurityConfig {
            api_key: None,
            public: false,
            ..ExecSecurityConfig::from_env()
        });
        assert!(config().requires_api_key());

        configure(ExecSecurityConfig {
            api_key: None,
            public: true,
            ..ExecSecurityConfig::from_env()
        });
        assert!(!config().requires_api_key());
    }

    #[test]
    fn input_depth_limit() {
        let _guard = TEST_LOCK.lock().unwrap();
        configure(ExecSecurityConfig {
            max_input_depth: 3,
            max_input_bytes: 1024,
            ..ExecSecurityConfig::from_env()
        });
        assert!(validate_input(&json!({ "a": { "b": { "c": { "d": 1 } } } })).is_err());
        assert!(validate_input(&json!({ "a": 1 })).is_ok());
    }

    #[test]
    fn graph_token_roundtrip() {
        let _guard = TEST_LOCK.lock().unwrap();
        let _guard = super::super::queue_disk::test_queue_lock().lock();
        super::super::durable::configure(
            std::env::temp_dir().join(format!("resuma-tok-{}", super::super::id::next_id())),
        );
        let gid = GraphId("g_testtoken123456".into());
        let token = issue_graph_token(&gid).expect("token");
        assert!(validate_graph_token(&gid, Some(&token)));
        assert!(!validate_graph_token(&gid, Some("wrong-token-value")));
    }
}
