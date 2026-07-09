//! Resuma developer CLI — `resuma new`, `resuma dev`, `resuma build`, `resuma routes`.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

mod add;
mod doctor;
mod install;
mod prompt;
mod scaffold;
mod update;

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
        /// Project directory name (prompted when omitted in an interactive terminal).
        name: Option<String>,
        /// Template: `basic`, `todo`, `flow`, `flow-fullstack`, or `production` (prompted when omitted).
        #[arg(long)]
        template: Option<String>,
    },
    /// Add an integration scaffold (sqlx, turso) to the current project.
    Add {
        /// Integration name: `sqlx` or `turso` (prompted when omitted).
        name: Option<String>,
    },
    /// Install editor/agent helpers (skill for Cursor, Codex, etc.).
    Install {
        #[command(subcommand)]
        item: InstallCommands,
    },
    /// Update `resuma` / `resuma-macros` in the current project, or reinstall the CLI.
    Update {
        /// Reinstall the global `resuma` CLI (`cargo install resuma --force`).
        #[arg(long)]
        cli: bool,
        /// Show installed vs available versions without changing anything.
        #[arg(long)]
        check: bool,
        /// Target version (default: this CLI's version).
        #[arg(long)]
        version: Option<String>,
    },
    /// Check Rust toolchain, CLI, and project setup.
    Doctor,
    /// Run the app with hot reload.
    Dev {
        /// Bind address (default: 127.0.0.1:3000). If the port is taken, the next free port is used.
        #[arg(long, default_value = "127.0.0.1:3000")]
        addr: String,
        /// Open the default browser once the server starts.
        #[arg(long)]
        open: bool,
        /// Skip building the JS runtime (useful when prebuilt).
        #[arg(long)]
        skip_runtime: bool,
        /// Kill any process listening on the dev port before starting (Linux).
        #[arg(long)]
        kill_stale: bool,
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
        /// Base URL of a running server to crawl for `--static-export`
        /// (start your app first). Defaults to `RESUMA_EXPORT_BASE_URL` or
        /// `http://127.0.0.1:3000`.
        #[arg(long)]
        base_url: Option<String>,
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

#[derive(Subcommand, Debug)]
enum InstallCommands {
    /// Copy the Resuma agent skill (SKILL.md) for Cursor / Codex / compatible editors.
    Skill {
        /// Install to `.cursor/skills/resuma/` in the current project.
        #[arg(long)]
        project: bool,
        /// Target: `cursor` (default), `project`, `agents`, or `all`.
        #[arg(long, value_name = "TARGET")]
        target: Vec<String>,
        /// Overwrite an existing SKILL.md.
        #[arg(long)]
        force: bool,
        /// List install paths without writing files.
        #[arg(long)]
        list: bool,
    },
}

/// Entry point for the `resuma` binary (`cargo install resuma`).
pub fn run() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Cli::parse();
    match args.command {
        Commands::New { name, template } => new_command(name, template),
        Commands::Add { name } => add_command(name),
        Commands::Install { item } => install_command(item),
        Commands::Update {
            cli,
            check,
            version,
        } => update::update_command(cli, check, version.as_deref()),
        Commands::Doctor => doctor::doctor_command(),
        Commands::Dev {
            addr,
            open,
            skip_runtime,
            kill_stale,
        } => dev_command(&addr, open, skip_runtime, kill_stale),
        Commands::Build {
            static_export,
            out,
            pages,
            base_url,
        } => build_command(static_export, &out, &pages, base_url.as_deref()),
        Commands::Routes { path, generate } => routes_command(&path, generate),
    }
}

fn new_command(name: Option<String>, template: Option<String>) -> Result<()> {
    let name = match name {
        Some(n) => n,
        None if prompt::is_interactive() => prompt::prompt_required("Project name: ")?,
        None => return prompt::missing_arg("project name required — resuma new my-app"),
    };

    let template = match template {
        Some(t) => t,
        None if prompt::is_interactive() => prompt::prompt_template()?,
        None => "basic".to_string(),
    };

    scaffold::create_project(&name, &template)
}

fn add_command(name: Option<String>) -> Result<()> {
    let name = match name {
        Some(n) => n,
        None if prompt::is_interactive() => prompt::prompt_integration()?,
        None => return prompt::missing_arg("integration required — resuma add sqlx"),
    };
    add::add_integration(&name)
}

fn install_command(item: InstallCommands) -> Result<()> {
    match item {
        InstallCommands::Skill {
            project,
            target,
            force,
            list,
        } => {
            let targets = install::parse_targets(&target, project)?;
            install::install_skill(&targets, force, list)
        }
    }
}

/// Ensure `cargo` works — auto-select `stable` when rustup has no default toolchain.
pub(crate) fn ensure_rust_toolchain() -> Result<()> {
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

#[cfg(unix)]
fn kill_stale_port(port: u16) {
    let target = format!("{port}/tcp");
    let status = Command::new("fuser").args(["-k", &target]).status();
    if let Ok(s) = status {
        if s.success() {
            println!("[resuma] freed port {port}");
        }
    }
}

#[cfg(not(unix))]
fn kill_stale_port(_port: u16) {}

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

fn dev_command(addr: &str, open: bool, skip_runtime: bool, kill_stale: bool) -> Result<()> {
    ensure_rust_toolchain()?;
    ensure_cargo_watch()?;
    if !skip_runtime {
        ensure_runtime_built()?;
    }
    maybe_regenerate_routes(Path::new("src/pages"))?;

    let preferred: std::net::SocketAddr = addr
        .parse()
        .map_err(|_| anyhow!("invalid --addr {addr:?} (expected e.g. 127.0.0.1:3000)"))?;
    if kill_stale {
        kill_stale_port(preferred.port());
    }
    let bound = crate::server::resolve_listen_addr(preferred)
        .map_err(|e| anyhow!("could not bind {preferred}: {e}"))?;
    if bound != preferred {
        println!(
            "[resuma] port {} in use, using http://{}",
            preferred.port(),
            bound
        );
    }
    let addr = bound.to_string();
    let url = format!("http://{}", addr);
    println!("[resuma] starting dev server at {}", url);
    if open {
        open_browser(&url);
    }
    std::env::set_var("RESUMA_DEV", "1");

    let mut watch_args = vec![
        "watch".to_string(),
        "-q".to_string(),
        "-w".to_string(),
        "src".to_string(),
        "-w".to_string(),
        "public".to_string(),
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

fn build_command(
    static_export: bool,
    out: &Path,
    pages: &Path,
    base_url: Option<&str>,
) -> Result<()> {
    ensure_rust_toolchain()?;
    maybe_regenerate_routes(pages)?;
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
        static_export_routes(out, pages, base_url)?;
    }

    println!("[resuma] build complete — binaries at target/release/");
    print_predeploy_checklist();
    Ok(())
}

/// Resolve the crawl base URL: CLI flag → `RESUMA_EXPORT_BASE_URL` → localhost.
fn export_base_url(base_url: Option<&str>) -> String {
    base_url
        .map(str::to_string)
        .or_else(|| std::env::var("RESUMA_EXPORT_BASE_URL").ok())
        .unwrap_or_else(|| "http://127.0.0.1:3000".to_string())
}

/// Pre-render discovered static routes by crawling a running server.
///
/// Resuma pages are real Rust modules compiled into the app binary, so the CLI
/// cannot render them in-process. Instead it fetches each route's real SSR HTML
/// over HTTP — start the app (`resuma dev` or the release binary) first.
fn static_export_routes(out: &Path, pages: &Path, base_url: Option<&str>) -> Result<()> {
    use crate::router::discover;

    std::fs::create_dir_all(out).with_context(|| format!("create {}", out.display()))?;

    let routes = discover(pages);
    if routes.is_empty() {
        println!(
            "[resuma] static export: no routes under {}",
            pages.display()
        );
        return Ok(());
    }

    let base = export_base_url(base_url);
    let (host, port) = parse_host_port(&base)
        .with_context(|| format!("invalid base URL `{base}` (expected http://host:port)"))?;

    // Static routes only — params (`:id`) and wildcards (`*`) need request data.
    let exportable: Vec<_> = routes
        .iter()
        .filter(|r| !r.is_layout && !r.pattern.contains(':') && !r.pattern.contains('*'))
        .collect();

    let skipped = routes
        .iter()
        .filter(|r| !r.is_layout && (r.pattern.contains(':') || r.pattern.contains('*')))
        .count();

    println!(
        "[resuma] static export: crawling {} route(s) from {base}",
        exportable.len()
    );

    let mut ok = 0usize;
    for route in &exportable {
        let html = match http_get(&host, port, &route.pattern) {
            Ok(body) => body,
            Err(e) => {
                return Err(anyhow!(
                    "failed to fetch {} from {base}: {e}\n\
                     Start the server first, e.g. `resuma dev` or run the release binary, \
                     then re-run `resuma build --static-export`.",
                    route.pattern
                ));
            }
        };

        let file_path = if route.pattern == "/" {
            out.join("index.html")
        } else {
            out.join(route.pattern.trim_start_matches('/'))
                .join("index.html")
        };
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&file_path, html)
            .with_context(|| format!("write {}", file_path.display()))?;
        println!(
            "[resuma] exported {} -> {}",
            route.pattern,
            file_path.display()
        );
        ok += 1;
    }

    println!(
        "[resuma] static export complete — {ok} page(s) written to {}",
        out.display()
    );
    if skipped > 0 {
        println!(
            "[resuma] {skipped} dynamic route(s) skipped (params/wildcards need runtime data)"
        );
    }
    Ok(())
}

/// Parse `http://host:port` (or `host:port`) into `(host, port)`.
fn parse_host_port(base: &str) -> Option<(String, u16)> {
    let rest = base
        .strip_prefix("http://")
        .or_else(|| base.strip_prefix("https://"))
        .unwrap_or(base)
        .trim_end_matches('/');
    let authority = rest.split('/').next().unwrap_or(rest);
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => (h.to_string(), p.parse().ok()?),
        None => (authority.to_string(), 80),
    };
    if host.is_empty() {
        return None;
    }
    Some((host, port))
}

/// Minimal blocking HTTP/1.1 GET (no TLS) — sufficient for crawling a local
/// Resuma server during static export. Returns the response body on `200 OK`.
fn http_get(host: &str, port: u16, path: &str) -> Result<String> {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let mut stream =
        TcpStream::connect((host, port)).with_context(|| format!("connect to {host}:{port}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(15))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(15))).ok();

    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nAccept: text/html\r\n\
         User-Agent: resuma-build\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes())?;

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw)?;

    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| anyhow!("malformed HTTP response (no header terminator)"))?;
    let header = String::from_utf8_lossy(&raw[..split]);
    let status_ok = header
        .lines()
        .next()
        .map(|line| line.contains(" 200"))
        .unwrap_or(false);
    if !status_ok {
        let status_line = header.lines().next().unwrap_or("(no status)");
        return Err(anyhow!("non-200 response: {status_line}"));
    }

    let body = &raw[split + 4..];
    Ok(String::from_utf8_lossy(body).into_owned())
}

/// Print a concise production pre-deploy checklist after a release build.
fn print_predeploy_checklist() {
    println!();
    println!("Pre-deploy checklist:");
    println!("  [ ] RESUMA_ENV=production set in the runtime environment");
    println!("  [ ] RESUMA_TRUST_PROXY=1 when behind Fly/nginx/Cloudflare");
    println!("  [ ] HOST=0.0.0.0 and PORT set for the platform (Fly sets PORT)");
    println!("  [ ] SITE_URL set to the public origin (sitemap, OG tags)");
    println!("  [ ] Container runs as a non-root user");
    println!("  [ ] /health and /ready wired into platform health checks");
    println!("  [ ] JS runtime assets embedded (loader/core/runtime) — built above");
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
                ("flow.js", runtime_dir.join("dist/flow.js")),
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
    if generate {
        generate_routes(path)?;
        return Ok(());
    }

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

/// Regenerate Flow route scaffolds when pages changed or registry is missing.
pub(crate) fn maybe_regenerate_routes(pages: &Path) -> Result<()> {
    if !pages.is_dir() {
        return Ok(());
    }
    let routes = crate::router::discover(pages);
    if routes.iter().all(|r| r.is_layout) {
        return Ok(());
    }
    if !routes_need_regeneration(pages, &routes)? {
        return Ok(());
    }
    generate_routes(pages)
}

fn routes_need_regeneration(
    pages: &Path,
    routes: &[crate::router::DiscoveredRoute],
) -> Result<bool> {
    let registry = pages.join("_registry.rs");
    let mod_rs = pages.join("mod.rs");
    if !registry.exists() || !mod_rs.exists() {
        return Ok(true);
    }

    let current = routes_fingerprint(routes);
    if read_registry_fingerprint(&registry).as_deref() != Some(current.as_str()) {
        return Ok(true);
    }

    let registry_mtime = newest_mtime(&[registry, mod_rs])?;
    let page_mtime = newest_page_mtime(pages, routes)?;
    Ok(page_mtime > registry_mtime)
}

fn newest_mtime(paths: &[PathBuf]) -> Result<std::time::SystemTime> {
    let mut newest = std::time::SystemTime::UNIX_EPOCH;
    for path in paths {
        if path.exists() {
            let m = std::fs::metadata(path)?.modified()?;
            if m > newest {
                newest = m;
            }
        }
    }
    Ok(newest)
}

fn newest_page_mtime(
    pages: &Path,
    routes: &[crate::router::DiscoveredRoute],
) -> Result<std::time::SystemTime> {
    let mut newest = std::time::SystemTime::UNIX_EPOCH;
    for route in routes {
        if route.is_layout {
            continue;
        }
        let m = std::fs::metadata(&route.file)?.modified()?;
        if m > newest {
            newest = m;
        }
    }
    let _ = pages;
    Ok(newest)
}

/// A page module path is only safe to embed in generated Rust if every `::`
/// segment is a valid Rust identifier. Rejecting anything else stops a crafted
/// filename (containing quotes, spaces, `;`, `}`, …) from injecting tokens into
/// `_registry.rs`, and avoids emitting code that fails to compile with a
/// confusing error (e.g. `my-page.rs`).
fn is_valid_module_path(module: &str) -> bool {
    !module.is_empty()
        && module.split("::").all(|seg| {
            let mut chars = seg.chars();
            matches!(chars.next(), Some(c) if c == '_' || c.is_ascii_alphabetic())
                && chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
        })
}

fn generate_routes(pages: &Path) -> Result<()> {
    let mut routes = crate::router::discover(pages);
    let before = routes.len();
    routes.retain(|r| {
        let ok = is_valid_module_path(&r.module);
        if !ok {
            eprintln!(
                "[resuma] skipping route with invalid module name `{}` (from {}): \
                 page file names must be valid Rust identifiers.",
                r.module,
                r.file.display()
            );
        }
        ok
    });
    if routes.len() != before {
        return Err(anyhow::anyhow!(
            "{} page file(s) have names that are not valid Rust identifiers; \
             rename them (letters, digits, underscore; must not start with a digit) and retry.",
            before - routes.len()
        ));
    }
    if routes.is_empty() {
        println!("[resuma] no routes found under {}", pages.display());
        return Ok(());
    }

    let mod_rs = pages.join("mod.rs");
    let registry = pages.join("_registry.rs");
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

fn routes_fingerprint(routes: &[crate::router::DiscoveredRoute]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    for r in routes {
        if r.is_layout {
            continue;
        }
        r.pattern.hash(&mut hasher);
        r.module.hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

const FINGERPRINT_PREFIX: &str = "// resuma-routes-fingerprint: ";

fn read_registry_fingerprint(registry: &Path) -> Option<String> {
    let first = std::fs::read_to_string(registry).ok()?;
    first
        .lines()
        .next()?
        .strip_prefix(FINGERPRINT_PREFIX)
        .map(str::to_string)
}

fn generate_pages_registry(routes: &[crate::router::DiscoveredRoute]) -> String {
    let fingerprint = routes_fingerprint(routes);
    let layout_routes: Vec<_> = routes
        .iter()
        .filter(|r| r.is_layout)
        .map(|r| (r.pattern.clone(), r.file.clone()))
        .collect();
    let page_routes: Vec<_> = routes.iter().filter(|r| !r.is_layout).collect();

    let mut code = format!(
        "{FINGERPRINT_PREFIX}{fingerprint}\n\
         // Auto-generated by `resuma routes --generate`. Do not edit by hand.\n\
         use resuma::prelude::*;\n\
         use resuma::FlowPageRegistry;\n\n\
         pub struct PagesRegistry;\n\n\
         impl FlowPageRegistry for PagesRegistry {{\n\
             fn routes(&self) -> &'static [(&'static str, &'static str)] {{\n\
                 &[\n",
    );

    for r in &page_routes {
        code.push_str(&format!(
            "                    (\"{}\", \"{}\"),\n",
            r.pattern.replace('\\', "\\\\").replace('"', "\\\""),
            r.module,
        ));
    }

    code.push_str(
        "                ]\n\
             }\n\n\
             fn layout_for(&self, pattern: &str) -> &'static [&'static str] {\n\
                 match pattern {\n",
    );

    for r in &page_routes {
        let chain = crate::router::layout_chain_for(&r.pattern, &layout_routes);
        if chain.is_empty() {
            code.push_str(&format!(
                "                    \"{}\" => &[],\n",
                r.pattern.replace('\\', "\\\\").replace('"', "\\\"")
            ));
        } else {
            let items: Vec<String> = chain
                .iter()
                .map(|p| format!("\"{}\"", p.replace('\\', "\\\\").replace('"', "\\\"")))
                .collect();
            code.push_str(&format!(
                "                    \"{}\" => &[{}],\n",
                r.pattern.replace('\\', "\\\\").replace('"', "\\\""),
                items.join(", ")
            ));
        }
    }

    code.push_str(
        "                    _ => &[],\n\
                 }\n\
             }\n\n\
             fn render(&self, module: &str, req: FlowRequest) -> Option<View> {\n\
                 match module {\n",
    );

    for r in &page_routes {
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
