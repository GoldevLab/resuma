//! Per-request HTTP context shared by loads, submits, middleware, and `#[server]` actions.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Request context for a single HTTP request in Resuma Flow / server actions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlowRequest {
    pub method: String,
    pub path: String,
    pub params: BTreeMap<String, String>,
    pub query: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
    /// Arbitrary middleware / handler metadata attached during the request.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: BTreeMap<String, Value>,
    /// Per-loader HTTP cache hints (`Cache-Control` values).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub cache_control: BTreeMap<String, String>,
}

impl FlowRequest {
    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    pub fn query_param(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(|s| s.as_str())
    }

    pub fn header(&self, key: &str) -> Option<&str> {
        if let Some(v) = self.headers.get(key) {
            return Some(v.as_str());
        }
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }

    pub fn set_extension(&mut self, key: impl Into<String>, value: Value) {
        self.extensions.insert(key.into(), value);
    }

    pub fn extension(&self, key: &str) -> Option<&Value> {
        self.extensions.get(key)
    }

    /// Build a request from plain HTTP parts (no framework-specific types).
    pub fn from_parts(
        method: impl Into<String>,
        path: impl Into<String>,
        headers: BTreeMap<String, String>,
        params: BTreeMap<String, String>,
        query: BTreeMap<String, String>,
    ) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            params,
            query,
            headers,
            ..Default::default()
        }
    }
}
