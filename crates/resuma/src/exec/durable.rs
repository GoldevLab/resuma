//! Durable storage — Resuma's own KV + checkpoint persistence (`.resuma/durable/`).

use std::fs;
use std::path::{Path, PathBuf};

use parking_lot::RwLock;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::core::{Result, ResumaError};

use super::types::{GraphId, GraphSnapshot, WorkerEvent};

static ROOT: RwLock<Option<PathBuf>> = RwLock::new(None);

/// Configure durable storage root (default: `.resuma/durable` under cwd).
pub fn configure(root: impl AsRef<Path>) {
    let p = root.as_ref().to_path_buf();
    let _ = fs::create_dir_all(&p);
    *ROOT.write() = Some(p);
}

fn root_dir() -> PathBuf {
    ROOT.read()
        .clone()
        .unwrap_or_else(|| PathBuf::from(".resuma/durable"))
}

fn key_path(namespace: &str, key: &str) -> PathBuf {
    let safe_ns = sanitize(namespace);
    let safe_key = sanitize(key);
    root_dir().join(safe_ns).join(format!("{safe_key}.json"))
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Get a JSON value from durable storage.
pub fn get(namespace: &str, key: &str) -> Option<Value> {
    let path = key_path(namespace, key);
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Set a JSON value in durable storage (atomic: temp file + fsync + rename).
pub fn set(namespace: &str, key: &str, value: &Value) -> Result<()> {
    use std::io::Write;
    let path = key_path(namespace, key);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(ResumaError::Io)?;
    }
    let data = serde_json::to_string_pretty(value)?;
    let tmp = path.with_extension("json.tmp");
    {
        let mut f = fs::File::create(&tmp).map_err(ResumaError::Io)?;
        f.write_all(data.as_bytes()).map_err(ResumaError::Io)?;
        f.sync_all().map_err(ResumaError::Io)?;
    }
    fs::rename(&tmp, &path).map_err(ResumaError::Io)
}

/// Typed get/set helpers.
pub fn get_typed<T: DeserializeOwned>(namespace: &str, key: &str) -> Result<Option<T>> {
    match get(namespace, key) {
        Some(v) => Ok(Some(serde_json::from_value(v)?)),
        None => Ok(None),
    }
}

pub fn set_typed<T: Serialize>(namespace: &str, key: &str, value: &T) -> Result<()> {
    set(namespace, key, &serde_json::to_value(value)?)
}

const GRAPHS_NS: &str = "graphs";
const EVENTS_NS: &str = "events";
const CHECKPOINTS_NS: &str = "checkpoints";

/// Persist graph snapshot for replay across restarts.
pub fn persist_graph(snapshot: &GraphSnapshot) -> Result<()> {
    set(GRAPHS_NS, &snapshot.id.0, &serde_json::to_value(snapshot)?)
}

pub fn load_graph(id: &GraphId) -> Option<GraphSnapshot> {
    get(GRAPHS_NS, &id.0).and_then(|v| serde_json::from_value(v).ok())
}

/// Append-only event log on disk.
pub fn persist_events(id: &GraphId, events: &[WorkerEvent]) -> Result<()> {
    set(EVENTS_NS, &id.0, &serde_json::to_value(events)?)
}

pub fn load_events(id: &GraphId) -> Option<Vec<WorkerEvent>> {
    get(EVENTS_NS, &id.0).and_then(|v| serde_json::from_value(v).ok())
}

/// Checkpoint worker state mid-execution.
pub fn save_checkpoint(id: &GraphId, state: &super::state::StateStore) -> Result<()> {
    set(
        CHECKPOINTS_NS,
        &id.0,
        &serde_json::to_value(state.snapshot())?,
    )
}

pub fn load_checkpoint(id: &GraphId) -> Option<super::state::StateStore> {
    let map = get(CHECKPOINTS_NS, &id.0)?;
    let store = super::state::StateStore::default();
    if let Some(obj) = map.as_object() {
        for (k, v) in obj {
            store.set(k.clone(), v.clone());
        }
    }
    Some(store)
}

const EXECUTIONS_NS: &str = "executions";

/// Persisted execution metadata for pause/resume across restarts.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionRecord {
    pub graph_id: GraphId,
    pub worker: String,
    pub input: Value,
    pub plan: super::types::ExecutionPlan,
    pub profile: super::resources::ResourceProfile,
    pub paused: bool,
    /// Hard cancel — graph must not be resumed (distinct from cooperative pause).
    #[serde(default)]
    pub cancelled: bool,
}

pub fn save_execution_record(record: &ExecutionRecord) -> Result<()> {
    set(
        EXECUTIONS_NS,
        &record.graph_id.0,
        &serde_json::to_value(record)?,
    )
}

pub fn load_execution_record(id: &GraphId) -> Option<ExecutionRecord> {
    get(EXECUTIONS_NS, &id.0).and_then(|v| serde_json::from_value(v).ok())
}

const TOKENS_NS: &str = "tokens";

/// Persist graph-scoped access token for SSE / UI controls.
pub fn save_graph_token(id: &GraphId, token: &str) -> Result<()> {
    set(TOKENS_NS, &id.0, &serde_json::json!({ "token": token }))
}

pub fn load_graph_token(id: &GraphId) -> Option<String> {
    get(TOKENS_NS, &id.0).and_then(|v| v.get("token").and_then(|t| t.as_str()).map(str::to_string))
}
