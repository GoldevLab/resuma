//! Resuma developer CLI — `resuma new`, `resuma dev`, `resuma build`, `resuma routes`.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

mod scaffold;
mod add;

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
    #[command(alias = "create")]
    New {
        /// Project directory name.
        name: String,
        /// Template: `basic`, `todo`, `flow`, or `flow-fullstack` (Flow + SQLx).
        #[arg(long, default_value = "basic")]
        template: String,
    },
    /// Add an integration scaffold (sqlx, turso) to the current project.
    Add {
        /// Integration name: `sqlx` or `turso`.
        name: String,
    },
    /// Run the app with hot reload.
    Dev {
        /// Bind address (default: 127.0.0.1:3000).
        #[arg(long, default_value = "127.0.0.1:3000")]
        addr: String,
        /// Open the default browser once the server starts.
        #[arg(long)]
        open: bool,
        /// Skip building the JS runtime (useful when prebuilt).
        #[arg(long)]
        skip_runtime: bool,
    },
    /// Build a production binary + JS bundles.
    Build {
        /// Pre-render static HTML for discovered routes into `--out`.
        #[arg(long)]
        static_export: bool,
        /// Output directory for static export (default: `dist`).
        #[arg(long, default_value = "dist")]
        out: PathBuf,
        /// Pages directory for Flow static export route discovery.
        #[arg(long, default_value = "src/pages")]
        pages: PathBuf,
    },
    /// Print or generate routes from file-based routing.
    Routes {
        /// Path to the pages directory (default: `src/pages`).
        #[arg(long, default_value = "src/pages")]
        path: PathBuf,
        /// Generate `src/pages/_registry.rs` scaffold.
        #[arg(long)]
        generate: bool,
    },
}

/// Entry point for the `resuma` binary (`cargo install resuma`).
pub fn run() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Cli::parse();
    match args.command {
        Commands::New { name, template } => scaffold::create_project(&name, &template),
        Commands::Add { name } => add::add_integration(&name),
        Commands::Dev {
            addr,
            open,
            skip_runtime,
        } => dev_command(&addr, open, skip_runtime),
        Commands::Build {
            static_export,
            out,
            pages,
        } => build_command(static_export, &out, &pages),
        Commands::Routes { path, generate } => routes_command(&path, generate),
    }
}

/// Ensure `cargo` works — auto-select `stable` when rustup has no default toolchain.
fn ensure_rust_toolchain() -> Result<()> {
    let probe = Command::new("cargo")
        .arg("--version")
        .output()
        .context("cargo not found — install Rust from https://rustup.rs")?;

    if probe.status.success() {
        return Ok(());
    }

    let err = format!(
        "{}{}",
        String::from_utf8_lossy(&probe.stderr),
        String::from_utf8_lossy(&probe.stdout)
    );
    let needs_default = err.contains("no default is configured")
        || err.contains("could not choose a version of cargo");

    if !needs_default {
        return Err(anyhow!(
            "cargo failed: {}",
            err.lines().next().unwrap_or("unknown error")
        ));
    }

    eprintln!("[resuma] no default Rust toolchain — running `rustup default stable`…");
    let status = Command::new("rustup")
        .args(["default", "stable"])
        .status()
        .context("rustup not found — install from https://rustup.rs")?;
    if !status.success() {
        return Err(anyhow!(
            "could not set default toolchain — run: rustup default stable"
        ));
    }

    let verify = Command::new("cargo")
        .arg("--version")
        .output()
        .context("cargo not found after rustup default stable")?;
    if verify.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "cargo still unavailable after `rustup default stable` — restart your terminal and try again"
        ))
    }
}

fn ensure_cargo_watch() -> Result<()> {
    let ok = Command::new("cargo")
        .args(["watch", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if ok {
        return Ok(());
    }

    eprintln!("[resuma] installing cargo-watch (one-time) for hot reload…");
    let status = Command::new("cargo")
        .args(["install", "cargo-watch"])
        .status()
        .context("failed to run cargo install cargo-watch")?;
    if !status.success() {
        return Err(anyhow!(
            "could not install cargo-watch — run manually: cargo install cargo-watch"
        ));
    }
    Ok(())
}

fn dev_command(addr: &str, open: bool, skip_runtime: bool) -> Result<()> {
    ensure_rust_toolchain()?;
    ensure_cargo_watch()?;
    if !skip_runtime {
        ensure_runtime_built()?;
    }

    let url = format!("http://{}", addr);
    println!("[resuma] starting dev server at {}", url);
    if open {
        open_browser(&url);
    }
    std::env::set_var("RESUMA_DEV", "1");

    let mut watch_args = vec![
        "watch".to_string(),
        "-c".to_string(),
        "-q".to_string(),
        "-w".to_string(),
        "src".to_string(),
        "-w".to_string(),
        "Cargo.toml".to_string(),
    ];
    #[cfg(windows)]
    watch_args.push("--poll".to_string());
    watch_args.push("-x".to_string());
    watch_args.push("run".to_string());

    println!("[resuma] hot reload enabled — save a file to rebuild and refresh the browser");

    let status = Command::new("cargo")
        .args(&watch_args)
        .env("RESUMA_ADDR", addr)
        .env("RESUMA_DEV", "1")
        .env("RUST_LOG", "info,resuma=debug")
        .status()
        .context("failed to spawn cargo watch")?;

    if !status.success() {
        return Err(anyhow!("dev exited with status {:?}", status.code()));
    }
    Ok(())
}

fn open_browser(url: &str) {
    #[cfg(windows)]
    let _ = Command::new("cmd").args(["/C", "start", "", url]).spawn();
    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg(url).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let _ = Command::new("xdg-open").arg(url).spawn();
}

fn build_command(static_export: bool, out: &Path, pages: &Path) -> Result<()> {
    ensure_rust_toolchain()?;
    ensure_runtime_built()?;

    println!("[resuma] cargo build --release");
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .status()
        .context("cargo build failed to start")?;
    if !status.success() {
        return Err(anyhow!("cargo build exited with {:?}", status.code()));
    }

    if static_export {
        static_export_routes(out, pages)?;
    }

    println!("[resuma] build complete — binaries at target/release/");
    Ok(())
}

fn static_export_routes(out: &Path, pages: &Path) -> Result<()> {
    use crate::router::discover;
    use crate::ssr::{render_to_string_at_path, PageOptions};

    std::fs::create_dir_all(out).with_context(|| format!("create {}", out.display()))?;

    let routes = discover(pages);
    if routes.is_empty() {
        println!(
            "[resuma] static export: no routes under {}",
            pages.display()
        );
        return Ok(());
    }

    let opts = PageOptions {
        title: "Static Export".into(),
        ..Default::default()
    };

    for route in routes.iter().filter(|r| !r.is_layout) {
        let file_path = if route.pattern == "/" {
            out.join("index.html")
        } else {
            out.join(route.pattern.trim_start_matches('/'))
                .join("index.html")
        };
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let pattern = route.pattern.clone();
        let label = pattern.clone();
        let html = render_to_string_at_path(&opts, &pattern, move || {
            crate::core::View::Text(format!(
                "Static export shell for {label} — customize `resuma build --static` with your page factories."
            ))
        });
        std::fs::write(&file_path, html)
            .with_context(|| format!("write {}", file_path.display()))?;
        println!("[resuma] exported {}", route.pattern);
    }

    Ok(())
}

fn ensure_runtime_built() -> Result<()> {
    let runtime_dir = Path::new("runtime");
    if !runtime_dir.exists() {
        return Ok(());
    }

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
            let assets = [
                ("runtime.js", runtime_dir.join("dist/runtime.js")),
                ("loader.js", runtime_dir.join("dist/loader.js")),
                ("core.js", runtime_dir.join("dist/core.js")),
            ];
            for (name, from) in assets {
                let to = runtime_assets_dir().join(name);
                if from.exists() {
                    if let Some(parent) = to.parent() {
                        std::fs::create_dir_all(parent).ok();
                    }
                    std::fs::copy(&from, &to)
                        .with_context(|| format!("failed to copy {name} to {}", to.display()))?;
                    println!("[resuma] {name} copied to {}", to.display());
                }
            }
        }
    }
    Ok(())
}

fn npm_bin() -> &'static str {
    if cfg!(windows) {
        "npm.cmd"
    } else {
        "npm"
    }
}

/// Where to copy rebuilt JS when developing the Resuma monorepo vs a standalone app.
fn runtime_assets_dir() -> PathBuf {
    let monorepo = Path::new("crates/resuma/assets");
    if monorepo.is_dir() {
        monorepo.to_path_buf()
    } else {
        PathBuf::from(".resuma/assets")
    }
}

fn routes_command(path: &Path, generate: bool) -> Result<()> {
    let routes = crate::router::discover(path);
    if routes.is_empty() {
        println!("[resuma] no routes found under {}", path.display());
        return Ok(());
    }

    let layouts_index: Vec<_> = routes
        .iter()
        .filter(|x| x.is_layout)
        .map(|x| (x.pattern.clone(), x.file.clone()))
        .collect();

    if generate {
        let mod_rs = path.join("mod.rs");
        let registry = path.join("_registry.rs");
        let mod_code = generate_pages_mod(&routes);
        let registry_code = generate_pages_registry(&routes);

        std::fs::write(&mod_rs, mod_code)
            .with_context(|| format!("failed to write {}", mod_rs.display()))?;
        std::fs::write(&registry, registry_code)
            .with_context(|| format!("failed to write {}", registry.display()))?;
        println!(
            "[resuma] generated {} and {}",
            mod_rs.display(),
            registry.display()
        );
    }

    println!("[resuma] discovered routes:");
    for r in &routes {
        let layouts = if r.is_layout {
            Vec::new()
        } else {
            crate::router::layout_chain_for(&r.pattern, &layouts_index)
        };
        println!(
            "  {:<32} → {} ({}{}) layouts={:?}",
            r.pattern,
            r.module,
            r.file.display(),
            if r.is_layout { " layout" } else { "" },
            if r.is_layout {
                Vec::<String>::new()
            } else {
                layouts
            },
        );
    }
    Ok(())
}

use std::collections::BTreeMap;

#[derive(Default)]
struct ModTree {
    children: BTreeMap<String, ModTree>,
    is_module: bool,
}

fn insert_mod_path(tree: &mut ModTree, parts: &[&str]) {
    if parts.is_empty() {
        return;
    }
    let node = tree.children.entry(parts[0].to_string()).or_default();
    if parts.len() == 1 {
        node.is_module = true;
    } else {
        insert_mod_path(node, &parts[1..]);
    }
}

fn emit_mod_tree(tree: &ModTree, depth: usize) -> String {
    let indent = "    ".repeat(depth);
    let mut out = String::new();
    for (name, child) in &tree.children {
        if child.children.is_empty() {
            out.push_str(&format!("{indent}pub mod {name};\n"));
        } else {
            out.push_str(&format!("{indent}pub mod {name} {{\n"));
            out.push_str(&emit_mod_tree(child, depth + 1));
            out.push_str(&format!("{indent}}}\n"));
        }
    }
    out
}

fn generate_pages_mod(routes: &[crate::router::DiscoveredRoute]) -> String {
    let mut tree = ModTree::default();
    for r in routes {
        if r.is_layout || r.module == "_registry" {
            continue;
        }
        let mut mod_path = r.module.clone();
        if r.file
            .file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s == "index")
            && r.module != "index"
        {
            mod_path = format!("{}::index", r.module);
        }
        let parts: Vec<&str> = mod_path.split("::").collect();
        insert_mod_path(&mut tree, &parts);
    }

    format!(
        "// Auto-generated by `resuma routes --generate`. Do not edit by hand.\n\n\
         {}\n\
         mod _registry;\n\
         pub use _registry::PagesRegistry;\n",
        emit_mod_tree(&tree, 0),
    )
}

fn module_rust_path(route: &crate::router::DiscoveredRoute) -> String {
    let stem = route
        .file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if stem == "index" && route.module != "index" {
        format!("super::{}::index", route.module)
    } else {
        format!("super::{}", route.module)
    }
}

fn generate_pages_registry(routes: &[crate::router::DiscoveredRoute]) -> String {
    let mut code = String::from(
        "// Auto-generated by `resuma routes --generate`. Do not edit by hand.\n\
         use resuma::prelude::*;\n\
         use resuma::FlowPageRegistry;\n\n\
         pub struct PagesRegistry;\n\n\
         impl FlowPageRegistry for PagesRegistry {\n\
             fn render(&self, module: &str, req: FlowRequest) -> Option<View> {\n\
                 match module {\n",
    );

    for r in routes {
        if r.is_layout {
            continue;
        }
        let path = module_rust_path(r);
        code.push_str(&format!(
            "            \"{}\" => Some({path}::page(req)),\n",
            r.module,
            path = path,
        ));
    }

    code.push_str(
        "            _ => None,\n\
             }\n\
             }\n\
             }\n",
    );
    code
}
