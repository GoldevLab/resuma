//! File-backed rate limiting — multi-process safe without Redis.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;

use crate::core::{Result, ResumaError};

use super::rate_limit::RateLimitBackend;

static ROOT: once_cell::sync::Lazy<Mutex<Option<PathBuf>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

/// Configure disk rate-limit root (e.g. `.resuma/rate-limit`).
pub fn configure(root: impl AsRef<Path>) {
    let p = root.as_ref().to_path_buf();
    let _ = fs::create_dir_all(&p);
    *ROOT.lock() = Some(p);
}

fn root_dir() -> PathBuf {
    ROOT
        .lock()
        .clone()
        .unwrap_or_else(|| PathBuf::from(".resuma/rate-limit"))
}

fn key_path(key: &str) -> PathBuf {
    let safe: String = key
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    root_dir().join(format!("{safe}.json"))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct BucketFile {
    timestamps_ms: Vec<u64>,
}

pub struct DiskBackend;

impl RateLimitBackend for DiskBackend {
    fn check(&self, key: &str, limit_per_minute: u32, window: Duration) -> Result<()> {
        disk_check(key, limit_per_minute, window)
    }
}

fn disk_check(key: &str, limit_per_minute: u32, window: Duration) -> Result<()> {
    if limit_per_minute == 0 {
        return Ok(());
    }
    let path = key_path(key);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let window_ms = window.as_millis() as u64;
    let now = now_ms();
    let cutoff = now.saturating_sub(window_ms);

    let mut bucket = read_bucket(&path);
    bucket.timestamps_ms.retain(|t| *t > cutoff);

    if bucket.timestamps_ms.len() as u32 >= limit_per_minute {
        return Err(ResumaError::RateLimited);
    }

    bucket.timestamps_ms.push(now);
    write_bucket(&path, &bucket)?;
    Ok(())
}

fn read_bucket(path: &Path) -> BucketFile {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_bucket(path: &Path, bucket: &BucketFile) -> Result<()> {
    let data = serde_json::to_string(bucket).map_err(ResumaError::Serde)?;
    let tmp = path.with_extension("json.tmp");
    {
        let mut f = fs::File::create(&tmp).map_err(ResumaError::Io)?;
        f.write_all(data.as_bytes()).map_err(ResumaError::Io)?;
        f.sync_all().map_err(ResumaError::Io)?;
    }
    fs::rename(&tmp, path).map_err(ResumaError::Io)?;
    Ok(())
}
