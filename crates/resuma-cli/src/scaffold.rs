//! `resuma new <name>` — scaffold a brand new Resuma project.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

const COUNTER_MAIN: &str = r#"use resuma::prelude::*;

#[component]
fn Counter() -> View {
    let count = use_signal(0_i32);
    view! {
        <main class="card">
            <h1>"Resuma Counter"</h1>
            <p>"Count: " {count}</p>
            <button onClick={ move |_| count.update(|c| *c += 1) }>"+"</button>
            <button onClick={ move |_| count.update(|c| *c -= 1) }>"-"</button>
        </main>
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    ResumaApp::new()
        .with_title("Resuma Counter")
        .page("/", || Counter::render(CounterProps::default()))
        .serve(ServeOptions::default())
        .await
}
"#;

const CARGO_TOML: &str = r#"[package]
name = "%NAME%"
version = "0.1.0"
edition = "2021"

[dependencies]
resuma = { version = "0.1", default-features = false }
tokio  = { version = "1", features = ["full"] }
"#;

const FLOW_MAIN: &str = r##"mod pages;

use resuma::prelude::*;
use pages::PagesRegistry;

#[layout("/")]
fn AppLayout() -> View {
    view! {
        <div class="shell">
            <nav class="nav"><NavLink href="/" activeClass="active">"Home"</NavLink></nav>
            <Slot />
        </div>
    }
}

const CSS: &str = r#"<style>
body { font-family: system-ui, sans-serif; max-width: 40rem; margin: 2rem auto; padding: 0 1rem; }
.nav a { margin-right: 1rem; color: #6366f1; text-decoration: none; }
.nav a.active { font-weight: 700; }
</style>"#;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    FlowApp::new()
        .with_title("%NAME%")
        .with_head(CSS)
        .auto_pages(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pages"),
            PagesRegistry,
        )
        .serve(FlowServeOptions::default())
        .await
}
"##;

const FLOW_PAGE_INDEX: &str = r#"use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <main>
            <h1>"Welcome to Resuma Flow"</h1>
            <p>"Edit " <code>"src/pages/index.rs"</code> " to get started."</p>
        </main>
    }
}
"#;

const FLOW_PAGES_MOD: &str = r#"pub mod index;
mod _registry;
pub use _registry::PagesRegistry;
"#;

const FLOW_PAGES_REGISTRY: &str = r#"use resuma::prelude::*;
use resuma::FlowPageRegistry;

pub struct PagesRegistry;

impl FlowPageRegistry for PagesRegistry {
    fn render(&self, module: &str, req: FlowRequest) -> Option<View> {
        match module {
            "index" => Some(super::index::page(req)),
            _ => None,
        }
    }
}
"#;

const FLOW_LAYOUT_MARKER: &str = r#"//! Layout marker — pairs with `#[layout("/")]` in `main.rs`.
"#;

const README: &str = r#"# %NAME%

Created with [Resuma](https://github.com/resuma/resuma) — SSR + Resumability + Flow in one crate.

## Develop

```sh
resuma dev
```

## Build

```sh
resuma build
```

## Add pages (Flow template)

```sh
resuma routes --generate --path src/pages
```
"#;

pub fn create_project(name: &str, template: &str) -> Result<()> {
    let dir = Path::new(name);
    if dir.exists() {
        return Err(anyhow!("directory `{}` already exists", name));
    }
    fs::create_dir_all(dir.join("src"))?;

    let cargo_toml = CARGO_TOML.replace("%NAME%", name);
    let readme = README.replace("%NAME%", name);

    fs::write(dir.join("Cargo.toml"), cargo_toml).context("write Cargo.toml")?;
    fs::write(dir.join("README.md"), readme).context("write README.md")?;
    fs::write(dir.join(".gitignore"), "target/\nCargo.lock\n").context("write .gitignore")?;

    match template {
        "counter" => {
            fs::write(dir.join("src/main.rs"), COUNTER_MAIN).context("write src/main.rs")?;
        }
        "flow" => {
            fs::create_dir_all(dir.join("src/pages"))?;
            fs::write(
                dir.join("src/main.rs"),
                FLOW_MAIN.replace("%NAME%", name),
            )
            .context("write src/main.rs")?;
            fs::write(dir.join("src/pages/index.rs"), FLOW_PAGE_INDEX)
                .context("write src/pages/index.rs")?;
            fs::write(dir.join("src/pages/layout.rs"), FLOW_LAYOUT_MARKER)
                .context("write src/pages/layout.rs")?;
            fs::write(dir.join("src/pages/mod.rs"), FLOW_PAGES_MOD)
                .context("write src/pages/mod.rs")?;
            fs::write(dir.join("src/pages/_registry.rs"), FLOW_PAGES_REGISTRY)
                .context("write src/pages/_registry.rs")?;
        }
        other => {
            return Err(anyhow!(
                "unknown template `{}` (try: counter, flow)",
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
