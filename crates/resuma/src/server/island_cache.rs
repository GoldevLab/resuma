//! Last-rendered island HTML for dev refresh (`GET /_resuma/island/:instance`).

use std::collections::HashMap;

use once_cell::sync::Lazy;
use parking_lot::RwLock;

use crate::ssr::escape_attr;

#[derive(Clone)]
struct IslandEntry {
    inner_html: String,
    chunk_id: String,
    load: String,
}

static CACHE: Lazy<RwLock<HashMap<String, IslandEntry>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

const MAX_ISLAND_CACHE_ENTRIES: usize = 256;

/// Store island inner HTML during SSR (called from the SSR layer).
pub fn cache_island_html(instance_id: &str, inner_html: &str, chunk_id: &str, load: &str) {
    let mut cache = CACHE.write();
    if cache.len() >= MAX_ISLAND_CACHE_ENTRIES && !cache.contains_key(instance_id) {
        if let Some(key) = cache.keys().next().cloned() {
            cache.remove(&key);
        }
    }
    cache.insert(
        instance_id.to_string(),
        IslandEntry {
            inner_html: inner_html.to_string(),
            chunk_id: chunk_id.to_string(),
            load: load.to_string(),
        },
    );
}

/// Full `<resuma-island>` element for HMR / refresh endpoint.
pub fn island_refresh_html(instance_id: &str) -> Option<String> {
    CACHE.read().get(instance_id).map(|entry| {
        format!(
            r#"<resuma-island data-r-chunk="{chunk}" data-r-instance="{inst}" data-r-load="{load}">{inner}</resuma-island>"#,
            chunk = escape_attr(&entry.chunk_id),
            inst = escape_attr(instance_id),
            load = escape_attr(&entry.load),
            inner = entry.inner_html,
        )
    })
}

/// Clear cache (tests).
#[doc(hidden)]
pub fn clear_island_cache() {
    CACHE.write().clear();
}
