//! File-based routing for Resuma.
//!
//! Conventions, mirroring SvelteKit / Next App Router so it feels familiar:
//!
//! ```text
//! src/routes/
//!     index.rs            -> /
//!     about.rs            -> /about
//!     users/[id].rs       -> /users/:id
//!     blog/[...slug].rs   -> /blog/*slug
//!     _layout.rs          -> shared layout (wraps siblings + nested routes)
//! ```
//!
//! At build time the CLI scans this directory and generates a Rust module
//! that registers each route on a `ResumaApp`. This crate provides only the
//! scanning / path parsing logic — code generation lives in `resuma-cli`.

use std::path::{Path, PathBuf};

use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredRoute {
    /// Absolute filesystem path to the route file.
    pub file: PathBuf,
    /// URL pattern such as `/users/:id`.
    pub pattern: String,
    /// Module path used by the generated registry (e.g. `users::index`).
    pub module: String,
    /// `true` if this is a layout file (`_layout.rs` / `layout.rs`).
    pub is_layout: bool,
}

pub fn discover<P: AsRef<Path>>(routes_root: P) -> Vec<DiscoveredRoute> {
    let root = routes_root.as_ref();
    let mut out = Vec::new();
    if !root.exists() { return out; }

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() { continue; }
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext != "rs" { continue; }

        let rel = match path.strip_prefix(root) { Ok(r) => r.to_path_buf(), Err(_) => continue };
        if let Some(route) = parse_route(rel.clone(), path.to_path_buf()) {
            out.push(route);
        }
    }
    out.sort_by(|a, b| a.pattern.cmp(&b.pattern));
    out
}

fn parse_route(rel: PathBuf, abs: PathBuf) -> Option<DiscoveredRoute> {
    let stem = rel.file_stem()?.to_str()?;
    let parent = rel.parent().unwrap_or(Path::new("")).to_path_buf();
    let is_layout = stem == "layout" || stem == "_layout";

    let mut segments: Vec<String> = parent
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if !is_layout && stem != "index" {
        segments.push(stem.to_string());
    }

    let pattern = if segments.is_empty() {
        "/".to_string()
    } else {
        let url_segments: Vec<String> = segments
            .iter()
            .map(|s| convert_segment(s))
            .collect();
        format!("/{}", url_segments.join("/"))
    };

    let module = if segments.is_empty() {
        "index".to_string()
    } else {
        segments
            .iter()
            .map(|s| s.replace('[', "_").replace(']', "_").replace("...", "rest_"))
            .collect::<Vec<_>>()
            .join("::")
    };

    Some(DiscoveredRoute { file: abs, pattern, module, is_layout })
}

fn convert_segment(seg: &str) -> String {
    if let Some(inner) = seg.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        if let Some(rest) = inner.strip_prefix("...") {
            return format!("*{}", rest);
        }
        return format!(":{}", inner);
    }
    seg.to_string()
}
