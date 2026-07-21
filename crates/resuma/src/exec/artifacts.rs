//! In-process / on-disk artifact store for large worker results
//! (meshes, images, binaries) that must not live inline in durable graph JSON.
//!
//! Artifacts created via [`WorkerContext`](super::workers::WorkerContext) are bound
//! to a graph id; `GET /_resuma/artifact/{id}` then requires that graph's access
//! token (or a strict exec API key).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::core::{Result, ResumaError};
use crate::server::security::random_token;

use super::types::GraphId;

const ARTIFACT_TTL: Duration = Duration::from_secs(3600);
const ARTIFACT_MAX: usize = 64;
const ARTIFACT_MAX_BYTES_DEFAULT: usize = 256 * 1024 * 1024;

fn max_artifact_bytes() -> usize {
    std::env::var("RESUMA_ARTIFACT_MAX_BYTES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(ARTIFACT_MAX_BYTES_DEFAULT)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub id: String,
    pub content_type: String,
    pub bytes: usize,
    /// When set, retrieval requires this graph's token (or admin API key).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_id: Option<String>,
}

struct CachedArtifact {
    bytes: Arc<Vec<u8>>,
    content_type: String,
    graph_id: Option<String>,
    at: Instant,
}

fn cache() -> &'static Mutex<HashMap<String, CachedArtifact>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CachedArtifact>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn artifact_dir() -> PathBuf {
    let root = std::env::var("RESUMA_DATA_DIR").unwrap_or_else(|_| ".resuma".into());
    PathBuf::from(root).join("artifacts")
}

fn prune(map: &mut HashMap<String, CachedArtifact>) {
    let now = Instant::now();
    map.retain(|_, v| now.duration_since(v.at) < ARTIFACT_TTL);
    while map.len() > ARTIFACT_MAX {
        let oldest = map.iter().min_by_key(|(_, v)| v.at).map(|(k, _)| k.clone());
        if let Some(k) = oldest {
            map.remove(&k);
        } else {
            break;
        }
    }
}

/// Store bytes without graph binding (capability URL: unguessable id only).
pub fn put(bytes: Vec<u8>, content_type: &str) -> Result<ArtifactRef> {
    put_bound(bytes, content_type, None)
}

/// Store bytes bound to an owning graph (authorized retrieval).
pub fn put_bound(
    bytes: Vec<u8>,
    content_type: &str,
    graph_id: Option<&GraphId>,
) -> Result<ArtifactRef> {
    if bytes.is_empty() {
        return Err(ResumaError::Validation("empty artifact".into()));
    }
    let max = max_artifact_bytes();
    if bytes.len() > max {
        return Err(ResumaError::Validation(format!(
            "artifact too large ({} > {} bytes)",
            bytes.len(),
            max
        )));
    }
    let id = format!("a_{}", random_token().chars().take(24).collect::<String>());
    let ctype = if content_type.is_empty() {
        "application/octet-stream"
    } else {
        content_type
    };
    let bound = graph_id.map(|g| g.0.clone());

    let aref = ArtifactRef {
        id: id.clone(),
        content_type: ctype.to_string(),
        bytes: bytes.len(),
        graph_id: bound.clone(),
    };

    // Best-effort disk mirror for process restarts within TTL window.
    let dir = artifact_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("{id}.bin"));
    let _ = std::fs::write(&path, &bytes);
    let meta_path = dir.join(format!("{id}.json"));
    let _ = std::fs::write(&meta_path, serde_json::to_string(&aref).unwrap_or_default());

    let len = bytes.len();
    let mut map = cache().lock().expect("artifact cache");
    prune(&mut map);
    map.insert(
        id.clone(),
        CachedArtifact {
            bytes: Arc::new(bytes),
            content_type: ctype.to_string(),
            graph_id: bound,
            at: Instant::now(),
        },
    );
    Ok(ArtifactRef {
        id,
        content_type: ctype.to_string(),
        bytes: len,
        graph_id: aref.graph_id,
    })
}

/// Store a JSON-serializable value as `application/json`.
pub fn put_json<T: Serialize>(value: &T) -> Result<ArtifactRef> {
    let bytes = serde_json::to_vec(value)
        .map_err(|e| ResumaError::Validation(format!("artifact json encode: {e}")))?;
    put(bytes, "application/json")
}

pub fn put_json_bound<T: Serialize>(value: &T, graph_id: &GraphId) -> Result<ArtifactRef> {
    let bytes = serde_json::to_vec(value)
        .map_err(|e| ResumaError::Validation(format!("artifact json encode: {e}")))?;
    put_bound(bytes, "application/json", Some(graph_id))
}

/// Returns `(bytes, content_type, bound_graph_id)`.
pub fn get(id: &str) -> Option<(Arc<Vec<u8>>, String, Option<String>)> {
    {
        let mut map = cache().lock().expect("artifact cache");
        prune(&mut map);
        if let Some(c) = map.get(id) {
            return Some((c.bytes.clone(), c.content_type.clone(), c.graph_id.clone()));
        }
    }
    // Disk fallback.
    let dir = artifact_dir();
    let path = dir.join(format!("{id}.bin"));
    let meta_path = dir.join(format!("{id}.json"));
    let bytes = std::fs::read(&path).ok()?;
    let aref = std::fs::read_to_string(&meta_path)
        .ok()
        .and_then(|s| serde_json::from_str::<ArtifactRef>(&s).ok());
    let ctype = aref
        .as_ref()
        .map(|r| r.content_type.clone())
        .unwrap_or_else(|| "application/octet-stream".into());
    let bound = aref.and_then(|r| r.graph_id);
    let arc = Arc::new(bytes);
    let mut map = cache().lock().expect("artifact cache");
    map.insert(
        id.to_string(),
        CachedArtifact {
            bytes: arc.clone(),
            content_type: ctype.clone(),
            graph_id: bound.clone(),
            at: Instant::now(),
        },
    );
    Some((arc, ctype, bound))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_and_get_roundtrip() {
        let r = put(b"hello-mesh".to_vec(), "text/plain").unwrap();
        assert!(r.id.starts_with("a_"));
        assert!(r.graph_id.is_none());
        let (bytes, ctype, bound) = get(&r.id).expect("artifact");
        assert_eq!(ctype, "text/plain");
        assert!(bound.is_none());
        assert_eq!(bytes.as_slice(), b"hello-mesh");
    }

    #[test]
    fn put_bound_stores_graph() {
        let gid = GraphId("g_testartifactbind".into());
        let r = put_bound(b"secret".to_vec(), "application/octet-stream", Some(&gid)).unwrap();
        assert_eq!(r.graph_id.as_deref(), Some("g_testartifactbind"));
        let (_, _, bound) = get(&r.id).unwrap();
        assert_eq!(bound.as_deref(), Some("g_testartifactbind"));
    }
}
