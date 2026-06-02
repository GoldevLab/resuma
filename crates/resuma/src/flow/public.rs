//! Serve files from a project `public/` directory at URL paths.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use walkdir::WalkDir;

/// A file read from `public/` to register as a GET route.
#[derive(Clone)]
pub struct PublicAsset {
    pub url_path: String,
    pub body: Arc<Vec<u8>>,
    pub content_type: String,
}

/// Relative paths (under `public/`) that override generated PWA SVG icons when present.
pub const PWA_ICON_CANDIDATES: &[(&str, &str, &str)] = &[
    ("icons/icon-192.png", "/icons/icon-192.png", "192x192"),
    ("icons/icon-512.png", "/icons/icon-512.png", "512x512"),
    ("icons/icon-maskable.png", "/icons/icon-maskable.png", "512x512"),
    ("icons/apple-touch-icon.png", "/icons/apple-touch-icon.png", "180x180"),
    ("icon-192.png", "/icons/icon-192.png", "192x192"),
    ("icon-512.png", "/icons/icon-512.png", "512x512"),
    ("icon.png", "/icons/icon-192.png", "192x192"),
];

/// Walk `dir` (typically `public/`) and collect servable assets.
pub fn collect_public_dir(dir: &Path) -> Vec<PublicAsset> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let root = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    let mut out = Vec::new();
    for entry in WalkDir::new(&root).min_depth(1).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let rel = match path.strip_prefix(&root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let rel_str = rel.to_string_lossy();
        if rel_str.starts_with('.') || rel_str.contains("/.") {
            continue;
        }
        let url_path = format!("/{}", rel_str.replace('\\', "/"));
        let body = match std::fs::read(path) {
            Ok(b) => Arc::new(b),
            Err(_) => continue,
        };
        let content_type = content_type_for_path(path);
        out.push(PublicAsset {
            url_path,
            body,
            content_type,
        });
    }
    out.sort_by(|a, b| a.url_path.cmp(&b.url_path));
    out
}

/// Default `public/` next to `CARGO_MANIFEST_DIR`.
pub fn default_public_dir() -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(|m| PathBuf::from(m).join("public"))
        .unwrap_or_else(|_| PathBuf::from("public"))
}

pub fn content_type_for_path(path: &Path) -> String {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ico" => "image/x-icon",
        "webmanifest" => "application/manifest+json; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn collect_skips_missing_dir() {
        assert!(collect_public_dir(Path::new("/nonexistent-resuma-public-dir")).is_empty());
    }

    #[test]
    fn maps_files_to_url_paths() {
        let dir = std::env::temp_dir().join(format!("resuma-public-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("images")).unwrap();
        fs::write(dir.join("images/a.png"), b"png").unwrap();
        let assets = collect_public_dir(&dir);
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].url_path, "/images/a.png");
        assert_eq!(assets[0].content_type, "image/png");
        let _ = fs::remove_dir_all(&dir);
    }
}
