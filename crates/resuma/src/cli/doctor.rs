//! `resuma doctor` — quick environment and project health check.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;

use super::ensure_rust_toolchain;
use super::update::{
    latest_crates_io_version, parse_resuma_dependency, ResumaDependency, CLI_VERSION,
};

const LOADER_GZIP_BUDGET: usize = 1024;
const CORE_GZIP_BUDGET: usize = 5700;

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
        println!("  src/pages/_registry.rs: in sync ({} routes)", discovered.len());
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
        println!("  runtime bundles: not built — run `npm run build` in runtime/ (or `resuma dev`)");
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
