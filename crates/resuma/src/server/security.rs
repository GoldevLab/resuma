//! Security primitives for Resuma HTTP servers — CSRF, rate limiting, headers, origin checks.
//!
//! Enabled by default on `ResumaApp::serve()` and `FlowApp::serve()`. Configure via
//! [`SecurityConfig`] or environment variables (see `docs/SECURITY.md`).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::Result;
use crate::core::ResumaError;
use axum::http::{header, HeaderMap, HeaderValue, Request};
use axum::response::Response;
use once_cell::sync::Lazy;
use parking_lot::RwLock;

/// Per-request CSP nonce stored in response extensions after HTML render.
#[derive(Clone, Debug)]
pub struct CspNonce(pub String);

/// Cookie name for double-submit CSRF protection.
pub const CSRF_COOKIE: &str = "__resuma-csrf";
/// Header clients must send on POST (actions + submits).
pub const CSRF_HEADER: &str = "x-resuma-csrf";
/// Form field name for progressive-enhancement submits.
pub const CSRF_FIELD: &str = "_csrf";

static CONFIG: Lazy<RwLock<SecurityConfig>> = Lazy::new(|| RwLock::new(SecurityConfig::from_env()));

static RATE_BUCKETS: Lazy<RwLock<HashMap<String, Vec<Instant>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Global security configuration (shared by ResumaApp and FlowApp).
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Require CSRF token on `POST /_resuma/action/*` and `POST /_resuma/submit/*`.
    pub csrf: bool,
    /// Validate `Origin` / `Referer` on mutating requests (same-origin).
    pub origin_check: bool,
    /// Trust `X-Forwarded-For` / `X-Forwarded-Proto` (set `RESUMA_TRUST_PROXY=1` behind Fly/nginx).
    pub trust_proxy: bool,
    /// Max POST body size in bytes.
    pub body_limit_bytes: usize,
    /// Max action RPC calls per client IP per minute.
    pub actions_per_minute: u32,
    /// Max form submits per client IP per minute.
    pub submits_per_minute: u32,
    /// Hide `/_resuma/benchmark.json` in production.
    pub hide_benchmark: bool,
    /// Sanitize error messages returned to clients.
    pub production: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl SecurityConfig {
    pub fn from_env() -> Self {
        let production = matches!(
            std::env::var("RESUMA_ENV").as_deref(),
            Ok("production") | Ok("prod")
        );
        let trust_proxy = matches!(
            std::env::var("RESUMA_TRUST_PROXY").as_deref(),
            Ok("1") | Ok("true") | Ok("TRUE")
        );
        Self {
            csrf: !env_flag_off("RESUMA_CSRF"),
            origin_check: !env_flag_off("RESUMA_ORIGIN_CHECK"),
            trust_proxy,
            body_limit_bytes: std::env::var("RESUMA_BODY_LIMIT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1024 * 1024),
            actions_per_minute: std::env::var("RESUMA_RATE_ACTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(120),
            submits_per_minute: std::env::var("RESUMA_RATE_SUBMITS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            hide_benchmark: production,
            production,
        }
    }
}

fn env_flag_off(name: &str) -> bool {
    matches!(
        std::env::var(name).as_deref(),
        Ok("0") | Ok("false") | Ok("FALSE") | Ok("off")
    )
}

/// Install global security config (call before `serve()` to override env defaults).
pub fn configure(config: SecurityConfig) {
    *CONFIG.write() = config;
}

pub fn config() -> SecurityConfig {
    CONFIG.read().clone()
}

/// Cryptographically random token (32 hex chars).
pub fn random_token() -> String {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).expect("OS random number generator");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn csrf_token() -> String {
    random_token()
}

/// True when the request arrived over HTTPS (direct TLS or `X-Forwarded-Proto`).
pub fn request_is_https<B>(req: &Request<B>) -> bool {
    let cfg = config();
    if cfg.trust_proxy {
        if let Some(proto) = req
            .headers()
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
        {
            if proto.eq_ignore_ascii_case("https") {
                return true;
            }
        }
    }
    req.uri().scheme_str() == Some("https")
}

/// Best-effort client IP for rate limiting.
pub fn client_ip<B>(req: &Request<B>) -> String {
    client_ip_from_parts(req.headers(), connect_addr(req))
}

pub fn client_ip_from_parts(headers: &HeaderMap, connect: Option<SocketAddr>) -> String {
    let cfg = config();
    if cfg.trust_proxy {
        if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
            if let Some(first) = xff.split(',').next() {
                let ip = first.trim();
                if !ip.is_empty() {
                    return ip.to_string();
                }
            }
        }
        if let Some(xri) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
            if !xri.is_empty() {
                return xri.to_string();
            }
        }
    }
    connect
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn connect_addr<B>(req: &Request<B>) -> Option<SocketAddr> {
    req.extensions()
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0)
}

/// Sliding-window rate limit. Returns `Err(RateLimited)` when exceeded.
pub fn check_rate_limit(ip: &str, bucket: &str, limit_per_minute: u32) -> Result<()> {
    if limit_per_minute == 0 {
        return Ok(());
    }
    let key = format!("{bucket}:{ip}");
    let now = Instant::now();
    let window = Duration::from_secs(60);
    let mut map = RATE_BUCKETS.write();
    let entries = map.entry(key).or_default();
    entries.retain(|t| now.duration_since(*t) < window);
    if entries.len() as u32 >= limit_per_minute {
        return Err(ResumaError::RateLimited);
    }
    entries.push(now);
    Ok(())
}

fn header_str(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie = header_str(headers, header::COOKIE.as_str())?;
    for part in cookie.split(';') {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=') {
            if k.trim() == name {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

/// Validate double-submit CSRF: header (or form field) must match cookie.
pub fn validate_csrf(headers: &HeaderMap, form_csrf: Option<&str>) -> Result<()> {
    let cfg = config();
    if !cfg.csrf {
        return Ok(());
    }
    let cookie = cookie_value(headers, CSRF_COOKIE).ok_or(ResumaError::InvalidCsrf)?;
    let header = header_str(headers, CSRF_HEADER);
    let token = header
        .as_deref()
        .or(form_csrf)
        .ok_or(ResumaError::InvalidCsrf)?;
    if token != cookie || token.len() < 16 {
        return Err(ResumaError::InvalidCsrf);
    }
    Ok(())
}

/// Reject cross-origin POST when `Origin`/`Referer` do not match the host.
pub fn validate_origin(headers: &HeaderMap, host: &str) -> Result<()> {
    let cfg = config();
    if !cfg.origin_check {
        return Ok(());
    }
    let host = host.split(':').next().unwrap_or(host).to_lowercase();

    if let Some(origin) = header_str(headers, header::ORIGIN.as_str()) {
        if !origin_matches_host(&origin, &host) {
            return Err(ResumaError::Forbidden("cross-origin request".into()));
        }
        return Ok(());
    }

    if let Some(referer) = header_str(headers, header::REFERER.as_str()) {
        if !referer_host_matches(&referer, &host) {
            return Err(ResumaError::Forbidden("invalid referer".into()));
        }
    }
    Ok(())
}

fn origin_matches_host(origin: &str, host: &str) -> bool {
    origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))
        .and_then(|rest| rest.split('/').next())
        .map(|h| {
            h.eq_ignore_ascii_case(host)
                || h.strip_prefix("www.").unwrap_or(h) == host.strip_prefix("www.").unwrap_or(host)
        })
        .unwrap_or(false)
}

fn referer_host_matches(referer: &str, host: &str) -> bool {
    referer
        .strip_prefix("http://")
        .or_else(|| referer.strip_prefix("https://"))
        .and_then(|rest| rest.split('/').next())
        .map(|h| h.eq_ignore_ascii_case(host))
        .unwrap_or(false)
}

/// Build `Set-Cookie` for CSRF double-submit.
pub fn csrf_set_cookie(token: &str, https: bool) -> HeaderValue {
    let secure = if https { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "{CSRF_COOKIE}={token}; Path=/; SameSite=Strict; HttpOnly{secure}"
    ))
    .unwrap_or_else(|_| HeaderValue::from_static("invalid"))
}

/// Options passed to [`apply_security_headers`].
#[derive(Debug, Clone, Default)]
pub struct SecurityHeaderOptions {
    pub csp_nonce: Option<String>,
    pub https: bool,
}

/// Apply standard security headers (Helmet-style baseline).
pub fn apply_security_headers(mut response: Response, opts: &SecurityHeaderOptions) -> Response {
    let headers = response.headers_mut();
    if opts.https {
        insert_header(
            headers,
            header::STRICT_TRANSPORT_SECURITY,
            "max-age=63072000; includeSubDomains; preload",
        );
    }
    insert_header(headers, header::X_FRAME_OPTIONS, "DENY");
    insert_header(headers, header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    insert_header(
        headers,
        header::HeaderName::from_static("x-xss-protection"),
        "0",
    );
    insert_header(
        headers,
        header::REFERRER_POLICY,
        "strict-origin-when-cross-origin",
    );
    insert_header(
        headers,
        header::HeaderName::from_static("permissions-policy"),
        "camera=(), microphone=(), geolocation=()",
    );
    insert_header(
        headers,
        header::HeaderName::from_static("cross-origin-opener-policy"),
        "same-origin",
    );
    insert_header(
        headers,
        header::HeaderName::from_static("cross-origin-resource-policy"),
        "same-origin",
    );
    insert_header(
        headers,
        header::HeaderName::from_static("x-dns-prefetch-control"),
        "off",
    );

    let csp = if let Some(nonce) = &opts.csp_nonce {
        let mut policy = format!(
            "default-src 'self'; script-src 'self' 'nonce-{nonce}'; style-src 'self' 'nonce-{nonce}'; img-src 'self' data:; font-src 'self'; connect-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'"
        );
        if opts.https {
            policy.push_str("; upgrade-insecure-requests");
        }
        policy
    } else {
        let mut policy = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'".to_string();
        if opts.https {
            policy.push_str("; upgrade-insecure-requests");
        }
        policy
    };
    insert_header(headers, header::CONTENT_SECURITY_POLICY, &csp);
    response
}

fn insert_header(headers: &mut axum::http::HeaderMap, name: header::HeaderName, value: &str) {
    if let Ok(v) = HeaderValue::from_str(value) {
        headers.insert(name, v);
    }
}

/// Guard mutating API requests (CSRF + origin + rate limit).
pub fn guard_mutation(
    headers: &HeaderMap,
    host: &str,
    ip: &str,
    bucket: &str,
    limit: u32,
    form_csrf: Option<&str>,
) -> Result<()> {
    check_rate_limit(ip, bucket, limit)?;
    validate_origin(headers, host)?;
    validate_csrf(headers, form_csrf)?;
    Ok(())
}

/// Map [`ResumaError`] to an HTTP status code.
pub fn http_status(err: &ResumaError) -> axum::http::StatusCode {
    axum::http::StatusCode::from_u16(err.status_code())
        .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}

/// Shared state for security-aware routers.
#[derive(Clone, Default)]
pub struct SecurityState {
    pub config: Arc<SecurityConfig>,
}

impl SecurityState {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    pub fn current() -> Self {
        Self::new(config())
    }
}
