//! Serialization helpers shared by the various server endpoints.

use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorPayload<'a> {
    pub error: &'a str,
}
