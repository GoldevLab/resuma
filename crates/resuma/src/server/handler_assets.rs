//! Register lazy handler / island JS served from SSR payloads.

use std::collections::BTreeMap;

use crate::core::ResumePayload;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Flatten handler symbols into an ES module for `/_resuma/handler/:chunk.js`.
pub fn handler_chunk_module(symbols: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    for (symbol, source) in symbols {
        let body = source.trim();
        if body.starts_with("function") || body.starts_with('(') || body.starts_with("async") {
            out.push_str(&format!("export {body}\n"));
        } else {
            out.push_str(&format!(
                "export function {symbol}(event, state, __resuma) {{ {body} }}\n",
                symbol = symbol
            ));
        }
    }
    out
}

/// Merge SSR handler chunks into the server's lazy-load map (dedupe by chunk id).
pub fn merge_payload_handlers(
    handler_chunks: &Arc<RwLock<HashMap<String, String>>>,
    island_chunks: &Arc<RwLock<HashMap<String, String>>>,
    payload: &ResumePayload,
) {
    let mut handlers = handler_chunks.write();
    for (chunk, symbols) in &payload.handlers {
        if chunk == "__page__" || handlers.contains_key(chunk) {
            continue;
        }
        handlers.insert(chunk.clone(), handler_chunk_module(symbols));
    }

    let mut islands = island_chunks.write();
    for island in &payload.islands {
        if islands.contains_key(island) {
            continue;
        }
        if let Some(symbols) = payload.handlers.get(island) {
            let mut module = handler_chunk_module(symbols);
            module.push_str("\nexport function resume(props, signals, root) {}\n");
            islands.insert(island.clone(), module);
        }
    }
}
