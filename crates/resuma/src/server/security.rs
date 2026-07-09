//! Security primitives for Resuma HTTP servers — CSRF, rate limiting, headers, origin checks.
//!
//! Enabled by default on `ResumaApp::serve()` and `FlowApp::serve()`. Configure via
//! [`SecurityConfig`] or environment variables (see `docs/SECURITY.md`).

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use crate::core::Result;
use crate::core::ResumaError;
use axum::http::{header, HeaderMap, HeaderValue, Request};
use axum::response::Response;
use once_cell::sync::Lazy;
use parking_lot::RwLock;

use super::rate_limit::{self, RateLimitBackend};

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

static RATE_INIT: Lazy<()> = Lazy::new(|| {
    rate_limit::install_default_backend();
});

/// Content-Security-Policy tuning (Qwik-style per-request nonces + configurable directives).
///
/// In dev (`RESUMA_DEV=1`), CSP is **off** by default so tooling matches Qwik's `isDev` skip.
/// Set `RESUMA_CSP_DEV=1` to enforce CSP while developing.
#[derive(Debug, Clone)]
pub struct CspConfig {
    /// Emit a `Content-Security-Policy` (or Report-Only) header on HTML responses.
    pub enabled: bool,
    /// Use `Content-Security-Policy-Report-Only` instead of enforcing.
    pub report_only: bool,
    /// Add `'strict-dynamic'` to `script-src` when a nonce is present (modern browsers).
    pub strict_dynamic: bool,
    /// Allow `eval` / `new Function` for the resumability runtime. Keep `true` unless you replace the client.
    pub unsafe_eval: bool,
    /// Extra `img-src` origins (e.g. `https://images.pexels.com`).
    pub img_src: Vec<String>,
    /// Extra `script-src` origins.
    pub script_src: Vec<String>,
    /// Extra `style-src` / `style-src-elem` origins (in addition to Google Fonts).
    pub style_src: Vec<String>,
    /// Extra `connect-src` origins (APIs, analytics).
    pub connect_src: Vec<String>,
    /// Extra `font-src` origins.
    pub font_src: Vec<String>,
}

impl Default for CspConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl CspConfig {
    pub fn from_env() -> Self {
        let enabled = !env_flag_off("RESUMA_CSP")
            && (env_flag_on("RESUMA_CSP_DEV") || !crate::server::dev::dev_mode_enabled());
        Self {
            enabled,
            report_only: env_flag_on("RESUMA_CSP_REPORT_ONLY"),
            strict_dynamic: !env_flag_off("RESUMA_CSP_STRICT_DYNAMIC"),
            unsafe_eval: !env_flag_off("RESUMA_CSP_UNSAFE_EVAL"),
            img_src: parse_csp_list_env("RESUMA_CSP_IMG_SRC"),
            script_src: parse_csp_list_env("RESUMA_CSP_SCRIPT_SRC"),
            style_src: parse_csp_list_env("RESUMA_CSP_STYLE_SRC"),
            connect_src: parse_csp_list_env("RESUMA_CSP_CONNECT_SRC"),
            font_src: parse_csp_list_env("RESUMA_CSP_FONT_SRC"),
        }
    }

    /// Permissive dev profile: CSP disabled (same idea as Qwik `plugin@csp` when `isDev`).
    pub fn disabled() -> Self {
        let mut c = Self::from_env();
        c.enabled = false;
        c
    }

    /// Strict production profile with optional extra image hosts.
    pub fn production(extra_img_src: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut c = Self::from_env();
        c.enabled = true;
        c.img_src = extra_img_src.into_iter().map(Into::into).collect();
        c
    }
}

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
    /// CSP headers and directive allowlists.
    pub csp: CspConfig,
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
            actions_per_minute: parse_rate_limit("RESUMA_RATE_ACTIONS", 120),
            submits_per_minute: parse_rate_limit("RESUMA_RATE_SUBMITS", 60),
            hide_benchmark: production,
            production,
            csp: CspConfig::from_env(),
        }
    }
}

fn env_flag_on(name: &str) -> bool {
    matches!(
        std::env::var(name).as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("on")
    )
}

fn parse_csp_list_env(name: &str) -> Vec<String> {
    std::env::var(name)
        .ok()
        .map(|raw| {
            raw.split(|c: char| c.is_whitespace() || c == ',')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn env_flag_off(name: &str) -> bool {
    matches!(
        std::env::var(name).as_deref(),
        Ok("0") | Ok("false") | Ok("FALSE") | Ok("off")
    )
}

fn parse_rate_limit(name: &str, default: u32) -> u32 {
    match std::env::var(name).ok().and_then(|v| v.parse().ok()) {
        Some(0) => {
            tracing::warn!(
                env = name,
                default,
                "rate limit of 0 disables limiting — using default"
            );
            default
        }
        Some(n) => n,
        None => default,
    }
}

/// Install global security config (call before `serve()` to override env defaults).
pub fn configure(config: SecurityConfig) {
    *CONFIG.write() = config;
}

pub fn config() -> SecurityConfig {
    CONFIG.read().clone()
}

/// Cryptographically random token (32 hex chars).
pub fn try_random_token() -> Result<String> {
    #[cfg(test)]
    if RNG_FORCE_FAIL.with(|cell| cell.get()) {
        return Err(ResumaError::ServiceUnavailable(
            "random number generator unavailable (test override)".into(),
        ));
    }
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes)
        .map_err(|e| ResumaError::ServiceUnavailable(format!("random number generator unavailable: {e}")))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// Cryptographically random token (32 hex chars).
///
/// Non-security identifiers (request ids). Uses a monotonic counter fallback when
/// the OS RNG is unavailable — never use for CSRF, CSP nonces, or secrets.
/// Prefer [`try_random_token`] for security-sensitive paths that must fail closed.
pub fn random_token() -> String {
    try_random_token().unwrap_or_else(|e| {
        tracing::error!(error = %e, "OS random number generator failed — using counter fallback");
        counter_fallback_token("req")
    })
}

fn counter_fallback_token(prefix: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    format!("{prefix}-{:016x}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Issue a fresh CSRF token. Fails closed when the OS RNG is unavailable.
pub fn csrf_token() -> Result<String> {
    try_random_token()
}

/// Resolve CSRF for a page render: reuse session cookie or mint a new token.
pub fn resolve_page_csrf(headers: &HeaderMap, csrf_enabled: bool) -> Result<(String, bool)> {
    if let Some(token) = csrf_from_cookie(headers) {
        return Ok((token, false));
    }
    if csrf_enabled {
        Ok((try_random_token()?, true))
    } else {
        Ok((String::new(), false))
    }
}

/// Resolve CSP nonce for a page render. Required when CSP is enabled.
pub fn resolve_page_csp_nonce(csp_enabled: bool) -> Result<String> {
    if csp_enabled {
        try_random_token()
    } else {
        Ok(String::new())
    }
}

/// Validate security config before `serve()`. Fails closed when
/// `RESUMA_TRUST_PROXY=1` is set without explicit `RESUMA_TRUSTED_PROXY_CIDRS`.
pub fn validate_config(cfg: &SecurityConfig) -> Result<()> {
    if cfg.trust_proxy && TRUSTED_PROXY_CIDRS.is_empty() {
        return Err(ResumaError::Other(
            "RESUMA_TRUSTED_PROXY_CIDRS is required when RESUMA_TRUST_PROXY=1 \
             (comma-separated CIDRs for proxies that overwrite X-Forwarded-For)"
                .into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn set_rng_force_fail(fail: bool) {
    RNG_FORCE_FAIL.with(|cell| cell.set(fail));
}

#[cfg(test)]
thread_local! {
    static RNG_FORCE_FAIL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Reuse an existing CSRF double-submit cookie when present and well-formed.
pub fn csrf_from_cookie(headers: &HeaderMap) -> Option<String> {
    let token = cookie_value(headers, CSRF_COOKIE)?;
    (token.len() == 32 && token.chars().all(|c| c.is_ascii_hexdigit())).then_some(token)
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
///
/// When `RESUMA_TRUST_PROXY=1`, the first `X-Forwarded-For` hop is trusted.
/// Only enable this behind a reverse proxy that **overwrites** (not appends to)
/// forwarding headers, or clients can spoof IPs to evade rate limits.
pub fn client_ip<B>(req: &Request<B>) -> String {
    client_ip_from_parts(req.headers(), connect_addr(req))
}

pub fn client_ip_from_parts(headers: &HeaderMap, connect: Option<SocketAddr>) -> String {
    let cfg = config();
    // Trusting forwarding headers is only safe when the direct peer is a proxy
    // we control (one that overwrites, not appends to, X-Forwarded-For). When
    // `RESUMA_TRUSTED_PROXY_CIDRS` is set (comma/space-separated CIDRs, e.g.
    // "10.0.0.0/8, fdaa::/16"), forwarding headers are only honored if the
    // connecting socket address falls inside one of those ranges.
    if cfg.trust_proxy && peer_is_trusted_proxy(connect) {
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

/// Parsed from `RESUMA_TRUSTED_PROXY_CIDRS` (comma/space-separated CIDRs).
static TRUSTED_PROXY_CIDRS: Lazy<Vec<(IpAddr, u8)>> = Lazy::new(parse_trusted_proxy_cidrs);

fn parse_trusted_proxy_cidrs() -> Vec<(IpAddr, u8)> {
    std::env::var("RESUMA_TRUSTED_PROXY_CIDRS")
        .ok()
        .map(|raw| {
            raw.split(|c: char| c.is_whitespace() || c == ',')
                .filter(|s| !s.is_empty())
                .filter_map(parse_cidr)
                .collect()
        })
        .unwrap_or_default()
}

fn parse_cidr(s: &str) -> Option<(IpAddr, u8)> {
    if let Some((ip, prefix)) = s.split_once('/') {
        let addr: IpAddr = ip.parse().ok()?;
        let p: u8 = prefix.parse().ok()?;
        Some((addr, p))
    } else {
        let addr: IpAddr = s.parse().ok()?;
        let p = if addr.is_ipv4() { 32 } else { 128 };
        Some((addr, p))
    }
}

fn ip_in_cidr(ip: IpAddr, network: IpAddr, prefix: u8) -> bool {
    match (ip, network) {
        (IpAddr::V4(ip), IpAddr::V4(net)) => {
            let mask = if prefix >= 32 {
                u32::MAX
            } else {
                u32::MAX << (32 - prefix)
            };
            (u32::from(ip) & mask) == (u32::from(net) & mask)
        }
        (IpAddr::V6(ip), IpAddr::V6(net)) if prefix >= 128 => ip == net,
        (IpAddr::V6(ip), IpAddr::V6(net)) => {
            let ip = ip.segments();
            let net = net.segments();
            let full = (prefix / 16) as usize;
            let rem = prefix % 16;
            for i in 0..full {
                if ip[i] != net[i] {
                    return false;
                }
            }
            if rem == 0 {
                return true;
            }
            let mask = !(0xffffu16 >> rem);
            (ip[full] & mask) == (net[full] & mask)
        }
        _ => false,
    }
}

/// Only honor `X-Forwarded-For` when the direct TCP peer is a trusted proxy.
fn peer_is_trusted_proxy(connect: Option<SocketAddr>) -> bool {
    let Some(addr) = connect else {
        return false;
    };
    let ip = addr.ip();
    if TRUSTED_PROXY_CIDRS.is_empty() {
        return false;
    }
    TRUSTED_PROXY_CIDRS
        .iter()
        .any(|(net, prefix)| ip_in_cidr(ip, *net, *prefix))
}

/// Sliding-window rate limit. Returns `Err(RateLimited)` when exceeded.
pub fn check_rate_limit(ip: &str, bucket: &str, limit_per_minute: u32) -> Result<()> {
    Lazy::force(&RATE_INIT);
    let effective_limit = if ip == "unknown" {
        (limit_per_minute / 4).max(1)
    } else {
        limit_per_minute
    };
    let key = format!("{bucket}:{ip}");
    rate_limit::check_rate_limit_key(&key, effective_limit)
}

/// Replace the global rate-limit backend (memory or disk by default).
pub fn configure_rate_limit_backend(backend: Arc<dyn RateLimitBackend>) {
    rate_limit::configure_rate_limit_backend(backend);
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
    if token.len() < 16 {
        // Compare against self to keep timing independent of token length.
        let _ = constant_time_eq(cookie.as_bytes(), cookie.as_bytes());
        return Err(ResumaError::InvalidCsrf);
    }
    if !verify_secret(&cookie, token) {
        return Err(ResumaError::InvalidCsrf);
    }
    Ok(())
}

/// Constant-time comparison for API keys, graph tokens, and CSRF secrets.
pub fn verify_secret(expected: &str, provided: &str) -> bool {
    constant_time_eq(expected.as_bytes(), provided.as_bytes())
}

/// Length-independent, constant-time byte comparison for secrets/tokens.
/// Avoids leaking match position or expected length via early-return timing.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let len_a = a.len();
    let len_b = b.len();
    let max_len = len_a.max(len_b);
    if max_len == 0 {
        return len_a == len_b;
    }
    let mut diff: u8 = u8::from(len_a != len_b);
    for i in 0..max_len {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        diff |= x ^ y;
    }
    diff == 0
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

/// Reject mutations when both `Origin` and `Referer` are absent (production hardening).
pub fn validate_origin_strict(headers: &HeaderMap, host: &str) -> Result<()> {
    let cfg = config();
    if !cfg.origin_check {
        return Ok(());
    }
    let has_origin = headers.get(header::ORIGIN.as_str()).is_some();
    let has_referer = headers.get(header::REFERER.as_str()).is_some();
    if !has_origin && !has_referer {
        return Err(ResumaError::Forbidden("origin or referer required".into()));
    }
    validate_origin(headers, host)
}

fn origin_matches_host(origin: &str, host: &str) -> bool {
    origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))
        .and_then(|rest| rest.split('/').next())
        // Browsers include the port in `Origin` (e.g. `http://localhost:3000`);
        // `host` arrives without it, so compare hostnames only.
        .map(|authority| authority.split(':').next().unwrap_or(authority))
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
        .map(|authority| authority.split(':').next().unwrap_or(authority))
        .map(|h| {
            h.eq_ignore_ascii_case(host)
                || h.strip_prefix("www.").unwrap_or(h) == host.strip_prefix("www.").unwrap_or(host)
        })
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

/// Build a CSP header value (Qwik-style nonces + Resuma runtime requirements).
pub fn build_content_security_policy(nonce: Option<&str>, https: bool, csp: &CspConfig) -> String {
    let mut directives: Vec<String> = vec![
        "default-src 'self'".into(),
        "base-uri 'self'".into(),
        "object-src 'none'".into(),
        "frame-ancestors 'none'".into(),
        "form-action 'self'".into(),
    ];

    let mut script_src = vec!["'self'".to_string()];
    if let Some(nonce) = nonce {
        script_src.push(format!("'nonce-{nonce}'"));
        if csp.strict_dynamic {
            script_src.push("'strict-dynamic'".into());
        }
    }
    if csp.unsafe_eval {
        script_src.push("'unsafe-eval'".into());
    }
    script_src.extend(csp.script_src.iter().cloned());
    directives.push(format!("script-src {}", script_src.join(" ")));

    let mut style_src = vec!["'self'".to_string()];
    if let Some(nonce) = nonce {
        style_src.push(format!("'nonce-{nonce}'"));
    }
    style_src.push("'unsafe-inline'".into());
    style_src.push("https://fonts.googleapis.com".into());
    style_src.extend(csp.style_src.iter().cloned());
    let style_joined = style_src.join(" ");
    directives.push(format!("style-src {style_joined}"));
    directives.push(format!("style-src-elem {style_joined}"));
    directives.push("style-src-attr 'unsafe-inline'".into());

    let mut img_src = vec!["'self'", "data:", "blob:"];
    img_src.extend(csp.img_src.iter().map(String::as_str));
    directives.push(format!("img-src {}", img_src.join(" ")));

    let mut font_src = vec!["'self'", "https://fonts.gstatic.com", "data:"];
    font_src.extend(csp.font_src.iter().map(String::as_str));
    directives.push(format!("font-src {}", font_src.join(" ")));

    let mut connect_src = vec!["'self'"];
    connect_src.extend(csp.connect_src.iter().map(String::as_str));
    directives.push(format!("connect-src {}", connect_src.join(" ")));

    if https {
        directives.push("upgrade-insecure-requests".into());
    }

    directives.join("; ")
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

    let sec = config();
    if sec.csp.enabled {
        let policy = build_content_security_policy(opts.csp_nonce.as_deref(), opts.https, &sec.csp);
        let header_name = if sec.csp.report_only {
            header::HeaderName::from_static("content-security-policy-report-only")
        } else {
            header::CONTENT_SECURITY_POLICY
        };
        insert_header(headers, header_name, &policy);
    }
    response
}

fn insert_header(headers: &mut axum::http::HeaderMap, name: header::HeaderName, value: &str) {
    match HeaderValue::from_str(value) {
        Ok(v) => {
            headers.insert(name, v);
        }
        Err(e) => {
            tracing::error!(header = ?name, error = %e, "invalid header value — skipped");
        }
    }
}

/// Validate handler/island chunk identifiers used in URLs and dynamic imports.
pub fn validate_chunk_id(id: &str) -> Result<()> {
    validate_identifier(id, 64, "chunk id")
}

/// Validate server action names used in `POST /_resuma/action/{name}`.
pub fn validate_action_name(name: &str) -> Result<()> {
    validate_identifier(name, 128, "action name")
}

/// Validate form submit handler names used in `POST /_resuma/submit/{name}`.
pub fn validate_submit_name(name: &str) -> Result<()> {
    validate_action_name(name)
}

fn validate_identifier(id: &str, max_len: usize, label: &str) -> Result<()> {
    if id.is_empty() || id.len() > max_len {
        return Err(ResumaError::validation(format!("invalid {label} length")));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ResumaError::validation(format!(
            "{label} must be alphanumeric, dash, or underscore"
        )));
    }
    Ok(())
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
    let cfg = config();
    if cfg.origin_check {
        if cfg.production || cfg.csrf {
            validate_origin_strict(headers, host)?;
        } else {
            validate_origin(headers, host)?;
        }
    }
    validate_csrf(headers, form_csrf)?;
    Ok(())
}

/// Emit warnings for insecure runtime configuration (non-fatal).
pub fn warn_insecure_config(cfg: &SecurityConfig) {
    if !cfg.csrf {
        tracing::warn!(
            "RESUMA_CSRF=0 — mutating requests are not CSRF-protected; \
             use only for local development"
        );
        eprintln!("[resuma] WARNING: CSRF protection is disabled (RESUMA_CSRF=0)");
    }
    if !cfg.origin_check {
        tracing::warn!(
            "RESUMA_ORIGIN_CHECK=0 — cross-origin POSTs are not rejected by origin check"
        );
        eprintln!("[resuma] WARNING: origin checks are disabled (RESUMA_ORIGIN_CHECK=0)");
    }
    if !cfg.csrf && !cfg.origin_check {
        tracing::warn!(
            "both CSRF and origin checks are disabled — mutating endpoints are wide open"
        );
        eprintln!(
            "[resuma] WARNING: CSRF and origin checks are both disabled — \
             do not expose this server to untrusted networks"
        );
    }
    if !cfg.production {
        tracing::warn!(
            "RESUMA_ENV is not production — error messages are not sanitized and \
             origin checks are relaxed when CSRF is off"
        );
    }
    let exec = crate::exec::security::config();
    if exec.public && !crate::server::dev::dev_mode_enabled() {
        tracing::warn!(
            "RESUMA_EXEC_PUBLIC=1 without RESUMA_DEV=1 — exec admin routes and \
             exec_status are exposed; set RESUMA_DEV=1 for local dev only"
        );
        eprintln!(
            "[resuma] WARNING: RESUMA_EXEC_PUBLIC=1 without RESUMA_DEV=1 — \
             restrict exec routes or enable authentication"
        );
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::time::Duration;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn origin_matches_ignoring_port() {
        // Browsers send the port in `Origin`; `host` arrives without it.
        assert!(origin_matches_host("http://localhost:3000", "localhost"));
        assert!(origin_matches_host("http://127.0.0.1:3939", "127.0.0.1"));
        assert!(origin_matches_host("https://example.com", "example.com"));
        assert!(origin_matches_host(
            "https://example.com:8443",
            "example.com"
        ));
        assert!(origin_matches_host(
            "https://www.example.com:443",
            "example.com"
        ));
    }

    #[test]
    fn origin_rejects_other_hosts() {
        assert!(!origin_matches_host("http://evil.test:3000", "localhost"));
        assert!(!origin_matches_host(
            "https://attacker.example",
            "example.com"
        ));
    }

    #[test]
    fn validate_chunk_id_rejects_traversal() {
        assert!(validate_chunk_id("../evil").is_err());
        assert!(validate_chunk_id("valid-chunk_1").is_ok());
    }

    #[test]
    fn referer_matches_ignoring_www() {
        assert!(referer_host_matches(
            "https://www.example.com/path",
            "example.com"
        ));
    }

    #[test]
    fn referer_matches_ignoring_port() {
        assert!(referer_host_matches(
            "http://localhost:3000/items",
            "localhost"
        ));
        assert!(referer_host_matches(
            "https://example.com:8443/x",
            "example.com"
        ));
        assert!(!referer_host_matches(
            "http://evil.test:3000/x",
            "localhost"
        ));
    }

    #[test]
    fn validate_origin_allows_same_host_with_port() {
        let mut headers = HeaderMap::new();
        headers.insert(header::ORIGIN, "http://localhost:3000".parse().unwrap());
        // host carries the port as it would from the HTTP `Host` header.
        assert!(validate_origin(&headers, "localhost:3000").is_ok());
    }

    #[test]
    fn constant_time_eq_matches_only_identical_bytes() {
        assert!(super::constant_time_eq(
            b"abc123def456ghij",
            b"abc123def456ghij"
        ));
        assert!(!super::constant_time_eq(
            b"abc123def456ghij",
            b"abc123def456ghiJ"
        ));
        assert!(!super::constant_time_eq(b"short", b"longer-value"));
        assert!(super::constant_time_eq(b"", b""));
    }

    #[test]
    fn try_random_token_returns_hex_string() {
        let t = try_random_token().expect("rng");
        assert_eq!(t.len(), 32);
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn rate_bucket_sweep_drops_expired_entries() {
        use std::time::Instant;
        let mut map: HashMap<String, Vec<Instant>> = HashMap::new();
        let now = Instant::now();
        let window = Duration::from_secs(60);
        let old = now - Duration::from_secs(120);
        map.insert("a:1".into(), vec![old]);
        map.insert("a:2".into(), vec![now]);
        map.retain(|_, entries| {
            entries.retain(|t| now.duration_since(*t) < window);
            !entries.is_empty()
        });
        assert!(!map.contains_key("a:1"), "expired bucket should be evicted");
        assert!(map.contains_key("a:2"), "fresh bucket should remain");
    }

    #[test]
    fn csp_allows_runtime_compiled_handlers() {
        let csp = build_content_security_policy(
            Some("abc123"),
            false,
            &CspConfig {
                enabled: true,
                strict_dynamic: true,
                unsafe_eval: true,
                ..CspConfig::from_env()
            },
        );

        assert!(csp.contains("'nonce-abc123'"));
        assert!(csp.contains("'strict-dynamic'"));
        assert!(csp.contains("'unsafe-eval'"));
        assert!(csp.contains("style-src 'self' 'nonce-abc123' 'unsafe-inline'"));
        assert!(csp.contains("style-src-elem 'self' 'nonce-abc123' 'unsafe-inline'"));
        assert!(csp.contains("style-src-attr 'unsafe-inline'"));
        assert!(csp.contains("img-src 'self' data: blob:"));
    }

    #[test]
    fn csp_extra_img_src() {
        let policy = build_content_security_policy(
            Some("n1"),
            false,
            &CspConfig {
                enabled: true,
                img_src: vec!["https://images.pexels.com".into()],
                ..CspConfig::from_env()
            },
        );
        assert!(policy.contains("img-src 'self' data: blob: https://images.pexels.com"));
    }

    #[test]
    fn csp_omitted_when_disabled() {
        let _guard = TEST_LOCK.lock().unwrap();
        configure(SecurityConfig {
            csp: CspConfig::disabled(),
            ..SecurityConfig::from_env()
        });
        let res = Response::new(axum::body::Body::empty());
        let res = apply_security_headers(
            res,
            &SecurityHeaderOptions {
                csp_nonce: Some("abc".into()),
                https: false,
            },
        );
        assert!(res.headers().get(header::CONTENT_SECURITY_POLICY).is_none());
    }

    #[test]
    fn validate_config_rejects_trust_proxy_without_cidrs() {
        let cfg = SecurityConfig {
            production: true,
            trust_proxy: true,
            ..SecurityConfig::from_env()
        };
        assert!(validate_config(&cfg).is_err());
    }

    #[test]
    fn validate_config_rejects_trust_proxy_in_dev_without_cidrs() {
        let cfg = SecurityConfig {
            production: false,
            trust_proxy: true,
            ..SecurityConfig::from_env()
        };
        assert!(validate_config(&cfg).is_err());
    }

    #[test]
    fn guard_mutation_strict_origin_when_csrf_enabled() {
        configure(SecurityConfig {
            csrf: true,
            origin_check: true,
            production: false,
            actions_per_minute: 10_000,
            submits_per_minute: 10_000,
            ..SecurityConfig::from_env()
        });
        let token = "0123456789abcdef0123456789abcdef";
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(&format!("__resuma-csrf={token}")).unwrap(),
        );
        headers.insert("x-resuma-csrf", HeaderValue::from_str(token).unwrap());
        assert!(matches!(
            guard_mutation(&headers, "localhost", "127.0.0.1", "test", 100, None),
            Err(ResumaError::Forbidden(_))
        ));
    }

    #[test]
    fn validate_action_name_rejects_empty() {
        assert!(validate_action_name("").is_err());
        assert!(validate_action_name("echo").is_ok());
    }

    #[test]
    fn csrf_cookie_rejects_short_or_non_hex_tokens() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("__resuma-csrf=0123456789abcdef"),
        );
        assert!(csrf_from_cookie(&headers).is_none());
    }

    #[test]
    fn validate_submit_name_matches_action_rules() {
        assert!(validate_submit_name("login").is_ok());
        assert!(validate_submit_name("../evil").is_err());
    }

    #[test]
    fn resolve_page_csrf_fails_closed_when_rng_unavailable() {
        set_rng_force_fail(true);
        let headers = HeaderMap::new();
        configure(SecurityConfig {
            csrf: true,
            ..SecurityConfig::from_env()
        });
        let err = resolve_page_csrf(&headers, true).unwrap_err();
        assert!(matches!(err, ResumaError::ServiceUnavailable(_)));
        set_rng_force_fail(false);
    }

    #[test]
    fn resolve_page_csrf_reuses_session_cookie_when_rng_unavailable() {
        set_rng_force_fail(true);
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("__resuma-csrf=0123456789abcdef0123456789abcdef"),
        );
        configure(SecurityConfig {
            csrf: true,
            ..SecurityConfig::from_env()
        });
        let (token, is_new) = resolve_page_csrf(&headers, true).expect("reuse cookie");
        assert_eq!(token, "0123456789abcdef0123456789abcdef");
        assert!(!is_new);
        set_rng_force_fail(false);
    }

    #[test]
    fn resolve_page_csp_nonce_fails_when_csp_enabled_and_rng_unavailable() {
        set_rng_force_fail(true);
        let err = resolve_page_csp_nonce(true).unwrap_err();
        assert!(matches!(err, ResumaError::ServiceUnavailable(_)));
        set_rng_force_fail(false);
        assert!(resolve_page_csp_nonce(false).unwrap().is_empty());
    }
}
