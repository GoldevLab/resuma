//! Negotiated gzip / brotli responses for embedded static JS assets.

use axum::body::Body;
use axum::http::{header, HeaderMap, HeaderValue, Response, StatusCode};
use once_cell::sync::Lazy;

pub(crate) struct EncodedAsset {
    raw_len: usize,
    gzip: Vec<u8>,
    brotli: Vec<u8>,
}

fn encode_asset(raw: &str) -> EncodedAsset {
    EncodedAsset {
        raw_len: raw.len(),
        gzip: gzip_raw(raw.as_bytes()),
        brotli: brotli_raw(raw.as_bytes()),
    }
}

fn gzip_raw(input: &[u8]) -> Vec<u8> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut enc = GzEncoder::new(Vec::new(), Compression::best());
    enc.write_all(input).expect("gzip encode");
    enc.finish().expect("gzip finish")
}

fn brotli_raw(input: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut out = Vec::new();
    {
        let mut enc = brotli::CompressorWriter::new(&mut out, 4096, 11, 22);
        enc.write_all(input).expect("brotli encode");
    }
    out
}

static LOADER: Lazy<EncodedAsset> = Lazy::new(|| encode_asset(super::runtime_asset::LOADER_JS));
static CORE: Lazy<EncodedAsset> = Lazy::new(|| encode_asset(super::runtime_asset::CORE_JS));
static FLOW: Lazy<EncodedAsset> = Lazy::new(|| encode_asset(super::runtime_asset::FLOW_JS));
static RUNTIME: Lazy<EncodedAsset> = Lazy::new(|| encode_asset(super::runtime_asset::RUNTIME_JS));

pub(crate) fn serve_js(
    headers: &HeaderMap,
    asset: &'static EncodedAsset,
    raw: &'static str,
) -> Response<Body> {
    let accept = headers
        .get(header::ACCEPT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let (body, encoding) = if accept.contains("br") {
        (Body::from(asset.brotli.clone()), "br")
    } else if accept.contains("gzip") {
        (Body::from(asset.gzip.clone()), "gzip")
    } else {
        (Body::from(raw.to_string()), "identity")
    };

    let mut res = Response::new(body);
    *res.status_mut() = StatusCode::OK;
    let h = res.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/javascript; charset=utf-8"),
    );
    if encoding != "identity" {
        h.insert(header::CONTENT_ENCODING, HeaderValue::from_static(encoding));
        h.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
    }
    h.insert(
        header::CACHE_CONTROL,
        // Fixed paths (/_resuma/flow.js etc.) change across releases — never mark
        // them immutable or browsers keep stale bundles for a year.
        HeaderValue::from_static("public, max-age=60, must-revalidate"),
    );
    res
}

pub(crate) fn loader_asset() -> &'static EncodedAsset {
    &LOADER
}

pub(crate) fn core_asset() -> &'static EncodedAsset {
    &CORE
}

pub(crate) fn flow_asset() -> &'static EncodedAsset {
    &FLOW
}

pub(crate) fn runtime_asset() -> &'static EncodedAsset {
    &RUNTIME
}

/// Report bundle sizes for diagnostics / benchmark pages.
pub fn asset_sizes() -> [(&'static str, usize, usize, usize); 3] {
    [
        (
            "loader.js",
            LOADER.raw_len,
            LOADER.gzip.len(),
            LOADER.brotli.len(),
        ),
        ("core.js", CORE.raw_len, CORE.gzip.len(), CORE.brotli.len()),
        (
            "runtime.js (legacy)",
            RUNTIME.raw_len,
            RUNTIME.gzip.len(),
            RUNTIME.brotli.len(),
        ),
    ]
}
