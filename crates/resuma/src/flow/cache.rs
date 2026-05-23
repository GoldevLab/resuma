//! Loader cache registry and page `Cache-Control` merging.

use std::collections::BTreeMap;

use once_cell::sync::Lazy;
use parking_lot::RwLock;

static LOADER_CACHE: Lazy<RwLock<BTreeMap<String, String>>> =
    Lazy::new(|| RwLock::new(BTreeMap::new()));

/// Register a default `Cache-Control` value for a `#[load]` handler.
pub fn register_loader_cache(name: &str, cache_control: impl Into<String>) {
    LOADER_CACHE
        .write()
        .insert(name.to_string(), cache_control.into());
}

/// Lookup a loader's registered cache policy.
pub fn loader_cache(name: &str) -> Option<String> {
    LOADER_CACHE.read().get(name).cloned()
}

/// Merge per-loader cache hints into a single page `Cache-Control` header.
pub fn merge_cache_control(hints: &BTreeMap<String, String>) -> Option<String> {
    if hints.is_empty() {
        return None;
    }
    if hints.len() == 1 {
        return hints.values().next().cloned();
    }

    let values: Vec<&str> = hints.values().map(|s| s.as_str()).collect();
    for v in &values {
        if v.contains("no-store") {
            return Some("no-store".into());
        }
    }
    for v in &values {
        if v.contains("no-cache") {
            return Some("no-cache".into());
        }
    }

    let mut min_age: Option<u64> = None;
    let mut is_private = false;
    for v in &values {
        if v.contains("private") {
            is_private = true;
        }
        if let Some(age) = parse_max_age(v) {
            min_age = Some(min_age.map_or(age, |current| current.min(age)));
        }
    }

    match min_age {
        Some(age) => {
            let scope = if is_private { "private" } else { "public" };
            Some(format!("{scope}, max-age={age}"))
        }
        None if is_private => Some("private".into()),
        None => values.first().map(|s| (*s).to_string()),
    }
}

fn parse_max_age(value: &str) -> Option<u64> {
    value.split(',').find_map(|part| {
        let part = part.trim();
        part.strip_prefix("max-age=")?.trim().parse().ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_single_hint() {
        let mut hints = BTreeMap::new();
        hints.insert("home".into(), "public, max-age=60".into());
        assert_eq!(
            merge_cache_control(&hints).as_deref(),
            Some("public, max-age=60")
        );
    }

    #[test]
    fn merge_picks_shorter_max_age() {
        let mut hints = BTreeMap::new();
        hints.insert("a".into(), "public, max-age=300".into());
        hints.insert("b".into(), "public, max-age=60".into());
        assert_eq!(
            merge_cache_control(&hints).as_deref(),
            Some("public, max-age=60")
        );
    }
}
