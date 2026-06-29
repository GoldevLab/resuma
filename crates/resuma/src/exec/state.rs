//! External durable state (`ctx.state`) — not RAM.

use std::collections::BTreeMap;

use parking_lot::RwLock;
use serde_json::Value;

/// Per-execution key-value store (checkpointed with graph).
#[derive(Debug, Default)]
pub struct StateStore {
    inner: RwLock<BTreeMap<String, Value>>,
}

impl StateStore {
    pub fn get(&self, key: &str) -> Option<Value> {
        self.inner.read().get(key).cloned()
    }

    pub fn set(&self, key: impl Into<String>, value: Value) {
        self.inner.write().insert(key.into(), value);
    }

    pub fn snapshot(&self) -> BTreeMap<String, Value> {
        self.inner.read().clone()
    }
}
