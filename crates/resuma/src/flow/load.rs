//! Server-side data loading — `#[load]` handlers run before page render.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::request::FlowRequest;

/// Result of a `#[load]` handler exposed to components.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum LoadValue<T> {
    Ok(T),
    Err(LoaderError),
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderError {
    pub status: u16,
    pub message: String,
}

impl std::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (status {})", self.message, self.status)
    }
}

impl LoaderError {
    pub fn new(status: u16, message: impl Into<String>) -> Self {
        Self { status, message: message.into() }
    }
}

/// Type-erased loader dispatch signature used by the Flow registry.
pub type LoadFn = fn(&FlowRequest) -> LoadDispatch;

pub enum LoadDispatch {
    Ready(Value),
    Pending,
}
