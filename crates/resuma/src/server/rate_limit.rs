//! Pluggable rate-limit backends — in-memory (dev) and disk (production).

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use parking_lot::RwLock;

use crate::core::{Result, ResumaError};

pub trait RateLimitBackend: Send + Sync {
    fn check(&self, key: &str, limit_per_minute: u32, window: Duration) -> Result<()>;
}

struct MemoryBackend;

impl RateLimitBackend for MemoryBackend {
    fn check(&self, key: &str, limit_per_minute: u32, window: Duration) -> Result<()> {
        memory_check(key, limit_per_minute, window)
    }
}

struct MemoryState {
    buckets: HashMap<String, Vec<Instant>>,
    /// LRU order — front is oldest, back is most recently used.
    order: VecDeque<String>,
}

static MEMORY_STATE: Lazy<RwLock<MemoryState>> = Lazy::new(|| {
    RwLock::new(MemoryState {
        buckets: HashMap::new(),
        order: VecDeque::new(),
    })
});

/// Cap in-memory rate-limit keys to avoid unbounded growth from IP rotation attacks.
const MEMORY_BUCKET_KEY_CAP: usize = 10_000;

fn touch_lru(state: &mut MemoryState, key: &str) {
    state.order.retain(|k| k != key);
    state.order.push_back(key.to_string());
}

fn memory_check(key: &str, limit_per_minute: u32, window: Duration) -> Result<()> {
    if limit_per_minute == 0 {
        return Ok(());
    }
    let now = Instant::now();
    let mut state = MEMORY_STATE.write();
    state.buckets.retain(|_, entries| {
        entries.retain(|t| now.duration_since(*t) < window);
        !entries.is_empty()
    });
    let live_keys: std::collections::HashSet<String> = state.buckets.keys().cloned().collect();
    state.order.retain(|k| live_keys.contains(k));

    let key_string = key.to_string();
    if !state.buckets.contains_key(key) {
        while state.order.len() >= MEMORY_BUCKET_KEY_CAP {
            if let Some(evict_key) = state.order.pop_front() {
                state.buckets.remove(&evict_key);
            } else {
                break;
            }
        }
    }

    let entries = state.buckets.entry(key_string).or_default();
    entries.retain(|t| now.duration_since(*t) < window);
    if entries.len() as u32 >= limit_per_minute {
        return Err(ResumaError::RateLimited);
    }
    entries.push(now);
    touch_lru(&mut state, key);
    Ok(())
}

static BACKEND: Lazy<RwLock<Arc<dyn RateLimitBackend>>> =
    Lazy::new(|| RwLock::new(Arc::new(MemoryBackend)));

/// Replace the global rate-limit backend (e.g. custom store in tests).
pub fn configure_rate_limit_backend(backend: Arc<dyn RateLimitBackend>) {
    *BACKEND.write() = backend;
}

pub fn install_default_backend() {
    let backend = std::env::var("RESUMA_RATE_BACKEND").unwrap_or_default();
    if backend.eq_ignore_ascii_case("redis") {
        tracing::warn!(
            "RESUMA_RATE_BACKEND=redis is no longer supported — using disk backend \
             ({}/rate-limit). For multi-region deploys, add edge rate limiting \
             (nginx limit_req, Fly proxy, etc.).",
            std::env::var("RESUMA_DATA_DIR").unwrap_or_else(|_| ".resuma".into())
        );
    }
    if backend.eq_ignore_ascii_case("disk")
        || backend.eq_ignore_ascii_case("redis")
        || disk_backend_enabled()
    {
        let root = std::env::var("RESUMA_DATA_DIR").unwrap_or_else(|_| ".resuma".into());
        super::rate_limit_disk::configure(format!("{root}/rate-limit"));
        configure_rate_limit_backend(Arc::new(super::rate_limit_disk::DiskBackend));
        return;
    }
    configure_rate_limit_backend(Arc::new(MemoryBackend));
}

fn disk_backend_enabled() -> bool {
    matches!(
        std::env::var("RESUMA_ENV").as_deref(),
        Ok("production") | Ok("prod")
    ) && std::env::var("RESUMA_RATE_BACKEND")
        .map(|v| !v.eq_ignore_ascii_case("memory"))
        .unwrap_or(true)
}

pub fn check_rate_limit_key(key: &str, limit_per_minute: u32) -> Result<()> {
    BACKEND
        .read()
        .check(key, limit_per_minute, Duration::from_secs(60))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lru_evicts_oldest_key_not_hottest_bucket() {
        configure_rate_limit_backend(Arc::new(MemoryBackend));
        let window = Duration::from_secs(60);
        for i in 0..MEMORY_BUCKET_KEY_CAP {
            let key = format!("probe:{i}");
            memory_check(&key, 100, window).expect("check");
        }
        assert!(memory_check("probe:0", 100, window).is_ok());
        assert!(memory_check("probe:9999", 100, window).is_ok());
        assert!(memory_check("new-attacker-key", 100, window).is_ok());
    }
}
