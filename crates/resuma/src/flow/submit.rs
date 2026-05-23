//! Form submissions — `#[submit]` handlers with progressive enhancement.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Result of a `#[submit]` handler.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SubmitValue<T> {
    Ok(T),
    Err(SubmitError),
    Running,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitError {
    pub message: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub field_errors: BTreeMap<String, String>,
}

impl SubmitError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field_errors: Default::default(),
        }
    }

    pub fn field(mut self, key: impl Into<String>, message: impl Into<String>) -> Self {
        self.field_errors.insert(key.into(), message.into());
        self
    }
}

/// Encode a submit handler return value, mapping validation errors to HTTP responses.
pub fn encode_submit_result<T: serde::Serialize>(
    res: Result<T, SubmitError>,
) -> crate::core::Result<serde_json::Value> {
    match res {
        Ok(v) => serde_json::to_value(v).map_err(Into::into),
        Err(e) => Err(crate::core::ResumaError::Other(
            serde_json::to_string(&e).unwrap_or_else(|_| e.message.clone()),
        )),
    }
}

/// Type-erased submit dispatch.
pub type SubmitFn = fn(Value) -> SubmitDispatch;

pub enum SubmitDispatch {
    Ok(Value),
    Err(SubmitError),
}
