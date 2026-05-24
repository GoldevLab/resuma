//! Global request extensions injected into every [`FlowRequest`] (DB handles, config).

use std::collections::BTreeMap;

use serde_json::Value;

/// Shared extensions merged into each Flow request before middleware runs.
#[derive(Debug, Clone, Default)]
pub struct FlowExtensions(pub BTreeMap<String, Value>);

impl FlowExtensions {
    pub fn insert(&mut self, key: impl Into<String>, value: Value) {
        self.0.insert(key.into(), value);
    }

    pub fn merge_into(&self, req: &mut crate::core::FlowRequest) {
        for (k, v) in &self.0 {
            req.extensions.entry(k.clone()).or_insert_with(|| v.clone());
        }
    }
}
