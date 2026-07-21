//! Serve files from a project `public/` directory at URL paths.
//!
//! **Security:** `public/` is for trusted static assets only (icons, fonts, robots.txt,
//! heightmaps). Do not place user-uploaded content here — use `POST /_resuma/upload`
//! instead. `.html` is served as `text/plain` and `.svg` as `application/octet-stream`
//! to reduce stored-XSS risk when opened directly.
//!
//! Large trees: set `RESUMA_PUBLIC_DISK=1` so files larger than 512 KiB are served
//! from disk on each request instead of being loaded fully into RAM at startup.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use walkdir::WalkDir;

/// How a public file is backed.
#[derive(Clone)]
pub enum PublicBody {
    Memory(Arc<Vec<u8>>),
    /// Absolute path read on each request (large assets).
    Disk(PathBuf),
}

/// A file read from `public/` to register as a GET route.
#[derive(Clone)]
pub struct PublicAsset {
    pub url_path: String,
    pub body: PublicBody,
    pub content_type: String,
}

impl PublicAsset {
    pub fn bytes(&self) -> Option<Arc<Vec<u8>>> {
        match &self.body {
            PublicBody::Memory(b) => Some(b.clone()),
            PublicBody::Disk(path) => std::fs::read(path).ok().map(Arc::new),
        }
    }
}

/// Relative paths (under `public/`) that override generated PWA SVG icons when present.
pub const PWA_ICON_CANDIDATES: &[(&str, &str, &str)] = &[
    ("icons/icon-192.png", "/icons/icon-192.png", "192x192"),
    ("icons/icon-512.png", "/icons/icon-512.png", "512x512"),
    (
        "icons/icon-maskable.png",
        "/icons/icon-maskable.png",
        "512x512",
    ),
    (
        "icons/apple-touch-icon.png",
        "/icons/apple-touch-icon.png",
        "180x180",
    ),
    ("icon-192.png", "/icons/icon-192.png", "192x192"),
    ("icon-512.png", "/icons/icon-512.png", "512x512"),
    ("icon.png", "/icons/icon-192.png", "192x192"),
];

fn public_disk_mode() -> bool {
    matches!(
        std::env::var("RESUMA_PUBLIC_DISK").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

fn inline_max_bytes() -> usize {
    std::env::var("RESUMA_PUBLIC_INLINE_MAX")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(512 * 1024)
}

/// Walk `dir` (typically `public/`) and collect servable assets.
pub fn collect_public_dir(dir: &Path) -> Vec<PublicAsset> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let root = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    let disk = public_disk_mode();
    let inline_max = inline_max_bytes();
    let mut out = Vec::new();
    for entry in WalkDir::new(&root)
        .min_depth(1)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_symlink() || !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        match path.canonicalize() {
            Ok(canonical) if canonical.starts_with(&root) => {}
            _ => continue,
        }
        let rel = match path.strip_prefix(&root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let rel_str = rel.to_string_lossy();
        if rel_str.starts_with('.') || rel_str.contains("/.") {
            continue;
        }
        let url_path = format!("/{}", rel_str.replace('\\', "/"));
        let meta = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let content_type = content_type_for_path(path);
        let body = if disk && meta.len() as usize > inline_max {
            PublicBody::Disk(path.to_path_buf())
        } else {
            match std::fs::read(path) {
                Ok(b) => PublicBody::Memory(Arc::new(b)),
                Err(_) => continue,
            }
        };
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
        "html" => "text/plain; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "application/octet-stream",
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
    fn svg_uses_non_executable_content_type() {
        assert_eq!(
            content_type_for_path(Path::new("icon.svg")),
            "application/octet-stream"
        );
        assert_eq!(
            content_type_for_path(Path::new("page.html")),
            "text/plain; charset=utf-8"
        );
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

    #[test]
    fn public_disk_mode_keeps_large_files_on_disk() {
        let dir = std::env::temp_dir().join(format!("resuma-public-disk-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let big = vec![0u8; 1024];
        fs::write(dir.join("big.bin"), &big).unwrap();

        let prev_disk = std::env::var_os("RESUMA_PUBLIC_DISK");
        let prev_max = std::env::var_os("RESUMA_PUBLIC_INLINE_MAX");
        std::env::set_var("RESUMA_PUBLIC_DISK", "1");
        std::env::set_var("RESUMA_PUBLIC_INLINE_MAX", "512");

        let assets = collect_public_dir(&dir);
        assert_eq!(assets.len(), 1);
        assert!(matches!(assets[0].body, PublicBody::Disk(_)));
        let got = assets[0].bytes().expect("disk asset readable");
        assert_eq!(got.as_slice(), big.as_slice());

        match prev_disk {
            Some(v) => std::env::set_var("RESUMA_PUBLIC_DISK", v),
            None => std::env::remove_var("RESUMA_PUBLIC_DISK"),
        }
        match prev_max {
            Some(v) => std::env::set_var("RESUMA_PUBLIC_INLINE_MAX", v),
            None => std::env::remove_var("RESUMA_PUBLIC_INLINE_MAX"),
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
