//! `resuma new <name>` - scaffold a brand new Resuma project.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

const BASIC_MAIN: &str = r##"use resuma::prelude::*;

const CSS: &str = r#"<style>
body { font-family: system-ui, sans-serif; max-width: 40rem; margin: 3rem auto; padding: 0 1rem; line-height: 1.6; color: #1e1b4b; }
h1 { margin: 0 0 .5rem; font-size: 2rem; }
p { margin: .5rem 0; color: #4338ca; }
</style>"#;

fn home() -> View {
    view! {
        <main>
            <h1>"Hello, Resuma"</h1>
            <p>"A static page - zero client JavaScript, pure SSR."</p>
            <p>"Add signals, #[server], and islands when you need interactivity."</p>
        </main>
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    ResumaApp::new()
        .with_title("%NAME%")
        .with_head(CSS)
        .page("/", || home())
        .serve(ServeOptions::default())
        .await
}
"##;

/// Full-feature todo showcase (kept in sync with `examples/todo` - update `templates/todo/` when editing the example).
const TODO_MAIN: &str = include_str!("../../templates/todo/main.rs");
const TODO_SECURITY: &str = include_str!("../../templates/todo/security.rs");
const TODO_STORE: &str = include_str!("../../templates/todo/todo_store.rs");

const CARGO_BASIC: &str = r#"[package]
name = "%NAME%"
version = "0.1.0"
edition = "2021"

[dependencies]
resuma = "%RESUMA_VERSION%"
tokio  = { version = "1", features = ["full"] }
"#;

const CARGO_TODO: &str = r#"[package]
name = "%NAME%"
version = "0.1.0"
edition = "2021"

[dependencies]
resuma      = "%RESUMA_VERSION%"
tokio       = { version = "1", features = ["full"] }
serde       = { version = "1", features = ["derive"] }
serde_json  = { version = "1" }
once_cell   = "1"
parking_lot = "0.12"
"#;

const CARGO_FLOW: &str = r#"[package]
name = "%NAME%"
version = "0.1.0"
edition = "2021"

[dependencies]
resuma = "%RESUMA_VERSION%"
tokio  = { version = "1", features = ["full"] }
serde  = { version = "1", features = ["derive"] }
"#;

const FLOW_MAIN: &str = include_str!("../../templates/flow/main.rs");
const FLOW_INDEX: &str = include_str!("../../templates/flow/pages/index.rs");
const FLOW_ABOUT: &str = include_str!("../../templates/flow/pages/about.rs");
const FLOW_MOD: &str = include_str!("../../templates/flow/pages/mod.rs");
const FLOW_REGISTRY: &str = include_str!("../../templates/flow/pages/_registry.rs");

const CARGO_FULLSTACK: &str = r#"[package]
name = "%NAME%"
version = "0.1.0"
edition = "2021"

[dependencies]
resuma = "%RESUMA_VERSION%"
tokio  = { version = "1", features = ["full"] }
serde  = { version = "1", features = ["derive"] }
sqlx   = { version = "0.8", features = ["runtime-tokio", "sqlite", "macros", "migrate"] }
anyhow = "1"
"#;

const FULLSTACK_MAIN: &str = include_str!("../../templates/flow-fullstack/main.rs");
const FULLSTACK_DB: &str = include_str!("../../templates/flow-fullstack/db.rs");
const FULLSTACK_PAGES_MOD: &str = include_str!("../../templates/flow-fullstack/pages/mod.rs");
const FULLSTACK_INDEX: &str = include_str!("../../templates/flow-fullstack/pages/index.rs");
const FULLSTACK_USERS: &str = include_str!("../../templates/flow-fullstack/pages/users.rs");
const FULLSTACK_REGISTRY: &str = include_str!("../../templates/flow-fullstack/pages/_registry.rs");
const FULLSTACK_MIGRATION: &str = include_str!("../../templates/add/sqlx/001_users.sql");

const RUST_TOOLCHAIN: &str = r#"[toolchain]
channel = "stable"
"#;

const README: &str = r##"# %NAME%

Created with [Resuma](https://github.com/GolfredoPerezFernandez/resuma).

## Templates

- **basic** - static SSR page, zero client JS
- **todo** - full Resuma showcase (signals, server, island, security, js!)
- **flow** - multi-page app with `src/pages/` and FlowApp
- **flow-fullstack** - Flow + SQLx (SQLite) with users CRUD sample

## Develop

    resuma dev
    resuma dev --open

## Add integrations

    resuma add sqlx
    resuma add turso

## Build

    resuma build
"##;

fn render_cargo(template: &str, name: &str) -> String {
    template
        .replace("%NAME%", name)
        .replace("%RESUMA_VERSION%", env!("CARGO_PKG_VERSION"))
}

pub fn create_project(name: &str, template: &str) -> Result<()> {
    let dir = Path::new(name);
    if dir.exists() {
        return Err(anyhow!("directory `{}` already exists", name));
    }
    fs::create_dir_all(dir.join("src"))?;

    let readme = README.replace("%NAME%", name);
    fs::write(dir.join("README.md"), readme).context("write README.md")?;
    fs::write(dir.join(".gitignore"), "target/\n").context("write .gitignore")?;
    fs::write(dir.join("rust-toolchain.toml"), RUST_TOOLCHAIN)
        .context("write rust-toolchain.toml")?;

    match template {
        "basic" => {
            fs::write(dir.join("Cargo.toml"), render_cargo(CARGO_BASIC, name))
                .context("write Cargo.toml")?;
            fs::write(dir.join("src/main.rs"), BASIC_MAIN.replace("%NAME%", name))
                .context("write src/main.rs")?;
        }
        "todo" => {
            fs::write(dir.join("Cargo.toml"), render_cargo(CARGO_TODO, name))
                .context("write Cargo.toml")?;
            let main_rs = TODO_MAIN
                .replace("Resuma · Todo", name)
                .replace("example-todo", name);
            fs::write(dir.join("src/main.rs"), main_rs).context("write src/main.rs")?;
            fs::write(dir.join("src/security.rs"), TODO_SECURITY)
                .context("write src/security.rs")?;
            fs::write(dir.join("src/todo_store.rs"), TODO_STORE)
                .context("write src/todo_store.rs")?;
        }
        "flow" => {
            fs::write(dir.join("Cargo.toml"), render_cargo(CARGO_FLOW, name))
                .context("write Cargo.toml")?;
            fs::write(dir.join("src/main.rs"), FLOW_MAIN.replace("%NAME%", name))
                .context("write src/main.rs")?;
            let pages = dir.join("src/pages");
            fs::create_dir_all(&pages)?;
            fs::write(pages.join("mod.rs"), FLOW_MOD).context("write pages/mod.rs")?;
            fs::write(pages.join("_registry.rs"), FLOW_REGISTRY)
                .context("write pages/_registry.rs")?;
            fs::write(pages.join("index.rs"), FLOW_INDEX).context("write pages/index.rs")?;
            fs::write(pages.join("about.rs"), FLOW_ABOUT).context("write pages/about.rs")?;
        }
        "flow-fullstack" => {
            fs::write(dir.join("Cargo.toml"), render_cargo(CARGO_FULLSTACK, name))
                .context("write Cargo.toml")?;
            fs::write(
                dir.join("src/main.rs"),
                FULLSTACK_MAIN.replace("%NAME%", name),
            )
            .context("write src/main.rs")?;
            fs::write(dir.join("src/db.rs"), FULLSTACK_DB).context("write src/db.rs")?;
            let pages = dir.join("src/pages");
            fs::create_dir_all(&pages)?;
            fs::write(pages.join("mod.rs"), FULLSTACK_PAGES_MOD).context("write pages/mod.rs")?;
            fs::write(pages.join("_registry.rs"), FULLSTACK_REGISTRY)
                .context("write pages/_registry.rs")?;
            fs::write(pages.join("index.rs"), FULLSTACK_INDEX).context("write pages/index.rs")?;
            fs::write(pages.join("users.rs"), FULLSTACK_USERS).context("write pages/users.rs")?;
            let mig = dir.join("migrations");
            fs::create_dir_all(&mig)?;
            fs::write(mig.join("001_users.sql"), FULLSTACK_MIGRATION)?;
            fs::write(dir.join(".env.example"), "DATABASE_URL=sqlite:local.db\n").ok();
        }
        other => {
            return Err(anyhow!(
                "unknown template `{}` (try: basic, todo, flow, flow-fullstack)",
                other
            ));
        }
    }

    println!("[resuma] created `{}` (template: {})", name, template);
    println!("\n  cd {}", name);
    println!("  resuma dev      # hot reload at http://127.0.0.1:3000");
    println!("  cargo run       # or plain cargo\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_template_page_returns_view() {
        assert!(BASIC_MAIN.contains("fn home() -> View"));
        assert!(!BASIC_MAIN.contains("fn home() {"));
    }

    #[test]
    fn cargo_templates_pin_the_cli_crate_version() {
        for cargo_template in [CARGO_BASIC, CARGO_TODO, CARGO_FLOW, CARGO_FULLSTACK] {
            let rendered = render_cargo(cargo_template, "smoke-app");
            assert!(rendered.contains("name = \"smoke-app\""));
            assert!(rendered.contains(&format!("\"{}\"", env!("CARGO_PKG_VERSION"))));
            assert!(!rendered.contains("%RESUMA_VERSION%"));
        }
    }
}
