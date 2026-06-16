# Getting started with Resuma

> Quick path to your first resumable Rust app.

**Published on crates.io:** [resuma 0.2.0](https://crates.io/crates/resuma) · **API docs:** [docs.rs/resuma](https://docs.rs/resuma)

**Interactive docs:** https://resuma-docs.fly.dev/docs · **Local:** clone `site-docs` (sibling repo at `../site-docs`) and `cargo run`

## Try examples

```sh
git clone https://github.com/GoldevLab/resuma
cd resuma

cargo run -p example-todo        # full showcase + backend security reference
cargo run -p example-flow-demo   # FlowApp, loaders, streaming
cargo run -p example-counter     # minimal counter
```

Static pages ship **zero client JS** until you interact.

## Prerequisites

* [Rust 1.91+](https://rustup.rs) — if `cargo` fails with “no default toolchain”, run `rustup default stable`
* [Node.js 18+](https://nodejs.org) (optional — rebuild JS runtime only)

## Install CLI

```sh
cargo install resuma   # from crates.io
resuma --help
```

From source while developing the monorepo:

```sh
cargo install --path crates/resuma --features cli
```

New projects include `rust-toolchain.toml` (stable) so `cargo`/`resuma dev` work even without a global rustup default.

```toml
# App Cargo.toml (what `resuma new` generates)
resuma = "0.4"
tokio  = { version = "1", features = ["full"] }
```

For library-only dependency trees (no CLI crates like clap/notify in the graph):

```toml
resuma = { version = "0.4", default-features = false }
```

## Create a project

```sh
resuma new my-app                  # basic — static SSR
resuma new my-app --template todo  # full showcase + security modules
cd my-app
resuma dev
```

| Template | What you get |
|----------|--------------|
| `basic` | Single SSR page, zero client JS |
| `todo` | Signals, `#[server]`, `#[island]`, security.rs, todo_store.rs |

For multi-page Flow apps, see [FLOW.md](./FLOW.md) and `examples/flow-pages`.

## Hello, Resuma

```rust
use resuma::prelude::*;

#[component]
fn Hello() -> View {
    let excited = signal(false);
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

* [ARCHITECTURE.md](./ARCHITECTURE.md) — resumability
* [SECURITY.md](./SECURITY.md) + [BACKEND.md](./BACKEND.md) — production hardening (`examples/todo`)
* [FLOW.md](./FLOW.md) — multi-page apps
* [docs/README.md](./README.md) — full doc index + `/docs/examples`
