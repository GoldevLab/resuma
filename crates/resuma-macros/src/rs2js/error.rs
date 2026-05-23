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
