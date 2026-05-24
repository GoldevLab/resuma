//! Immutable static bytes served by [`FlowApp::static_asset`](crate::flow::FlowApp::static_asset).

use axum::http::{header, HeaderValue};

/// Cache-Control for content-hashed or versioned bundles embedded at compile time.
pub const STATIC_IMMUTABLE_CACHE: &str = "public, max-age=31536000, immutable";

/// Build a GET response for a fixed static asset (Cache-Control + Content-Type).
pub fn static_asset_response(
    content_type: &str,
    body: &'static [u8],
) -> ([(header::HeaderName, HeaderValue); 2], Vec<u8>) {
    let ct = HeaderValue::from_str(content_type)
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
    (
        [
            (header::CONTENT_TYPE, ct),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static(STATIC_IMMUTABLE_CACHE),
            ),
        ],
        body.to_vec(),
    )
}
