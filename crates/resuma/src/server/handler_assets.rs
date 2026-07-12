//! Register lazy handler / island JS served from SSR payloads.

use std::collections::BTreeMap;

use crate::core::ResumePayload;
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;

/// Flatten handler symbols into an ES module for `/_resuma/handler/:chunk.js`.
pub fn handler_chunk_module(symbols: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    for (symbol, source) in symbols {
        out.push_str(&handler_export(symbol, source));
    }
    out
}

/// Island chunk module — handlers plus optional no-op `resume` entry.
pub fn island_chunk_module(symbols: &BTreeMap<String, String>) -> String {
    let mut out = handler_chunk_module(symbols);
    out.push_str("export function resume(_props, _signals, _root) {}\n");
    out
}

const ISLAND_STUB: &str = "export function resume(_props, _signals, _root) {}\n";

fn handler_export(symbol: &str, source: &str) -> String {
    let body = source.trim();
    if is_function_expression(body) {
        format!("export const {symbol} = {body};\n")
    } else {
        format!("export function {symbol}(event, state, __resuma) {{ {body} }}\n")
    }
}

fn is_function_expression(source: &str) -> bool {
    source.starts_with("function") || source.starts_with('(') || source.starts_with("async")
}

fn module_has_symbol(module: &str, symbol: &str) -> bool {
    module.contains(&format!("export const {symbol} "))
        || module.contains(&format!("export function {symbol}("))
}

/// Short stable digest for cache-busting lazy chunk URLs.
pub fn chunk_digest(source: &str) -> String {
    let hash = Sha256::digest(source.as_bytes());
    format!("{:x}", hash)[..16].to_string()
}

/// Attach server-side chunk digests for chunks referenced on this page.
pub fn attach_chunk_digests(
    payload: &mut ResumePayload,
    handler_chunks: &Arc<RwLock<HashMap<String, String>>>,
    island_chunks: &Arc<RwLock<HashMap<String, String>>>,
) {
    let handlers = handler_chunks.read();
    let islands = island_chunks.read();
    let mut digests = BTreeMap::new();
    for chunk in payload.lazy_chunks.iter().chain(payload.islands.iter()) {
        if let Some(src) = handlers.get(chunk).or_else(|| islands.get(chunk)) {
            digests.insert(chunk.clone(), chunk_digest(src));
        }
    }
    payload.chunk_digests = digests;
}

/// Merge SSR handler chunks into the server's lazy-load map.
pub fn merge_payload_handlers(
    handler_chunks: &Arc<RwLock<HashMap<String, String>>>,
    island_chunks: &Arc<RwLock<HashMap<String, String>>>,
    payload: &ResumePayload,
) {
    let mut handlers = handler_chunks.write();
    for (chunk, symbols) in &payload.handlers {
        let module = handlers.entry(chunk.clone()).or_default();
        for (symbol, source) in symbols {
            if !module_has_symbol(module, symbol) {
                module.push_str(&handler_export(symbol, source));
            }
        }
    }

    let mut islands = island_chunks.write();
    for island in &payload.islands {
        if islands.contains_key(island) {
            continue;
        }
        let module = payload
            .handlers
            .get(island)
            .map(island_chunk_module)
            .unwrap_or_else(|| ISLAND_STUB.to_string());
        islands.insert(island.clone(), module);
    }
}
