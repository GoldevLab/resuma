//! `resuma new <name>` — scaffold a brand new Resuma project.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

const BASIC_MAIN: &str = r##"use resuma::prelude::*;

const CSS: &str = r#"<style>
body { font-family: system-ui, sans-serif; max-width: 40rem; margin: 3rem auto; padding: 0 1rem; line-height: 1.6; color: #1e1b4b; }
h1 { margin: 0 0 .5rem; font-size: 2rem; }
p { margin: .5rem 0; color: #4338ca; }
</style>"#;

fn Home() -> View {
    view! {
        <main>
            <h1>"Hello, Resuma"</h1>
            <p>"A static page — zero client JavaScript, pure SSR."</p>
            <p>"Add signals, #[server], and islands when you need interactivity."</p>
        </main>
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    ResumaApp::new()
        .with_title("%NAME%")
        .with_head(CSS)
        .page("/", || Home())
        .serve(ServeOptions::default())
        .await
}
"##;

/// Full-feature todo showcase (kept in sync with `examples/todo` — update `templates/todo/` when editing the example).
const TODO_MAIN: &str = include_str!("../templates/todo/main.rs");
const TODO_SECURITY: &str = include_str!("../templates/todo/security.rs");
const TODO_STORE: &str = include_str!("../templates/todo/todo_store.rs");

const CARGO_BASIC: &str = r#"[package]
name = "%NAME%"
version = "0.1.0"
edition = "2021"

[dependencies]
resuma = { version = "0.1", default-features = false }
tokio  = { version = "1", features = ["full"] }
"#;

const CARGO_TODO: &str = r#"[package]
name = "%NAME%"
version = "0.1.0"
edition = "2021"

[dependencies]
resuma      = { version = "0.1", default-features = false }
tokio       = { version = "1", features = ["full"] }
serde       = { version = "1", features = ["derive"] }
serde_json  = { version = "1" }
once_cell   = "1"
parking_lot = "0.12"
"#;

const README: &str = r##"# %NAME%

Created with [Resuma](https://github.com/GolfredoPerezFernandez/resuma).

## Templates

- **basic** - static SSR page, zero client JS
- **todo** - full Resuma showcase (signals, server, island, security, js!)

## Develop

    resuma dev

## Build

    resuma build
"##;

pub fn create_project(name: &str, template: &str) -> Result<()> {
    let dir = Path::new(name);
    if dir.exists() {
        return Err(anyhow!("directory `{}` already exists", name));
    }
    fs::create_dir_all(dir.join("src"))?;

    let readme = README.replace("%NAME%", name);
    fs::write(dir.join("README.md"), readme).context("write README.md")?;
    fs::write(dir.join(".gitignore"), "target/\nCargo.lock\n").context("write .gitignore")?;

    match template {
        "basic" => {
            fs::write(dir.join("Cargo.toml"), CARGO_BASIC.replace("%NAME%", name))
                .context("write Cargo.toml")?;
            fs::write(
                dir.join("src/main.rs"),
                BASIC_MAIN.replace("%NAME%", name),
            )
            .context("write src/main.rs")?;
        }
        "todo" => {
            fs::write(dir.join("Cargo.toml"), CARGO_TODO.replace("%NAME%", name))
                .context("write Cargo.toml")?;
            let main_rs = TODO_MAIN
                .replace("Resuma · Todo", name)
                .replace("example-todo", name);
            fs::write(dir.join("src/main.rs"), main_rs).context("write src/main.rs")?;
            fs::write(dir.join("src/security.rs"), TODO_SECURITY).context("write src/security.rs")?;
            fs::write(dir.join("src/todo_store.rs"), TODO_STORE).context("write src/todo_store.rs")?;
        }
        other => {
            return Err(anyhow!(
                "unknown template `{}` (try: basic, todo)",
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
