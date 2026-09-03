//! Dev-friendly disk serving for `/static/client/{id}.js` bundles.
//!
//! Production keeps `include_bytes!` in memory (immutable `?v=` digests).
//! When [`client_disk_mode`] is on, GET handlers re-read files from
//! `{client_dir}/{disk_file}` so `npm run watch:client` works without a cargo rebuild
//! (pokeLegens / False World dogfood).

use std::path::PathBuf;
use std::sync::OnceLock;

use parking_lot::RwLock;

use super::{content_digest, validate_client_id, CLIENT_SCRIPT_PREFIX};

static CLIENT_DIR: OnceLock<RwLock<Option<PathBuf>>> = OnceLock::new();
static CLIENT_DISK_RELS: OnceLock<RwLock<std::collections::HashMap<String, String>>> =
    OnceLock::new();

fn client_dir_slot() -> &'static RwLock<Option<PathBuf>> {
    CLIENT_DIR.get_or_init(|| RwLock::new(None))
}

fn disk_rels() -> &'static RwLock<std::collections::HashMap<String, String>> {
    CLIENT_DISK_RELS.get_or_init(|| RwLock::new(std::collections::HashMap::new()))
}

/// `RESUMA_CLIENT_DISK=1` forces on; `=0` forces off; otherwise follows `RESUMA_DEV`.
pub fn client_disk_mode() -> bool {
    match std::env::var("RESUMA_CLIENT_DISK").as_deref() {
        Ok("0") | Ok("false") | Ok("FALSE") => false,
        Ok("1") | Ok("true") | Ok("TRUE") => true,
        _ => crate::server::dev::dev_mode_enabled(),
    }
}

/// Default `{CARGO_MANIFEST_DIR}/static/client` (app crate when `cargo run`).
pub fn default_client_dir() -> PathBuf {
    std::env::var("RESUMA_CLIENT_DIR")
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::var("CARGO_MANIFEST_DIR").map(|m| PathBuf::from(m).join("static/client"))
        })
        .unwrap_or_else(|_| PathBuf::from("static/client"))
}

/// Override the directory used when [`client_disk_mode`] is active.
pub fn set_client_dir(dir: impl Into<PathBuf>) {
    *client_dir_slot().write() = Some(dir.into());
}

pub fn client_dir() -> PathBuf {
    client_dir_slot()
        .read()
        .clone()
        .unwrap_or_else(default_client_dir)
}

/// Relative path under the client dir (`play.js`, `fw-inventory.js`). No `..`.
pub fn validate_client_disk_file(file: &str) -> Result<(), ()> {
    if file.is_empty() || file.len() > 128 {
        return Err(());
    }
    if file.starts_with('/') || file.contains("..") || file.contains('\\') {
        return Err(());
    }
    if !file
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
    {
        return Err(());
    }
    Ok(())
}

/// Remember which on-disk file backs `id` (default `{id}.js` via [`register_client_disk_file`]).
pub fn register_client_disk_file(id: &str, file: &str) {
    if validate_client_id(id).is_err() || validate_client_disk_file(file).is_err() {
        tracing::warn!(
            client_id = id,
            disk_file = file,
            "invalid client disk mapping — skipped"
        );
        return;
    }
    disk_rels().write().insert(id.to_string(), file.to_string());
}

pub fn client_disk_rel(id: &str) -> String {
    disk_rels()
        .read()
        .get(id)
        .cloned()
        .unwrap_or_else(|| format!("{id}.js"))
}

pub fn resolve_client_disk_path(id: &str) -> PathBuf {
    client_dir().join(client_disk_rel(id))
}

/// Read current bytes from disk when disk mode is on and the file exists.
pub fn read_client_disk_bytes(id: &str) -> Option<Vec<u8>> {
    if !client_disk_mode() {
        return None;
    }
    let path = resolve_client_disk_path(id);
    let root = client_dir();
    let root = root.canonicalize().unwrap_or(root);
    let canonical = path.canonicalize().ok()?;
    if !canonical.starts_with(&root) {
        return None;
    }
    std::fs::read(canonical).ok()
}

/// Live `?v=` digest from disk when possible (so watch rebuilds bust the module URL).
pub fn client_script_url_disk_aware(id: &str, memory_digest: Option<&str>) -> String {
    let path = format!("{CLIENT_SCRIPT_PREFIX}{id}.js");
    if let Some(bytes) = read_client_disk_bytes(id) {
        return format!("{path}?v={}", content_digest(&bytes));
    }
    match memory_digest {
        Some(v) => format!("{path}?v={v}"),
        None => path,
    }
}

/// Soft cache always when serving from disk (watch-friendly).
pub fn client_asset_cache_control() -> &'static str {
    if client_disk_mode() {
        crate::server::static_assets::STATIC_DEV_CACHE
    } else {
        crate::server::static_assets::static_cache_control()
    }
}

/// Build GET response body: prefer disk bytes, else embedded `include_bytes!`.
pub fn client_asset_body(id: &str, embedded: &'static [u8]) -> Vec<u8> {
    read_client_disk_bytes(id).unwrap_or_else(|| embedded.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn rejects_path_traversal_disk_file() {
        assert!(validate_client_disk_file("../x.js").is_err());
        assert!(validate_client_disk_file("/abs.js").is_err());
        assert!(validate_client_disk_file("play.js").is_ok());
        assert!(validate_client_disk_file("nested/play.js").is_ok());
    }

    #[test]
    fn disk_read_respects_client_dir() {
        let dir = std::env::temp_dir().join(format!("resuma-client-disk-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("widget.js"), b"export const n=1").unwrap();

        let prev = std::env::var_os("RESUMA_CLIENT_DISK");
        std::env::set_var("RESUMA_CLIENT_DISK", "1");
        set_client_dir(&dir);
        register_client_disk_file("widget", "widget.js");

        let got = read_client_disk_bytes("widget").expect("disk read");
        assert_eq!(got.as_slice(), b"export const n=1");

        match prev {
            Some(v) => std::env::set_var("RESUMA_CLIENT_DISK", v),
            None => std::env::remove_var("RESUMA_CLIENT_DISK"),
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
