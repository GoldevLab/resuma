//! File-backed rate limiting — multi-process safe without Redis.

use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use fs2::FileExt;
use parking_lot::Mutex;

use crate::core::{Result, ResumaError};

use super::rate_limit::RateLimitBackend;

static ROOT: once_cell::sync::Lazy<Mutex<Option<PathBuf>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

/// Max on-disk rate-limit bucket files before evicting oldest by mtime.
const DISK_BUCKET_FILE_CAP: usize = 50_000;

/// Configure disk rate-limit root (e.g. `.resuma/rate-limit`).
pub fn configure(root: impl AsRef<Path>) {
    let p = root.as_ref().to_path_buf();
    let _ = fs::create_dir_all(&p);
    *ROOT.lock() = Some(p);
}

fn root_dir() -> PathBuf {
    ROOT.lock()
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

fn maybe_evict_oldest_bucket_files() {
    let dir = root_dir();
    let Ok(read_dir) = fs::read_dir(&dir) else {
        return;
    };
    let mut files: Vec<(PathBuf, SystemTime)> = read_dir
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .filter_map(|e| {
            let modified = e.metadata().ok()?.modified().ok()?;
            Some((e.path(), modified))
        })
        .collect();
    if files.len() < DISK_BUCKET_FILE_CAP {
        return;
    }
    files.sort_by_key(|(_, modified)| *modified);
    let to_remove = files.len().saturating_sub(DISK_BUCKET_FILE_CAP / 2);
    for (path, _) in files.into_iter().take(to_remove) {
        let _ = fs::remove_file(path);
    }
}

fn disk_check(key: &str, limit_per_minute: u32, window: Duration) -> Result<()> {
    if limit_per_minute == 0 {
        return Ok(());
    }
    let path = key_path(key);
    if !path.exists() {
        maybe_evict_oldest_bucket_files();
    }
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&path)
        .map_err(ResumaError::Io)?;

    file.lock_exclusive().map_err(ResumaError::Io)?;

    let result = (|| {
        let window_ms = window.as_millis() as u64;
        let now = now_ms();
        let cutoff = now.saturating_sub(window_ms);

        let mut bucket = read_bucket_file(&file);
        bucket.timestamps_ms.retain(|t| *t > cutoff);

        if bucket.timestamps_ms.len() as u32 >= limit_per_minute {
            return Err(ResumaError::RateLimited);
        }

        bucket.timestamps_ms.push(now);
        write_bucket_file(&mut file, &bucket)
    })();

    let _ = file.unlock();
    result
}

fn read_bucket_file(file: &fs::File) -> BucketFile {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = file;
    let _ = file.seek(SeekFrom::Start(0));
    let mut data = String::new();
    if file.read_to_string(&mut data).is_ok() {
        if let Ok(bucket) = serde_json::from_str(&data) {
            return bucket;
        }
    }
    BucketFile::default()
}

fn write_bucket_file(file: &mut fs::File, bucket: &BucketFile) -> Result<()> {
    let data = serde_json::to_string(bucket).map_err(ResumaError::Serde)?;
    file.set_len(0).map_err(ResumaError::Io)?;
    file.seek(SeekFrom::Start(0)).map_err(ResumaError::Io)?;
    file.write_all(data.as_bytes()).map_err(ResumaError::Io)?;
    file.sync_all().map_err(ResumaError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclusive_lock_serializes_updates() {
        let dir = std::env::temp_dir().join(format!("resuma-rl-{}", std::process::id()));
        configure(&dir);
        assert!(disk_check("test-key", 100, Duration::from_secs(60)).is_ok());
        assert!(disk_check("test-key", 100, Duration::from_secs(60)).is_ok());
        let _ = fs::remove_dir_all(dir);
    }
}
