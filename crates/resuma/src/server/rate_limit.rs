//! Pluggable rate-limit backends (in-memory default, optional Redis).

use std::sync::Arc;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;

use crate::core::{ResumaError, Result};

pub trait RateLimitBackend: Send + Sync {
    fn check(&self, key: &str, limit_per_minute: u32, window: Duration) -> Result<()>;
}

struct MemoryBackend;

impl RateLimitBackend for MemoryBackend {
    fn check(&self, key: &str, limit_per_minute: u32, window: Duration) -> Result<()> {
        memory_check(key, limit_per_minute, window)
    }
}

static MEMORY_BUCKETS: Lazy<RwLock<HashMap<String, Vec<Instant>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

fn memory_check(key: &str, limit_per_minute: u32, window: Duration) -> Result<()> {
    if limit_per_minute == 0 {
        return Ok(());
    }
    let now = Instant::now();
    let mut map = MEMORY_BUCKETS.write();
    map.retain(|_, entries| {
        entries.retain(|t| now.duration_since(*t) < window);
        !entries.is_empty()
    });
    let entries = map.entry(key.to_string()).or_default();
    entries.retain(|t| now.duration_since(*t) < window);
    if entries.len() as u32 >= limit_per_minute {
        return Err(ResumaError::RateLimited);
    }
    entries.push(now);
    Ok(())
}

#[cfg(feature = "redis-rate-limit")]
mod redis_backend {
    use super::*;
    use std::env;

    pub struct RedisBackend {
        client: redis::Client,
    }

    impl RedisBackend {
        pub fn from_env() -> Option<Self> {
            let url = env::var("RESUMA_REDIS_URL").ok()?;
            redis::Client::open(url).ok().map(|client| Self { client })
        }
    }

    impl RateLimitBackend for RedisBackend {
        fn check(&self, key: &str, limit_per_minute: u32, window: Duration) -> Result<()> {
            if limit_per_minute == 0 {
                return Ok(());
            }
            let mut conn = self
                .client
                .get_connection()
                .map_err(|e| ResumaError::Internal(format!("redis rate limit: {e}")))?;
            let redis_key = format!("resuma:rl:{key}");
            let count: u32 = redis::cmd("INCR")
                .arg(&redis_key)
                .query(&mut conn)
                .map_err(|e| ResumaError::Internal(format!("redis INCR: {e}")))?;
            if count == 1 {
                let _: () = redis::cmd("EXPIRE")
                    .arg(&redis_key)
                    .arg(window.as_secs().max(1))
                    .query(&mut conn)
                    .map_err(|e| ResumaError::Internal(format!("redis EXPIRE: {e}")))?;
            }
            if count > limit_per_minute {
                return Err(ResumaError::RateLimited);
            }
            Ok(())
        }
    }

    pub fn try_default() -> Option<Arc<dyn RateLimitBackend>> {
        RedisBackend::from_env().map(|b| Arc::new(b) as Arc<dyn RateLimitBackend>)
    }
}

static BACKEND: Lazy<RwLock<Arc<dyn RateLimitBackend>>> =
    Lazy::new(|| RwLock::new(Arc::new(MemoryBackend)));

/// Replace the global rate-limit backend (e.g. Redis in multi-instance deploys).
pub fn configure_rate_limit_backend(backend: Arc<dyn RateLimitBackend>) {
    *BACKEND.write() = backend;
}

pub fn install_default_backend() {
    #[cfg(feature = "redis-rate-limit")]
    if let Some(redis) = redis_backend::try_default() {
        configure_rate_limit_backend(redis);
        return;
    }
    configure_rate_limit_backend(Arc::new(MemoryBackend));
}

pub fn check_rate_limit_key(key: &str, limit_per_minute: u32) -> Result<()> {
    BACKEND
        .read()
        .check(key, limit_per_minute, Duration::from_secs(60))
}
