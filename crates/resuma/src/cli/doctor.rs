//! `resuma doctor` — quick environment and project health check.

use std::path::Path;
use std::process::Command;

use anyhow::Result;

use super::ensure_rust_toolchain;
use super::update::{latest_crates_io_version, parse_resuma_dependency, ResumaDependency, CLI_VERSION};

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
                    if v != CLI_VERSION && !v.starts_with(CLI_VERSION) && !CLI_VERSION.starts_with(&v) {
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
            if has_flow_pages(&cwd) && !cwd.join("src/pages/_registry.rs").exists() {
                println!("  src/pages/_registry.rs: missing — run `resuma routes --generate`");
            }
        } else {
            println!("  (not a Resuma app — no resuma in [dependencies])");
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
