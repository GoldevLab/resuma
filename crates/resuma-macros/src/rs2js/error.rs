use proc_macro2::Span;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("rs2js: unsupported construct: {message}")]
pub struct Rs2JsError {
    pub message: String,
    pub span: Span,
}

impl Rs2JsError {
    pub fn unsupported(what: &str, span: Span) -> Self {
        Self {
            message: what.to_string(),
            span,
        }
    }
}

/// User-facing compile error with remediation hints (handlers, effects, computed, debounce).
pub fn translation_help(context: &str, err: &Rs2JsError) -> String {
    format!(
        "Resuma could not compile this {context} to browser JavaScript: {}.\n\n\
         Supported client code is intentionally small and resumable. Try one of these:\n\
         - update signals directly: count.update(|c| *c += 1)\n\
         - use js! {{ ... }} for DOM/browser APIs\n\
         - move complex Rust or database work into a #[server] action",
        err.message
    )
}
