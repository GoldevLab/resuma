# Getting started with Resuma

> **Getting Started Resumably** — the Qwik-style quick path to your first resumable Rust app.

Resuma is a **resumable** Rust web framework: no hydration, no eager JS execution. Components run on the server; a tiny client loader resumes interactivity on demand. **Resuma Flow** adds file-based pages, loads, and submits in one crate (like Qwik + Qwik City, unified).

## Try it right away

Rust can't run in-browser playgrounds like Qwik's StackBlitz yet. Clone the repo and run a live example:

```sh
git clone https://github.com/resuma/resuma
cd resuma

cargo run -p example-counter    # minimal counter
cargo run -p example-flow-demo  # full-stack demo
cargo run -p example-website    # this docs site
```

Open http://127.0.0.1:3000 — static pages load **zero client JS**.

## Prerequisites

* [Rust 1.74+](https://rustup.rs) (stable)
* [Node.js 18+](https://nodejs.org) (optional — only to rebuild the JS runtime)
* VS Code + rust-analyzer (recommended)

## Install the CLI

**Published (recommended):**

```sh
cargo install resuma
```

**From source:**

```sh
git clone https://github.com/resuma/resuma
cd resuma
cargo install --path crates/resuma --features cli
resuma --help
```

In app `Cargo.toml`, depend on the library only (skip CLI deps):

```toml
resuma = { version = "0.1", default-features = false }
```

## Create an app using the CLI

```sh
# Counter starter (default)
resuma new my-app
resuma new my-app --template counter

# Full-stack starter (Resuma + Flow)
resuma create my-app --template flow

cd my-app
```

| Template | What you get |
|----------|--------------|
| `counter` | Single page, `ResumaApp`, resumable signals |
| `flow` | `FlowApp`, `src/pages/`, layouts, route registry |

## Start the development server

```sh
resuma dev
# → http://127.0.0.1:3000 with hot reload
```

Or plain `cargo run`.

## Hello, Resuma

```rust
use resuma::prelude::*;

#[component]
fn Hello() -> View {
    let excited = use_signal(false);
    view! {
        <main>
            <h1>"Hello Resuma"</h1>
            <button onClick={ move |_| excited.set(true) }>"Click me"</button>
        </main>
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    ResumaApp::new()
        .page("/", || Hello::render(HelloProps::default()))
        .serve(ServeOptions::default())
        .await
}
```

## Next steps

* [docs/ARCHITECTURE.md](ARCHITECTURE.md) — how resumability works
* [docs/PACKAGE.md](PACKAGE.md) — Resuma¹ + Flow² map
* [docs/FLOW.md](FLOW.md) — loads, submits, middleware
* `/docs/benchmark` on the docs site — bundle sizes vs Qwik

Welcome aboard.
