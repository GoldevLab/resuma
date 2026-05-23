//! Helpers for emitting / parsing the resumability payload.

use crate::context::ResumePayload;

/// Serialize a `ResumePayload` to a compact JSON blob suitable for inlining
/// in `<script type="resuma/state">`.
pub fn encode_payload(payload: &ResumePayload) -> String {
    serde_json::to_string(payload).unwrap_or_else(|_| "{}".into())
}
