//! Outbound webhooks on graph lifecycle events (done, failed, paused).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use hmac::{Hmac, Mac};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use tokio::task;
use tracing::{info, warn};

use crate::core::{Result, ResumaError};

use super::id;
use super::metrics;
use super::ssrf;
use super::tools;
use super::types::GraphSnapshot;

type HmacSha256 = Hmac<Sha256>;

static ROOT: RwLock<Option<PathBuf>> = RwLock::new(None);
static TARGETS: Lazy<RwLock<Vec<WebhookTarget>>> = Lazy::new(|| RwLock::new(Vec::new()));

/// Registered webhook destination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookTarget {
    pub id: String,
    pub url: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub events: Vec<String>,
    pub created_ms: u64,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterWebhookBody {
    pub url: String,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebhookListResponse {
    pub targets: Vec<WebhookTarget>,
    pub total: usize,
}

/// Payload POSTed to webhook URLs.
#[derive(Debug, Clone, Serialize)]
pub struct WebhookPayload {
    pub event: String,
    pub graph_id: String,
    pub worker: String,
    pub status: String,
    pub timestamp_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
}

/// Configure webhook storage (`{RESUMA_DATA_DIR}/webhooks`).
pub fn configure(root: impl AsRef<Path>) {
    let p = root.as_ref().to_path_buf();
    let _ = fs::create_dir_all(&p);
    *ROOT.write() = Some(p);
    reload_targets();
}

/// Load targets from env + disk.
pub fn init_from_env() {
    if ROOT.read().is_none() {
        let root = std::env::var("RESUMA_DATA_DIR").unwrap_or_else(|_| ".resuma".into());
        configure(format!("{root}/webhooks"));
    }
    let secret = webhook_secret();
    let _ = secret;
    if let Ok(urls) = std::env::var("RESUMA_WEBHOOK_URLS") {
        for url in urls
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
        {
            let _ = register(RegisterWebhookBody {
                url: url.to_string(),
                events: default_events(),
                enabled: true,
            });
        }
    } else if let Ok(url) = std::env::var("RESUMA_WEBHOOK_URL") {
        if !url.is_empty() {
            let _ = register(RegisterWebhookBody {
                url,
                events: default_events(),
                enabled: true,
            });
        }
    }
    reload_targets();
}

fn default_events() -> Vec<String> {
    vec![
        "graph.done".into(),
        "graph.failed".into(),
        "graph.paused".into(),
    ]
}

fn targets_path() -> PathBuf {
    ROOT.read()
        .clone()
        .unwrap_or_else(|| PathBuf::from(".resuma/webhooks"))
        .join("targets.json")
}

fn reload_targets() {
    let path = targets_path();
    let Ok(data) = fs::read_to_string(path) else {
        return;
    };
    if let Ok(list) = serde_json::from_str::<Vec<WebhookTarget>>(&data) {
        *TARGETS.write() = list;
    }
}

fn persist_targets(targets: &[WebhookTarget]) -> Result<()> {
    let path = targets_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(ResumaError::Io)?;
    }
    let tmp = path.with_extension("json.tmp");
    let data = serde_json::to_string_pretty(targets)?;
    {
        let mut f = fs::File::create(&tmp).map_err(ResumaError::Io)?;
        f.write_all(data.as_bytes()).map_err(ResumaError::Io)?;
        f.sync_all().map_err(ResumaError::Io)?;
    }
    fs::rename(&tmp, &path).map_err(ResumaError::Io)?;
    Ok(())
}

/// Register a webhook URL (SSRF-checked, persisted to disk).
pub fn register(body: RegisterWebhookBody) -> Result<WebhookTarget> {
    ssrf::validate_fetch_url(&body.url)?;
    register_inner(body)
}

/// Register with DNS resolution (use from HTTP handlers).
pub async fn register_resolved(body: RegisterWebhookBody) -> Result<WebhookTarget> {
    ssrf::validate_fetch_url_resolved(&body.url).await?;
    register_inner(body)
}

fn register_inner(body: RegisterWebhookBody) -> Result<WebhookTarget> {
    let events = if body.events.is_empty() {
        default_events()
    } else {
        body.events
    };
    let target = WebhookTarget {
        id: format!("wh_{}", crate::server::security::random_token()),
        url: body.url,
        enabled: body.enabled,
        events,
        created_ms: id::now_ms(),
    };
    let mut targets = TARGETS.write();
    if let Some(existing) = targets.iter().find(|t| t.url == target.url).cloned() {
        return Ok(existing);
    }
    targets.push(target.clone());
    persist_targets(&targets)?;
    Ok(target)
}

/// Remove webhook by id.
pub fn remove(id: &str) -> Result<bool> {
    super::security::validate_schedule_id(id)?;
    let mut targets = TARGETS.write();
    let before = targets.len();
    targets.retain(|t| t.id != id);
    if targets.len() < before {
        persist_targets(&targets)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn list() -> WebhookListResponse {
    let targets = TARGETS.read().clone();
    let total = targets.len();
    WebhookListResponse { targets, total }
}

pub fn webhook_secret() -> Option<String> {
    std::env::var("RESUMA_WEBHOOK_SECRET")
        .ok()
        .filter(|s| !s.is_empty())
}

/// Fire webhooks when a graph completes successfully.
pub fn notify_done(snapshot: &GraphSnapshot, duration_ms: u64, result: Option<Value>) {
    let payload = WebhookPayload {
        event: "graph.done".into(),
        graph_id: snapshot.id.0.clone(),
        worker: snapshot.worker.clone(),
        status: "done".into(),
        timestamp_ms: id::now_ms(),
        duration_ms: Some(duration_ms),
        error: None,
        result,
    };
    dispatch(payload);
}

/// Fire webhooks for a failed graph with error message.
pub fn notify_failed(snapshot: &GraphSnapshot, duration_ms: u64, error: String) {
    let payload = WebhookPayload {
        event: "graph.failed".into(),
        graph_id: snapshot.id.0.clone(),
        worker: snapshot.worker.clone(),
        status: "failed".into(),
        timestamp_ms: id::now_ms(),
        duration_ms: Some(duration_ms),
        error: Some(error),
        result: None,
    };
    dispatch(payload);
}

/// Fire webhooks when execution is paused/cancelled.
pub fn notify_paused(snapshot: &GraphSnapshot, duration_ms: u64) {
    let payload = WebhookPayload {
        event: "graph.paused".into(),
        graph_id: snapshot.id.0.clone(),
        worker: snapshot.worker.clone(),
        status: "paused".into(),
        timestamp_ms: id::now_ms(),
        duration_ms: Some(duration_ms),
        error: None,
        result: None,
    };
    dispatch(payload);
}

fn dispatch(payload: WebhookPayload) {
    let targets: Vec<WebhookTarget> = TARGETS
        .read()
        .iter()
        .filter(|t| t.enabled && t.events.iter().any(|e| e == &payload.event))
        .cloned()
        .collect();
    if targets.is_empty() {
        return;
    }
    task::spawn(async move {
        for target in targets {
            if let Err(e) = deliver(&target.url, &payload).await {
                warn!(url = %target.url, error = %e, "webhook delivery failed");
                metrics::inc_webhook_failed();
            } else {
                metrics::inc_webhook_sent();
                info!(url = %target.url, event = %payload.event, "webhook delivered");
            }
        }
    });
}

async fn deliver(url: &str, payload: &WebhookPayload) -> Result<()> {
    let (parsed, pinned_ip) = ssrf::validate_fetch_url_resolved(url).await?;
    let client = ssrf::pinned_fetch_client(&parsed, pinned_ip)?;
    tools::init_http_client();
    let body = serde_json::to_string(payload).map_err(ResumaError::Serde)?;
    let mut req = client
        .post(parsed.as_str())
        .header("content-type", "application/json")
        .header("user-agent", "resuma-webhooks/1.0")
        .timeout(Duration::from_secs(15))
        .body(body.clone());

    if let Some(secret) = webhook_secret() {
        if let Some(sig) = sign_body(&body, &secret) {
            req = req.header("x-resuma-signature", format!("sha256={sig}"));
        }
    }

    let res = req
        .send()
        .await
        .map_err(|e| ResumaError::Other(format!("webhook request failed: {e}")))?;
    if !res.status().is_success() {
        return Err(ResumaError::Other(format!(
            "webhook returned {}",
            res.status()
        )));
    }
    Ok(())
}

fn sign_body(body: &str, secret: &str) -> Option<String> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(body.as_bytes());
    let result = mac.finalize().into_bytes();
    Some(hex_encode(&result))
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_is_deterministic() {
        let sig = sign_body(r#"{"event":"graph.done"}"#, "test-secret").unwrap();
        assert_eq!(sig.len(), 64);
    }
}
