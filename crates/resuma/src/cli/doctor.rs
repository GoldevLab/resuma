//! `resuma doctor` — quick environment and project health check.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;

use super::ensure_rust_toolchain;
use super::update::{
    latest_crates_io_version, parse_resuma_dependency, ResumaDependency, CLI_VERSION,
};

// Keep in sync with `runtime/scripts/measure.mjs` and `docs/STABILITY.md`.
const LOADER_GZIP_BUDGET: usize = 1100;
const CORE_GZIP_BUDGET: usize = 8500;

pub fn doctor_command() -> Result<()> {
    println!("[resuma] doctor\n");

    check_tool("rustc", &["--version"], "Rust compiler");
    check_tool("cargo", &["--version"], "Cargo");
    let _ = ensure_rust_toolchain();

    let watch_ok = Command::new("cargo")
        .args(["watch", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    println!(
        "  cargo-watch: {}",
        if watch_ok {
            "installed (hot reload ready)"
        } else {
            "missing — `resuma dev` will install it automatically"
        }
    );

    let cli_on_path = Command::new("resuma")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if cli_on_path {
        let ver = Command::new("resuma")
            .arg("--version")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        println!("  resuma CLI (PATH): {ver}");
    } else {
        println!("  resuma CLI (PATH): not found — run `cargo install resuma`");
    }
    println!("  resuma CLI (this binary): v{CLI_VERSION}");

    if let Some(latest) = latest_crates_io_version("resuma") {
        println!("  resuma (crates.io): v{latest}");
        if latest != CLI_VERSION {
            println!("  → newer CLI available: resuma update --cli");
        }
    }

    check_runtime_bundle_sizes();

    let cwd = std::env::current_dir()?;
    let cargo_path = cwd.join("Cargo.toml");
    if cargo_path.exists() {
        println!();
        println!("Project: {}", cwd.display());
        let cargo = std::fs::read_to_string(&cargo_path).unwrap_or_default();
        if let Some(dep) = parse_resuma_dependency(&cargo) {
            match dep {
                ResumaDependency::Version(v) => {
                    println!("  resuma dependency: v{v}");
                    if v != CLI_VERSION
                        && !v.starts_with(CLI_VERSION)
                        && !CLI_VERSION.starts_with(&v)
                    {
                        println!("  → run `resuma update` to align with CLI v{CLI_VERSION}");
                    }
                }
                ResumaDependency::Path(path) => {
                    println!("  resuma dependency: path ({path})");
                }
            }
            if !cwd.join("rust-toolchain.toml").exists() {
                println!("  rust-toolchain.toml: missing (optional but recommended)");
            }
            if has_flow_pages(&cwd) {
                check_flow_pages(&cwd);
            }
            check_client_assets(&cwd);
        } else {
            println!("  (not a Resuma app — no resuma in [dependencies])");
        }

        if std::env::var("RESUMA_ENV").as_deref() == Ok("production") {
            println!("  RESUMA_ENV: production");
        } else {
            println!("  RESUMA_ENV: not production (dev defaults — set RESUMA_ENV=production for deploy)");
        }
    } else {
        println!();
        println!("Project: (no Cargo.toml in current directory)");
    }

    println!("\n[resuma] doctor complete");
    Ok(())
}

fn check_tool(bin: &str, args: &[&str], label: &str) {
    match Command::new(bin).args(args).output() {
        Ok(o) if o.status.success() => {
            let line = String::from_utf8_lossy(&o.stdout).trim().to_string();
            println!("  {label}: {line}");
        }
        _ => println!("  {label}: not found"),
    }
}

fn has_flow_pages(root: &Path) -> bool {
    root.join("src/pages").is_dir()
}

fn check_flow_pages(root: &Path) {
    let pages = root.join("src/pages");
    let registry = pages.join("_registry.rs");
    if !registry.exists() {
        println!("  src/pages/_registry.rs: missing — run `resuma routes --generate`");
        return;
    }

    let discovered: Vec<String> = crate::router::discover(&pages)
        .into_iter()
        .filter(|r| !r.is_layout)
        .map(|r| r.module)
        .collect();

    let registered = parse_registry_modules(&registry);
    let missing: Vec<_> = discovered
        .iter()
        .filter(|m| !registered.contains(m))
        .cloned()
        .collect();
    let stale: Vec<_> = registered
        .iter()
        .filter(|m| !discovered.contains(m))
        .cloned()
        .collect();

    if missing.is_empty() && stale.is_empty() {
        println!(
            "  src/pages/_registry.rs: in sync ({} routes)",
            discovered.len()
        );
    } else {
        if !missing.is_empty() {
            println!(
                "  src/pages/_registry.rs: stale — missing modules: {}",
                missing.join(", ")
            );
        }
        if !stale.is_empty() {
            println!(
                "  src/pages/_registry.rs: stale — removed modules still registered: {}",
                stale.join(", ")
            );
        }
        println!("  → run `resuma routes --generate` (or `resuma dev` / `resuma build` to auto-regenerate)");
    }
}

fn parse_registry_modules(registry: &Path) -> Vec<String> {
    let content = std::fs::read_to_string(registry).unwrap_or_default();
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if !line.contains("=> Some") {
                return None;
            }
            let rest = line.strip_prefix('"')?;
            let module = rest.split('"').next()?;
            Some(module.to_string())
        })
        .collect()
}

fn check_runtime_bundle_sizes() {
    let dist = runtime_dist_dir();
    let loader = dist.join("loader.js");
    let core = dist.join("core.js");

    if !loader.exists() || !core.exists() {
        println!(
            "  runtime bundles: not built — run `npm run build` in runtime/ (or `resuma dev`)"
        );
        return;
    }

    match (gzip_len(&loader), gzip_len(&core)) {
        (Ok(loader_gz), Ok(core_gz)) => {
            let loader_ok = loader_gz <= LOADER_GZIP_BUDGET;
            let core_ok = core_gz <= CORE_GZIP_BUDGET;
            println!(
                "  runtime loader.js: {} gzip (budget {} B){}",
                fmt_bytes(loader_gz),
                LOADER_GZIP_BUDGET,
                if loader_ok { "" } else { " — OVER BUDGET" }
            );
            println!(
                "  runtime core.js:   {} gzip (budget {} B){}",
                fmt_bytes(core_gz),
                CORE_GZIP_BUDGET,
                if core_ok { "" } else { " — OVER BUDGET" }
            );
        }
        _ => println!("  runtime bundles: could not measure gzip sizes"),
    }
}

fn runtime_dist_dir() -> PathBuf {
    let monorepo = Path::new("runtime/dist");
    if monorepo.is_dir() {
        return monorepo.to_path_buf();
    }
    PathBuf::from(".resuma/assets")
}

fn gzip_len(path: &Path) -> Result<usize, std::io::Error> {
    use std::io::Write;
    let raw = std::fs::read(path)?;
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&raw)?;
    Ok(encoder.finish()?.len())
}

fn fmt_bytes(n: usize) -> String {
    if n < 1024 {
        format!("{n} B")
    } else {
        format!("{:.2} KiB", n as f64 / 1024.0)
    }
}

/// Scans `static/client/*.js` for files no `include_bytes!(...)` in `src/` points
/// at — i.e. files that were `client_asset`-registered once and then abandoned
/// when the app hand-rolled a `v2`/`v3` rename instead of relying on Resuma's
/// content-digest cache busting. This exact drift (~250 orphaned files, tens of
/// MB) is what motivated the check: see `resuma` CHANGELOG `[Unreleased]`.
fn check_client_assets(root: &Path) {
    let client_dir = root.join("static/client");
    let Ok(entries) = std::fs::read_dir(&client_dir) else {
        return;
    };

    let referenced = referenced_client_asset_paths(root);
    let mut orphaned: Vec<String> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("js"))
        .filter(|path| {
            let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
            !referenced.contains(&canonical)
        })
        .filter_map(|path| path.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    orphaned.sort();

    if orphaned.is_empty() {
        println!("  static/client/: no orphaned assets");
        return;
    }
    println!(
        "  static/client/: {} file(s) not referenced by any client_asset(...) / include_bytes!(...) in src/:",
        orphaned.len()
    );
    for name in orphaned.iter().take(20) {
        println!("    - {name}");
    }
    if orphaned.len() > 20 {
        println!("    ...and {} more", orphaned.len() - 20);
    }
    println!(
        "  → delete unused files, or register them via .client_asset(\"id\", include_bytes!(...))"
    );
}

/// Resolved, canonicalized filesystem paths every `include_bytes!("...")` literal
/// under `root/src` points at. `include_bytes!` paths are relative to the file
/// they appear in (not the crate root), so resolution must happen per-file.
fn referenced_client_asset_paths(root: &Path) -> std::collections::HashSet<PathBuf> {
    let mut referenced = std::collections::HashSet::new();
    let src_dir = root.join("src");
    if !src_dir.is_dir() {
        return referenced;
    }
    for entry in walkdir::WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        let Some(parent) = path.parent() else {
            continue;
        };
        for rel in extract_include_bytes_paths(&content) {
            let resolved = parent.join(&rel);
            referenced.insert(resolved.canonicalize().unwrap_or(resolved));
        }
    }
    referenced
}

/// Extracts string-literal path arguments from `include_bytes!("...")` call
/// sites via plain text search (fast, dependency-free — good enough for a
/// health-check heuristic; not a full Rust parser).
fn extract_include_bytes_paths(content: &str) -> Vec<String> {
    const NEEDLE: &str = "include_bytes!";
    let mut out = Vec::new();
    let mut cursor = 0usize;
    while let Some(found) = content[cursor..].find(NEEDLE) {
        let after_needle = cursor + found + NEEDLE.len();
        let rest = &content[after_needle..];
        let Some(open) = rest.find('"') else { break };
        let Some(close) = rest[open + 1..].find('"') else {
            break;
        };
        out.push(rest[open + 1..open + 1 + close].to_string());
        cursor = after_needle + open + 1 + close + 1;
    }
    out
}

#[cfg(test)]
mod client_asset_tests {
    use super::*;

    #[test]
    fn extracts_single_and_multiple_include_bytes_paths() {
        let src = r#"
            .client_asset("fw-item-icons", include_bytes!("../static/client/fw-item-icons.js"))
            .client_asset(
                "fw-meadow-gpu",
                include_bytes!("../static/client/fw-meadow-gpu.js"),
            )
        "#;
        assert_eq!(
            extract_include_bytes_paths(src),
            vec![
                "../static/client/fw-item-icons.js".to_string(),
                "../static/client/fw-meadow-gpu.js".to_string(),
            ]
        );
    }

    #[test]
    fn extracts_nothing_when_absent() {
        assert!(extract_include_bytes_paths("fn main() {}").is_empty());
    }

    #[test]
    fn flags_orphaned_js_files_not_reachable_from_src() {
        let root = std::env::temp_dir().join(format!(
            "resuma-doctor-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let client_dir = root.join("static/client");
        let src_dir = root.join("src");
        std::fs::create_dir_all(&client_dir).unwrap();
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(client_dir.join("live.js"), b"// live").unwrap();
        std::fs::write(client_dir.join("dead-v1.js"), b"// dead").unwrap();
        std::fs::write(
            src_dir.join("main.rs"),
            br#"fn register() { let _ = include_bytes!("../static/client/live.js"); }"#,
        )
        .unwrap();

        let referenced = referenced_client_asset_paths(&root);
        let live_canonical = client_dir.join("live.js").canonicalize().unwrap();
        let dead_canonical = client_dir.join("dead-v1.js").canonicalize().unwrap();
        assert!(referenced.contains(&live_canonical));
        assert!(!referenced.contains(&dead_canonical));

        std::fs::remove_dir_all(&root).unwrap();
    }
}
