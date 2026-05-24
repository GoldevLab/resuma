# Resuma

Rust web framework with SSR, resumability, islands, server actions and lazy JS handlers.

**No hydration. No WASM bundle by default. ~3KB runtime.**

```bash
cargo install resuma
resuma new my-app --template todo   # or just `resuma new` for interactive prompts
cd my-app
resuma dev --open
```

Build a counter:

```rust
use resuma::prelude::*;

#[component]
fn Counter() -> View {
    let count = use_signal(0);

    view! {
        <main>
            <h1>"Count: " {count}</h1>
            <button onClick={ move |_| count.update(|c| *c += 1) }>"+"</button>
        </main>
    }
}
```

That click handler compiles to JavaScript automatically — lazy-loaded on first interaction, wired to resumed signal state. No hydration. No re-running components in the browser.

> If you like Qwik's resumability and Leptos' Rust-first approach, Resuma explores the space between both.

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/resuma.svg)](https://crates.io/crates/resuma)
[![docs.rs](https://img.shields.io/docsrs/resuma)](https://docs.rs/resuma)
[![License](https://img.shields.io/crates/l/resuma.svg)](https://github.com/GolfredoPerezFernandez/resuma)

**Docs:** [resuma-docs.fly.dev](https://resuma-docs.fly.dev) · **API:** [docs.rs/resuma](https://docs.rs/resuma) · **Repo:** [GitHub](https://github.com/GolfredoPerezFernandez/resuma)

</div>

---

## Why Resuma?

| Classic SSR + hydration | **Resuma** |
| --- | --- |
| Re-run components to attach listeners | **Resume** serialized state and handlers |
| JS grows with app size | ~3KB runtime + lazy handler chunks |
| Manual interactive boundaries | Every `#[component]` is resumable by default |
| Custom server RPC wiring | `#[server] async fn` + built-in endpoint |
| Ship framework + app logic upfront | rs2js compiles handlers to small JS on demand |

**Mental model:** components run on the server. The browser never re-executes them. SSR embeds signals and handler refs in HTML; the tiny client runtime resumes lazily — on first click or when an island scrolls into view.

---

## Quick start

> **Requires:** Rust 1.91+ ([rustup](https://rustup.rs))

```bash
cargo install resuma
resuma new                          # interactive — name + template menu
resuma new my-app --template todo   # full showcase (signals, server, islands)
cd my-app
resuma dev --open
```

**Templates:** `basic` (static SSR) · `todo` (full showcase) · `flow` (file-based pages) · `flow-fullstack` (Flow + SQLx)

Wire it up:

```rust
#[tokio::main]
async fn main() -> std::io::Result<()> {
    ResumaApp::new()
        .with_title("Counter")
        .page("/", || Counter::render(CounterProps::default()))
        .serve(ServeOptions::default())
        .await
}
```

Library only (no CLI):

```toml
[dependencies]
resuma = "0.3"
tokio  = { version = "1", features = ["full"] }
```

---

## CLI

| Command | What it does |
| --- | --- |
| `resuma new` | Scaffold a project — interactive prompts when run in a terminal |
| `resuma add sqlx` / `turso` | Drop in DB scaffolding (migrations, helpers) |
| `resuma dev` | Hot reload via `cargo-watch` + dev WebSocket |
| `resuma build` | Release binary + JS bundles (`--static-export` for static HTML) |
| `resuma routes --generate` | Discover `src/pages/` and emit `_registry.rs` |
| `resuma update` | Bump `resuma` / `resuma-macros` in your project |
| `resuma update --cli` | Reinstall the global CLI (`cargo install resuma --force`) |
| `resuma update --check` | Show installed vs available versions |
| `resuma doctor` | Check toolchain, CLI, and project setup |

```bash
resuma add              # interactive menu
resuma update           # align deps with CLI version
resuma doctor           # sanity check before you debug
```

---

## Server actions

```rust
#[server]
async fn search(q: String) -> Vec<String> {
    db::search(&q).await
}

#[component]
fn LiveSearch() -> View {
    let query   = use_signal(String::new());
    let results = use_signal::<Vec<String>>(vec![]);

    view! {
        <input
            onInput={ js! {
                state.query.set(event.target.value);
                const r = await __resuma.action('search', [event.target.value]);
                state.results.set(r);
            }}
        />
        <ul>{format!("{} results", results.peek().len())}</ul>
    }
}
```

`#[server]` registers `POST /_resuma/action/search`. Call it from the client without custom wiring.

---

## Islands (when you need them)

Every `#[component]` is already resumable. Reach for `#[island]` only for heavy widgets or viewport-triggered loading:

```rust
#[island(load = "visible")]
fn LiveChart() -> View {
    let points = use_signal(vec![1, 4, 2, 8]);
    view! { /* JS loads when this scrolls into view */ }
}
```

---

## Resuma Flow

One crate — `resuma` ships core + Flow. File-based pages, loaders, form submits:

```bash
resuma new my-app --template flow
resuma new my-app --template flow-fullstack   # + SQLx SQLite sample
resuma add sqlx                               # add DB layer to existing project
```

| API | Purpose |
| --- | --- |
| `FlowApp` | App builder with page registry |
| `#[load]` | Server data before render |
| `#[submit]` | Form mutations |
| `src/pages/` | File-based routing |

See [`docs/FLOW.md`](docs/FLOW.md) and the live guide at [resuma-docs.fly.dev/docs](https://resuma-docs.fly.dev/docs).

---

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                   resuma crate (v0.3)                    │
│                                                          │
│   core ──► ssr ──► server (axum)                         │
│     │              GET  /_resuma/runtime.js              │
│     │              POST /_resuma/action/:name            │
│     └──► flow + router (pages, loads, submits)           │
│                                                          │
│   resuma-macros (proc-macros + rs2js → JS handlers)    │
└──────────────────────────────────────────────────────────┘
                         │ HTTP
                         ▼
┌──────────────────────────────────────────────────────────┐
│                   Browser (~3KB)                         │
│   parse resuma/state · delegate events · lazy handlers   │
└──────────────────────────────────────────────────────────┘
```

Deep dive: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) · Security: [`docs/SECURITY.md`](docs/SECURITY.md) · Backend patterns: [`docs/BACKEND.md`](docs/BACKEND.md)

---

## Project layout

```
Resuma/
├── crates/
│   ├── resuma/             # runtime + SSR + server + flow + CLI
│   └── resuma-macros/      # view!, #[component], rs2js
├── apps/docs-site/         # documentation site
├── runtime/                # TypeScript source for the ~3KB client
└── examples/               # counter, todo, flow-demo, flow-pages
```

**From source:**

```bash
git clone https://github.com/GolfredoPerezFernandez/resuma
cd resuma
cargo install --path crates/resuma --features cli --force

cargo run -p example-counter    # http://127.0.0.1:3000
cargo run -p example-todo       # full-stack + security showcase
cargo run -p example-website    # docs site locally
```

---

## What ships in v0.3

- `view!{}` — JSX-like templates, no extra sigils
- `#[component]` — resumable boundary by default
- `#[server]` — async RPC endpoints
- `#[island]` — optional lazy client bundles
- `js!{}` — escape hatch for raw client handlers
- rs2js — Rust handlers → lazy JS chunks
- SSR payload with resumability state in HTML
- ~3KB client runtime (delegation + signals + RPC)
- axum server with `/_resuma/*` routes
- File-based routing (`src/pages/[id].rs` → `/users/:id`)
- Flow: `#[load]`, `#[submit]`, layouts, static export scaffold
- CLI: `new`, `add`, `dev`, `build`, `routes`, `update`, `doctor`

---

## Why "Resuma"?

Spanish for both *resumes* (continues) and *summary* — the framework's superpower is **resuming** execution from a serialized summary of the server-side render.

## License

MIT OR Apache-2.0
