//! Static bytes served by [`FlowApp::static_asset`](crate::flow::FlowApp::static_asset).

use axum::http::{header, HeaderValue};

/// Cache-Control for content-addressed / versioned bundles in production.
pub const STATIC_IMMUTABLE_CACHE: &str = "public, max-age=31536000, immutable";

/// Soft cache for `RESUMA_DEV=1` — never mark unhashed edit targets immutable.
pub const STATIC_DEV_CACHE: &str = "public, max-age=0, must-revalidate";

/// Production vs dev Cache-Control for embedded static / client assets.
pub fn static_cache_control() -> &'static str {
    if crate::server::dev::dev_mode_enabled() {
        STATIC_DEV_CACHE
    } else {
        STATIC_IMMUTABLE_CACHE
    }
}

/// Build a GET response for a fixed static asset (Cache-Control + Content-Type).
pub fn static_asset_response(
    content_type: &str,
    body: &'static [u8],
) -> ([(header::HeaderName, HeaderValue); 2], Vec<u8>) {
    let ct = HeaderValue::from_str(content_type)
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
    let cache = HeaderValue::from_static(static_cache_control());
    (
        [(header::CONTENT_TYPE, ct), (header::CACHE_CONTROL, cache)],
        body.to_vec(),
    )
}

/// Owned-bytes variant (disk-backed client assets, etc.).
pub fn static_bytes_response(
    content_type: &str,
    body: Vec<u8>,
    cache_control: &'static str,
) -> ([(header::HeaderName, HeaderValue); 2], Vec<u8>) {
    let ct = HeaderValue::from_str(content_type)
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
    let cache = HeaderValue::from_static(cache_control);
    (
        [(header::CONTENT_TYPE, ct), (header::CACHE_CONTROL, cache)],
        body,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_control_is_soft_in_dev() {
        let prev = std::env::var("RESUMA_DEV").ok();
        std::env::set_var("RESUMA_DEV", "1");
        assert_eq!(static_cache_control(), STATIC_DEV_CACHE);
        match prev {
            Some(v) => std::env::set_var("RESUMA_DEV", v),
            None => std::env::remove_var("RESUMA_DEV"),
        }
    }
}
