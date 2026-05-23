//! HTTP request helpers for Resuma Flow.

pub use crate::core::FlowRequest;

use std::collections::BTreeMap;

/// Build a [`FlowRequest`] from axum HTTP parts.
pub fn from_http(
    method: &str,
    path: &str,
    headers: &axum::http::HeaderMap,
    params: BTreeMap<String, String>,
    query: BTreeMap<String, String>,
) -> FlowRequest {
    FlowRequest::from_parts(
        method,
        path,
        headers
            .iter()
            .filter_map(|(k, v)| {
                v.to_str()
                    .ok()
                    .map(|s| (k.as_str().to_string(), s.to_string()))
            })
            .collect(),
        params,
        query,
    )
}
