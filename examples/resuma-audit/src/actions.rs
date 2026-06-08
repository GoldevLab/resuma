//! Shared server actions and submits for audit demos.

use resuma::prelude::*;
use serde::{Deserialize, Serialize};

#[server]
async fn audit_echo(msg: String) -> String {
    format!("Echo: {msg}")
}

#[server]
async fn audit_add(a: i32, b: i32) -> i32 {
    a + b
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreetForm {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreetResult {
    pub message: String,
}

#[submit]
pub async fn audit_greet(
    data: GreetForm,
    _req: &FlowRequest,
) -> std::result::Result<GreetResult, SubmitError> {
    if data.name.trim().is_empty() {
        return Err(SubmitError::new("Invalid input").field("name", "Name required"));
    }
    Ok(GreetResult {
        message: format!("Hello, {}!", data.name.trim()),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchData {
    pub query: String,
    pub results: Vec<String>,
}

#[load]
pub async fn audit_search(req: &FlowRequest) -> SearchData {
    let q = req.query_param("q").unwrap_or("").trim().to_string();
    let results = if q.len() >= 2 {
        vec![
            format!("Result A for '{q}'"),
            format!("Result B for '{q}'"),
            format!("Result C for '{q}'"),
        ]
    } else {
        vec![]
    };
    SearchData { query: q, results }
}

fn audit_delayed_stream_view(data: &str) -> View {
    view! { <p>{data.to_string()}</p> }
}

#[load(stream, cache = "public, max-age=30")]
pub async fn audit_delayed(_req: &FlowRequest) -> String {
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
    "Streamed after 800ms delay".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedData {
    pub value: String,
    pub timestamp: String,
}

#[load(cache = "public, max-age=60")]
pub async fn audit_cached(_req: &FlowRequest) -> CachedData {
    CachedData {
        value: "Cached loader response".into(),
        timestamp: chrono_now(),
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrgForm {
    pub item: String,
}

#[submit]
pub async fn audit_prg(
    data: PrgForm,
    _req: &FlowRequest,
) -> std::result::Result<Redirect, SubmitError> {
    if data.item.trim().is_empty() {
        return Err(SubmitError::new("Invalid").field("item", "Item required"));
    }
    Ok(redirect(format!(
        "/audit/cookbook/prg?added={}",
        urlencoding_simple(data.item.trim())
    )))
}

fn urlencoding_simple(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "+".into(),
            c if c.is_ascii_alphanumeric() || c == '-' || c == '_' => c.to_string(),
            c => format!("%{:02X}", c as u8),
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
}

#[load]
pub async fn audit_user(req: &FlowRequest) -> std::result::Result<UserProfile, LoaderError> {
    let id = req.param("id").unwrap_or("0");
    if id == "404" {
        return Err(LoaderError::new(404, "User not found"));
    }
    Ok(UserProfile {
        id: id.to_string(),
        name: format!("User #{id}"),
    })
}
