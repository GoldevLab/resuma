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
resuma = { version = "0.1" }
tokio  = { version = "1", features = ["full"] }
"#;

const README: &str = r#"# %NAME%

Created with [Resuma](https://github.com/resuma/resuma) — the first Rust
framework with **SSR + Resumability + Islands + Server Actions + JS bridge**.

## Develop

```sh
resuma dev
```

## Build

```sh
resuma build
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

    let main_rs = match template {
        "counter" => COUNTER_MAIN,
        other => return Err(anyhow!("unknown template `{}`", other)),
    };
    fs::write(dir.join("src/main.rs"), main_rs).context("write src/main.rs")?;

    println!("[resuma] created `{}` (template: {})", name, template);
    println!("\n  cd {}\n  resuma dev\n", name);
    Ok(())
}
