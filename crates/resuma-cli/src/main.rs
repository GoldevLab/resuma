//! `resuma` — Resuma's developer CLI.
//!
//! Subcommands:
//!
//! * `resuma new <name>`   Scaffold a fresh Resuma app.
//! * `resuma dev`          Run `cargo run` with hot reload (cargo watch under the hood).
//! * `resuma build`        Run `cargo build --release` plus the runtime/ts asset pipeline.
//! * `resuma routes`       Inspect file-based routes discovered by `resuma-router`.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

mod scaffold;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "resuma")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new Resuma app from a template.
    New {
        /// Project directory name.
        name: String,
        /// Template name (default: `counter`).
        #[arg(long, default_value = "counter")]
        template: String,
    },
    /// Run the app with hot reload.
    Dev {
        /// Bind address (default: 127.0.0.1:3000).
        #[arg(long, default_value = "127.0.0.1:3000")]
        addr: String,
        /// Skip building the JS runtime (useful when prebuilt).
        #[arg(long)]
        skip_runtime: bool,
    },
    /// Build a production binary + JS bundles.
    Build,
    /// Print the routes discovered by file-based routing.
    Routes {
        /// Path to the `routes` directory (default: `src/routes`).
        #[arg(long, default_value = "src/routes")]
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Cli::parse();
    match args.command {
        Commands::New { name, template } => scaffold::create_project(&name, &template),
        Commands::Dev { addr, skip_runtime } => dev_command(&addr, skip_runtime),
        Commands::Build => build_command(),
        Commands::Routes { path } => routes_command(&path),
    }
}

fn dev_command(addr: &str, skip_runtime: bool) -> Result<()> {
    if !skip_runtime {
        ensure_runtime_built()?;
    }

    println!("[resuma] starting dev server at http://{}", addr);

    // Use cargo-watch when available; fall back to cargo run.
    let has_watch = Command::new("cargo")
        .args(["watch", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let status = if has_watch {
        Command::new("cargo")
            .args(["watch", "-c", "-q", "-x", "run"])
            .env("RESUMA_ADDR", addr)
            .env("RUST_LOG", "info,resuma=debug")
            .status()
            .context("failed to spawn cargo-watch")?
    } else {
        eprintln!("[resuma] cargo-watch not found — install with `cargo install cargo-watch` for hot reload");
        Command::new("cargo")
            .args(["run"])
            .env("RESUMA_ADDR", addr)
            .env("RUST_LOG", "info,resuma=debug")
            .status()
            .context("failed to spawn cargo run")?
    };

    if !status.success() {
        return Err(anyhow!("dev exited with status {:?}", status.code()));
    }
    Ok(())
}

fn build_command() -> Result<()> {
    ensure_runtime_built()?;

    println!("[resuma] cargo build --release");
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .status()
        .context("cargo build failed to start")?;
    if !status.success() {
        return Err(anyhow!("cargo build exited with {:?}", status.code()));
    }
    println!("[resuma] build complete — binaries at target/release/");
    Ok(())
}

fn ensure_runtime_built() -> Result<()> {
    let runtime_dir = Path::new("runtime");
    if !runtime_dir.exists() { return Ok(()); }

    let pkg_lock = runtime_dir.join("node_modules");
    if !pkg_lock.exists() {
        println!("[resuma] installing runtime dependencies (npm install)");
        let status = Command::new(npm_bin())
            .args(["install"])
            .current_dir(runtime_dir)
            .status();
        if let Ok(s) = status {
            if !s.success() {
                eprintln!("[resuma] npm install failed — continuing with fallback runtime");
                return Ok(());
            }
        }
    }

    println!("[resuma] building JS runtime");
    let status = Command::new(npm_bin())
        .args(["run", "build"])
        .current_dir(runtime_dir)
        .status();
    if let Ok(s) = status {
        if s.success() {
            // Copy the bundle into the server's embedded assets.
            let from = runtime_dir.join("dist/runtime.js");
            let to = Path::new("crates/resuma-server/assets/runtime.js");
            if from.exists() {
                if let Some(parent) = to.parent() { std::fs::create_dir_all(parent).ok(); }
                std::fs::copy(&from, to)
                    .with_context(|| format!("failed to copy runtime to {}", to.display()))?;
                println!("[resuma] runtime bundle copied to {}", to.display());
            }
        }
    }
    Ok(())
}

fn npm_bin() -> &'static str {
    if cfg!(windows) { "npm.cmd" } else { "npm" }
}

fn routes_command(path: &Path) -> Result<()> {
    let routes = resuma_router::discover(path);
    if routes.is_empty() {
        println!("[resuma] no routes found under {}", path.display());
        return Ok(());
    }
    println!("[resuma] discovered routes:");
    for r in routes {
        println!(
            "  {:<32} → {} ({}{})",
            r.pattern,
            r.module,
            r.file.display(),
            if r.is_layout { " layout" } else { "" },
        );
    }
    Ok(())
}
