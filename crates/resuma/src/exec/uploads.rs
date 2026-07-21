//! Ephemeral user uploads (multipart) — distinct from trusted `public/`.
//!
//! * Anonymous store: `POST /_resuma/upload` → [`UploadReceipt`]
//! * Named handlers: `#[upload]` → `POST /_resuma/upload/{name}`

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{Result, ResumaError};
use crate::server::security::random_token;

const UPLOAD_TTL: Duration = Duration::from_secs(1800);
const UPLOAD_MAX: usize = 16;

/// Default max single upload (override with `RESUMA_UPLOAD_MAX_BYTES`).
fn max_upload_bytes() -> usize {
    std::env::var("RESUMA_UPLOAD_MAX_BYTES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8 * 1024 * 1024)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadReceipt {
    pub id: String,
    pub content_type: String,
    pub bytes: usize,
    pub url: String,
}

/// In-memory multipart payload passed to `#[upload]` handlers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedFile {
    pub bytes: Vec<u8>,
    pub content_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UploadMeta {
    pub max_bytes: usize,
    /// Empty = accept any content type.
    pub mime: Vec<String>,
}

pub type UploadFuture = Pin<Box<dyn Future<Output = Result<Value>> + Send>>;
pub type UploadFn = fn(UploadedFile) -> UploadFuture;

struct UploadEntry {
    meta: UploadMeta,
    run: UploadFn,
}

static HANDLERS: Lazy<RwLock<HashMap<String, UploadEntry>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Register a named `#[upload]` handler (called from macro-generated ctor).
pub fn register_upload(name: &str, meta: UploadMeta, run: UploadFn) {
    HANDLERS
        .write()
        .insert(name.to_string(), UploadEntry { meta, run });
}

pub fn has_registered_uploads() -> bool {
    !HANDLERS.read().is_empty()
}

pub fn lookup_upload(name: &str) -> Option<(UploadMeta, UploadFn)> {
    HANDLERS.read().get(name).map(|e| (e.meta.clone(), e.run))
}

struct CachedUpload {
    bytes: Vec<u8>,
    content_type: String,
    at: Instant,
}

fn cache() -> &'static Mutex<HashMap<String, CachedUpload>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CachedUpload>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn prune(map: &mut HashMap<String, CachedUpload>) {
    let now = Instant::now();
    map.retain(|_, v| now.duration_since(v.at) < UPLOAD_TTL);
    while map.len() > UPLOAD_MAX {
        let oldest = map.iter().min_by_key(|(_, v)| v.at).map(|(k, _)| k.clone());
        if let Some(k) = oldest {
            map.remove(&k);
        } else {
            break;
        }
    }
}

pub fn store(bytes: Vec<u8>, content_type: &str) -> Result<UploadReceipt> {
    store_with_limit(bytes, content_type, max_upload_bytes())
}

pub fn store_with_limit(bytes: Vec<u8>, content_type: &str, max: usize) -> Result<UploadReceipt> {
    if bytes.is_empty() {
        return Err(ResumaError::Validation("empty upload".into()));
    }
    if bytes.len() > max {
        return Err(ResumaError::Validation(format!(
            "upload too large ({} > {} bytes)",
            bytes.len(),
            max
        )));
    }
    let id = format!("u_{}", random_token().chars().take(20).collect::<String>());
    let ctype = if content_type.is_empty() {
        "application/octet-stream"
    } else {
        content_type
    };
    let len = bytes.len();
    let mut map = cache().lock().expect("upload cache");
    prune(&mut map);
    map.insert(
        id.clone(),
        CachedUpload {
            bytes,
            content_type: ctype.to_string(),
            at: Instant::now(),
        },
    );
    Ok(UploadReceipt {
        id: id.clone(),
        content_type: ctype.to_string(),
        bytes: len,
        url: format!("/_resuma/uploads/{id}"),
    })
}

pub fn take(id: &str) -> Option<(Vec<u8>, String)> {
    let mut map = cache().lock().expect("upload cache");
    prune(&mut map);
    map.get(id)
        .map(|c| (c.bytes.clone(), c.content_type.clone()))
}

/// Validate size + optional MIME allow-list for a named upload handler.
pub fn validate_uploaded(file: &UploadedFile, meta: &UploadMeta) -> Result<()> {
    if file.bytes.is_empty() {
        return Err(ResumaError::Validation("empty upload".into()));
    }
    if file.bytes.len() > meta.max_bytes {
        return Err(ResumaError::Validation(format!(
            "upload too large ({} > {} bytes)",
            file.bytes.len(),
            meta.max_bytes
        )));
    }
    if !meta.mime.is_empty() {
        let ct = file.content_type.to_ascii_lowercase();
        let ok = meta.mime.iter().any(|m| {
            let m = m.to_ascii_lowercase();
            ct == m || ct.starts_with(&format!("{m};"))
        });
        if !ok {
            return Err(ResumaError::Validation(format!(
                "content type `{}` not allowed",
                file.content_type
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_fetch() {
        let r = store(b"png-bytes".to_vec(), "image/png").unwrap();
        let (b, c) = take(&r.id).unwrap();
        assert_eq!(c, "image/png");
        assert_eq!(b, b"png-bytes");
    }

    #[test]
    fn mime_allow_list() {
        let meta = UploadMeta {
            max_bytes: 1024,
            mime: vec!["image/png".into()],
        };
        let ok = UploadedFile {
            bytes: vec![1, 2, 3],
            content_type: "image/png".into(),
            filename: None,
        };
        assert!(validate_uploaded(&ok, &meta).is_ok());
        let bad = UploadedFile {
            bytes: vec![1],
            content_type: "image/jpeg".into(),
            filename: None,
        };
        assert!(validate_uploaded(&bad, &meta).is_err());
    }
}
