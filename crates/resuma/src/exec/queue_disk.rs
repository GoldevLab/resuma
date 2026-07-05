//! File-backed job queue — multi-process safe via atomic rename (no Redis).
//!
//! Layout per queue:
//! ```text
//! .resuma/queue/{name}/
//!   pending/     ← new jobs (enqueue)
//!   processing/  ← claimed by exactly one Resuma process
//!   done/        ← finished
//!   failed/      ← could not start worker
//! ```
//!
//! **Claim protocol:** `rename(pending/id.json → processing/id.json)` is atomic on
//! the same filesystem — only one process wins per job.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use parking_lot::RwLock;

use crate::core::{Result, ResumaError};

use super::queue::QueueMessage;
use super::security::validate_schedule_id;

static QUEUE_ROOT: RwLock<Option<PathBuf>> = RwLock::new(None);

/// Configure on-disk queue root (default `.resuma/queue`).
pub fn configure(root: impl AsRef<Path>) {
    let p = root.as_ref().to_path_buf();
    let _ = fs::create_dir_all(&p);
    *QUEUE_ROOT.write() = Some(p);
}

pub fn root() -> PathBuf {
    QUEUE_ROOT
        .read()
        .clone()
        .unwrap_or_else(|| PathBuf::from(".resuma/queue"))
}

fn queue_dir(queue: &str) -> PathBuf {
    root().join(sanitize(queue))
}

fn pending_dir(queue: &str) -> PathBuf {
    queue_dir(queue).join("pending")
}

fn processing_dir(queue: &str) -> PathBuf {
    queue_dir(queue).join("processing")
}

fn done_dir(queue: &str) -> PathBuf {
    queue_dir(queue).join("done")
}

fn failed_dir(queue: &str) -> PathBuf {
    queue_dir(queue).join("failed")
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

fn pending_path(queue: &str, id: &str) -> Result<PathBuf> {
    validate_schedule_id(id)?;
    Ok(pending_dir(queue).join(format!("{id}.json")))
}

fn processing_path(queue: &str, id: &str) -> Result<PathBuf> {
    validate_schedule_id(id)?;
    Ok(processing_dir(queue).join(format!("{id}.json")))
}

/// Atomic enqueue: write temp file then rename into `pending/`.
pub fn persist_pending(queue: &str, msg: &QueueMessage) -> Result<()> {
    let dir = pending_dir(queue);
    fs::create_dir_all(&dir).map_err(ResumaError::Io)?;
    let final_path = pending_path(queue, &msg.id)?;
    let tmp_path = dir.join(format!("{}.json.tmp", msg.id));
    let data = serde_json::to_string_pretty(msg)?;
    {
        let mut file = fs::File::create(&tmp_path).map_err(ResumaError::Io)?;
        file.write_all(data.as_bytes()).map_err(ResumaError::Io)?;
        file.sync_all().map_err(ResumaError::Io)?;
    }
    fs::rename(&tmp_path, &final_path).map_err(ResumaError::Io)?;
    Ok(())
}

/// Try to claim a specific job (pending → processing). Fails if another process got it.
pub fn try_claim(queue: &str, id: &str) -> Result<()> {
    let src = pending_path(queue, id)?;
    if !src.exists() {
        return Err(ResumaError::Other("job already claimed or missing".into()));
    }
    fs::create_dir_all(processing_dir(queue)).map_err(ResumaError::Io)?;
    let dst = processing_path(queue, id)?;
    fs::rename(&src, &dst).map_err(|e| ResumaError::Other(format!("claim failed: {e}")))
}

/// Claim the oldest pending job, if any. Safe across multiple Resuma processes.
pub fn claim_next(queue: &str) -> Option<QueueMessage> {
    let dir = pending_dir(queue);
    let Ok(entries) = fs::read_dir(&dir) else {
        return None;
    };

    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if path.extension().and_then(|x| x.to_str()) != Some("json") {
                return None;
            }
            path.file_stem()?.to_str().map(str::to_string)
        })
        .collect();
    names.sort();

    for id in names {
        if validate_schedule_id(&id).is_err() {
            continue;
        }
        if try_claim(queue, &id).is_err() {
            continue;
        }
        let path = processing_path(queue, &id).ok()?;
        let data = fs::read_to_string(&path).ok()?;
        return serde_json::from_str(&data).ok();
    }
    None
}

/// Mark a claimed job finished (`processing/` → `done/` or `failed/`).
pub fn complete(queue: &str, id: &str, success: bool) -> Result<()> {
    let src = processing_path(queue, id)?;
    if !src.exists() {
        return Ok(());
    }
    let dest_dir = if success {
        done_dir(queue)
    } else {
        failed_dir(queue)
    };
    fs::create_dir_all(&dest_dir).map_err(ResumaError::Io)?;
    let dest = dest_dir.join(format!("{id}.json"));
    fs::rename(&src, &dest).or_else(|_| {
        let data = fs::read(&src).map_err(ResumaError::Io)?;
        fs::write(&dest, data).map_err(ResumaError::Io)?;
        fs::remove_file(&src).map_err(ResumaError::Io)
    })
}

#[deprecated(note = "use complete(queue, id, true)")]
pub fn mark_done(queue: &str, id: &str) -> Result<()> {
    complete(queue, id, true)
}

#[deprecated(note = "use complete(queue, id, false)")]
pub fn mark_failed(queue: &str, id: &str) -> Result<()> {
    complete(queue, id, false)
}

/// On startup: return stuck `processing/` jobs to `pending/` (crash recovery).
pub fn recover_processing(queue: &str) -> usize {
    let proc_dir = processing_dir(queue);
    let pending = pending_dir(queue);
    let _ = fs::create_dir_all(&pending);
    let Ok(entries) = fs::read_dir(&proc_dir) else {
        return 0;
    };
    let mut n = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if validate_schedule_id(id).is_err() {
            continue;
        }
        let Ok(dest) = pending_path(queue, id) else {
            continue;
        };
        if fs::rename(&path, &dest).is_ok() {
            n += 1;
        }
    }
    n
}

/// List queue names that exist on disk.
pub fn list_queues() -> Vec<String> {
    let Ok(entries) = fs::read_dir(root()) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(str::to_string))
        .collect()
}

/// Queue depth snapshot (for ops / UI).
#[derive(Debug, Clone, serde::Serialize)]
pub struct QueueStats {
    pub queue: String,
    pub pending: usize,
    pub processing: usize,
    pub done: usize,
    pub failed: usize,
}

pub fn stats(queue: &str) -> QueueStats {
    QueueStats {
        queue: queue.to_string(),
        pending: count_json_files(&pending_dir(queue)),
        processing: count_json_files(&processing_dir(queue)),
        done: count_json_files(&done_dir(queue)),
        failed: count_json_files(&failed_dir(queue)),
    }
}

fn count_json_files(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
                .count()
        })
        .unwrap_or(0)
}

#[cfg(test)]
/// Serializes exec integration tests that share global worker/durable state.
pub(crate) fn exec_test_lock() -> &'static parking_lot::Mutex<()> {
    static LOCK: once_cell::sync::Lazy<parking_lot::Mutex<()>> =
        once_cell::sync::Lazy::new(|| parking_lot::Mutex::new(()));
    &LOCK
}

#[cfg(test)]
pub(crate) fn test_queue_lock() -> &'static parking_lot::Mutex<()> {
    static LOCK: once_cell::sync::OnceCell<parking_lot::Mutex<()>> =
        once_cell::sync::OnceCell::new();
    LOCK.get_or_init(|| parking_lot::Mutex::new(()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp_queue() -> PathBuf {
        let p = std::env::temp_dir().join(format!("resuma-q-{}", crate::exec::id::next_id()));
        configure(&p);
        p
    }

    #[test]
    fn claim_is_exclusive() {
        let _guard = test_queue_lock().lock();
        let _root = temp_queue();
        let msg = QueueMessage {
            id: "m_test".into(),
            worker: "w".into(),
            input: json!({}),
        };
        persist_pending("default", &msg).unwrap();
        let first = claim_next("default").unwrap();
        assert_eq!(first.id, "m_test");
        assert!(claim_next("default").is_none());
        complete("default", "m_test", true).unwrap();
        assert_eq!(stats("default").done, 1);
    }

    #[test]
    fn claim_rejects_path_traversal_filename() {
        let _guard = test_queue_lock().lock();
        let _root = temp_queue();
        let dir = pending_dir("default");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("..%2f..%2foutside.json"), b"{}").unwrap();
        assert!(claim_next("default").is_none());
    }
}
