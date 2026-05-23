# Getting started with Resuma

A 5-minute tour of the framework.

## 1. Install Rust and Node

* [`rustup.rs`](https://rustup.rs) — Rust 1.74+ (stable channel).
* [`nodejs.org`](https://nodejs.org) — Node 18+ (only needed if you want to rebuild the JS runtime; a fallback bundle ships in the repo).

## 2. Build the workspace

```sh
git clone https://github.com/resuma/resuma
cd resuma
cargo build
```

This compiles every crate plus the `resuma` CLI binary.

## 3. Run the counter example

```sh
cargo run -p example-counter
```

Open http://127.0.0.1:3000 and inspect the page:

* Network tab — only `runtime.js` loads (~3KB).
* `<script type="resuma/state">…</script>` — that's the resumability payload.
* Click the button — DevTools shows a single dynamic `import()` and the count updates instantly.

## 4. Build your first component

Create `src/main.rs`:

```rust
use resuma::prelude::*;

#[component]
fn Hello(name: String) -> View {
    let exclaimed = use_signal(false);
    view! {
        <main>
            <h1>"Hello " {name} { if exclaimed.peek() { "!!" } else { "" } }</h1>
            <button onClick={ move |_| exclaimed.set(true) }>"emphasis"</button>
        </main>
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    ResumaApp::new()
        .page("/", || Hello::render(HelloProps { name: "world".into(), ..Default::default() }))
        .serve(ServeOptions::default())
        .await
}
```

`Cargo.toml`:

```toml
[dependencies]
resuma = { path = "../crates/resuma" }
tokio  = { version = "1", features = ["full"] }
```

Run `cargo run`. Done.

## 5. Add a server action

```rust
#[server]
async fn count_words(s: String) -> usize {
    s.split_whitespace().count()
}
```

Inside a `view!`:

```rust
onClick={ js! {
    const n = await __resuma.action('count_words', [state.text.value]);
    state.count.set(n);
}}
```

The action runs on the server, but is invoked from the browser exactly as if it were local.

## 6. Use the CLI

```sh
cargo install --path crates/resuma-cli

resuma new my-app
cd my-app
resuma dev      # → cargo-watch + auto reload
resuma build    # → release binary + JS bundle
resuma routes   # → list discovered file-based routes
```

## 7. Going further

* [`docs/ARCHITECTURE.md`](ARCHITECTURE.md) — how resumability, islands, and the JS bridge actually work.
* [`examples/todo`](../examples/todo) — server actions + state mutation.
* `runtime/src/runtime.ts` — the entire ~3KB client runtime, in heavily commented TypeScript.

Welcome aboard. ✊
