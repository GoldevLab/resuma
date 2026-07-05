//! Helpers for emitting / parsing the resumability payload.

use super::context::ResumePayload;

/// Serialize a `ResumePayload` to a compact JSON blob suitable for inlining
/// in `<script type="resuma/state">`.
///
/// Escapes `<`, `>`, `&`, and Unicode line separators so user-controlled signal
/// data cannot break out of the script block (XSS).
pub fn encode_payload(payload: &ResumePayload) -> String {
    let raw = match serde_json::to_string(payload) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "failed to serialize resumability payload");
            "{}".into()
        }
    };
    sanitize_json_for_script(&raw)
}

/// Prevent `</script>` and HTML injection from serialized JSON embedded in HTML.
pub fn sanitize_json_for_script(json: &str) -> String {
    json.replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}
