//! Global request extensions injected into every [`crate::FlowRequest`] (DB handles, config).

use std::collections::BTreeMap;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde_json::Value;

/// Shared extensions merged into each Flow request before middleware runs.
#[derive(Debug, Clone, Default)]
pub struct FlowExtensions(pub BTreeMap<String, Value>);

static GLOBAL_EXTENSIONS: Lazy<RwLock<FlowExtensions>> =
    Lazy::new(|| RwLock::new(FlowExtensions::default()));

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

/// Install extensions for submit/action handlers (set by [`FlowApp::into_router`]).
pub fn set_global_extensions(ext: FlowExtensions) {
    *GLOBAL_EXTENSIONS.write() = ext;
}

/// Extensions configured on the active Flow app.
pub fn global_extensions() -> FlowExtensions {
    GLOBAL_EXTENSIONS.read().clone()
}
