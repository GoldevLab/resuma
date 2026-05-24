//! Security wiring for the todo showcase — production backend patterns in one module.
//!
//! Covers: session context, authorization, validation, audit logging, API keys,
//! See docs site → `/docs/security/todo`.

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};

use resuma::prelude::*;
use resuma::ResumaError;
use resuma::SecurityConfig;
use serde_json::json;

/// Max task title length (validation pipe).
pub const MAX_TITLE_LEN: usize = 120;

/// Max tasks stored in memory (DoS guard for demo server).
pub const MAX_TODOS: usize = 200;

/// Demo session cookie — production apps use httpOnly + Secure flags.
pub const DEMO_USER_COOKIE: &str = "resuma_demo_user";

pub const DEFAULT_USER: &str = "guest";

static ACTION_CALLS: AtomicU64 = AtomicU64::new(0);

/// Register action middleware and rely on framework CSRF / rate limits / headers.
pub fn install() {
    set_action_middleware(action_pipeline);
}

fn action_pipeline(
    req: FlowRequest,
) -> Pin<Box<dyn Future<Output = std::result::Result<FlowRequest, ResumaError>> + Send>> {
    Box::pin(async move {
        let req = attach_request_id(req);
        let req = attach_session(req)?;
        audit_action(&req);

        if let Ok(expected) = std::env::var("RESUMA_TODO_API_KEY") {
            if !expected.is_empty() {
                let got = req.header("x-resuma-demo-key").unwrap_or("");
                if got != expected {
                    return Err(ResumaError::Unauthorized);
                }
            }
        }

        Ok(req)
    })
}

/// Resolve demo user from cookie and inject auth extensions (NestJS Guard equivalent).
pub fn attach_session(mut req: FlowRequest) -> std::result::Result<FlowRequest, ResumaError> {
    let user = session_user(&req);
    if !is_allowed_user(&user) {
        return Err(ResumaError::Unauthorized);
    }

    let mut roles = vec!["user".to_string()];
    if admin_users().iter().any(|a| a == &user) {
        roles.push("admin".into());
    }

    req.set_extension("authenticated", json!(true));
    req.set_extension("user_id", json!(user));
    req.set_extension("roles", json!(roles));
    Ok(req)
}

pub fn session_user(req: &FlowRequest) -> String {
    cookie_value(req.header("cookie"), DEMO_USER_COOKIE)
        .filter(|u| is_allowed_user(u))
        .unwrap_or_else(|| DEFAULT_USER.into())
}

fn cookie_value(cookie: Option<&str>, key: &str) -> Option<String> {
    cookie.and_then(|raw| {
        raw.split(';').find_map(|part| {
            let (k, v) = part.split_once('=')?;
            if k.trim() == key {
                Some(v.trim().to_string())
            } else {
                None
            }
        })
    })
}

pub fn demo_users() -> &'static [&'static str] {
    &["guest", "alice", "bob"]
}

pub fn is_allowed_user(user: &str) -> bool {
    !user.is_empty()
        && user.len() <= 32
        && user
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        && demo_users().contains(&user)
}

pub fn admin_users() -> Vec<String> {
    std::env::var("RESUMA_TODO_ADMINS")
        .ok()
        .map(|s| {
            s.split(',')
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_else(|| vec!["alice".into()])
}

pub fn client_ip(req: &FlowRequest) -> &str {
    req.header("x-forwarded-for")
        .or_else(|| req.header("x-real-ip"))
        .unwrap_or("unknown")
}

/// NestJS LoggingInterceptor / Next.js middleware — correlate logs per request.
pub fn attach_request_id(mut req: FlowRequest) -> FlowRequest {
    let id = req
        .header("x-request-id")
        .map(String::from)
        .unwrap_or_else(|| {
            let n = ACTION_CALLS.load(Ordering::Relaxed);
            format!("todo-{n:06}")
        });
    req.set_extension("request_id", json!(id));
    req
}

pub fn request_id(req: &FlowRequest) -> Option<&str> {
    req.extension("request_id").and_then(|v| v.as_str())
}

fn audit_action(req: &FlowRequest) {
    let n = ACTION_CALLS.fetch_add(1, Ordering::Relaxed) + 1;
    let user = req.user_id().unwrap_or("?");
    let rid = request_id(req).unwrap_or("-");
    println!(
        "[todo:security] #{n} rid={rid} {} {} user={user} ip={}",
        req.method,
        req.path,
        client_ip(req)
    );
}

pub fn serve_options() -> ServeOptions {
    ServeOptions {
        addr: bind_addr(),
        security: SecurityConfig {
            csrf: true,
            origin_check: true,
            trust_proxy: matches!(
                std::env::var("RESUMA_TRUST_PROXY").as_deref(),
                Ok("1") | Ok("true") | Ok("TRUE")
            ),
            body_limit_bytes: 256 * 1024,
            actions_per_minute: 90,
            submits_per_minute: 45,
            hide_benchmark: true,
            production: matches!(
                std::env::var("RESUMA_ENV").as_deref(),
                Ok("production") | Ok("prod")
            ),
        },
    }
}

fn bind_addr() -> SocketAddr {
    if let Ok(raw) = std::env::var("RESUMA_ADDR") {
        if let Ok(addr) = raw.parse() {
            return addr;
        }
    }
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    format!("{host}:{port}")
        .parse()
        .unwrap_or_else(|_| SocketAddr::from(([127, 0, 0, 1], port)))
}

// ── Validation (pipe equivalent) ─────────────────────────────────────────────

pub fn normalize_title(title: &str) -> std::result::Result<String, ResumaError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(ResumaError::Other("title required".into()));
    }
    if title.len() > MAX_TITLE_LEN {
        return Err(ResumaError::Other(format!(
            "title max {MAX_TITLE_LEN} chars"
        )));
    }
    Ok(title.to_string())
}

pub fn valid_id(id: u64) -> std::result::Result<(), ResumaError> {
    if id == 0 {
        return Err(ResumaError::Other("invalid id".into()));
    }
    Ok(())
}

pub fn can_add_todo(current_len: usize) -> std::result::Result<(), ResumaError> {
    if current_len >= MAX_TODOS {
        return Err(ResumaError::Forbidden(format!("max {MAX_TODOS} tasks")));
    }
    Ok(())
}

// ── Authorization (ACL equivalent) ─────────────────────────────────────────

pub fn require_user(req: &FlowRequest) -> std::result::Result<&str, ResumaError> {
    req.user_id().ok_or(ResumaError::Unauthorized)
}

pub fn assert_owner(owner_id: &str, req: &FlowRequest) -> std::result::Result<(), ResumaError> {
    let uid = require_user(req)?;
    if owner_id != uid && !req.has_role("admin") {
        return Err(ResumaError::Forbidden("not your task".into()));
    }
    Ok(())
}
