<div align="center">

# 🌊 Resuma

**The first Rust web framework with SSR + Resumability + Islands + Server Actions + a friendly JS Bridge.**

*Better than Leptos: zero hydration, true resumability, native islands, automatic Rust→JS handler compilation.*

</div>

---

## What is this?

Resuma is a from-scratch Rust framework for building modern web apps. It picks the best ideas from the JavaScript world and brings them to Rust **without compromise**:

| Feature | Leptos / Yew / Dioxus | **Resuma** |
| --- | --- | --- |
| Render mode | Hydration | **Resumability** |
| Initial JS bundle | All components | ~3KB runtime |
| Islands architecture | ❌ | ✅ first-class `#[island]` |
| Server actions | Partial | `#[server] async fn` + RPC |
| JS interop | Manual `wasm-bindgen` | **`js!{}` escape hatch** |
| Templates | RSX / DSL | **JSX-like `view!{}`** without `$` noise |
| Rust → JS compilation | ❌ | ✅ via `resuma-rs2js` |

The mental model: **components only run on the server**. The browser never re-executes them. Instead, the SSR pass serialises every signal, handler reference and island into the HTML, and the tiny client runtime *resumes* execution lazily — exactly when the user clicks something.

## Hello, Resuma

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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    ResumaApp::new()
        .with_title("Counter")
        .page("/", || Counter::render(CounterProps::default()))
        .serve(ServeOptions::default())
        .await
}
```

That single click handler is **automatically translated to JavaScript** by `resuma-rs2js`, lazy-loaded on first interaction, and runs against the resumed signal state. No hydration, no re-execution, no WASM bundle.

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

`#[server]` registers an RPC endpoint at `/_resuma/action/search`. The handler is dispatched there transparently.

## Islands

```rust
#[island]
fn LiveCounter() -> View {
    let count = use_signal(0);
    view! {
        <button onClick={ move |_| count.update(|c| *c += 1) }>{count}</button>
    }
}
```

Mark any component with `#[island]` and Resuma will package its handlers into an isolated chunk that ships only when the island scrolls into view (or immediately, configurable).

## Resuma Flow (full-stack layer)

**One crate** — `resuma` includes core + Flow (like Qwik + Qwik City, unified).

| Resuma Flow | Purpose |
|-------------|---------|
| `FlowApp` | App builder with page registry |
| `#[load]` | Server data before render |
| `#[submit]` | Form mutations |
| `src/pages/` | File-based pages |

See [`docs/PACKAGE.md`](docs/PACKAGE.md) and [`docs/FLOW.md`](docs/FLOW.md).

**Live docs site:** `cargo run -p example-website` → http://127.0.0.1:3000

```bash
resuma new my-app                    # static SSR (default)
resuma new my-app --template todo    # full Resuma showcase
```

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                      Resuma App                          │
│                                                          │
│   ┌──────────────────┐    ┌────────────────────────┐     │
│   │ resuma-core      │    │ resuma-macros          │     │
│   │ Signal/View/Comp │◄──►│ view!/#[component]/... │     │
│   └────────┬─────────┘    └──────────┬─────────────┘     │
│            │                          │                  │
│            ▼                          ▼                  │
│   ┌──────────────────┐    ┌────────────────────────┐     │
│   │ resuma-ssr       │    │ resuma-rs2js           │     │
│   │ View → HTML      │    │ Rust closures → JS     │     │
│   └────────┬─────────┘    └────────────────────────┘     │
│            │                                              │
│            ▼                                              │
│   ┌──────────────────────────────────────────────────┐   │
│   │ resuma-server (axum)                             │   │
│   │  GET  /_resuma/runtime.js                        │   │
│   │  POST /_resuma/action/:name                      │   │
│   │  GET  /_resuma/handler/:chunk.js                 │   │
│   │  GET  /_resuma/island/:chunk.js                  │   │
│   └──────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────┘
                         │ HTTP
                         ▼
┌──────────────────────────────────────────────────────────┐
│                   Browser (~3KB)                         │
│   ┌──────────────────────────────────────────────────┐   │
│   │ Resuma runtime                                   │   │
│   │  • parse <script type="resuma/state">…           │   │
│   │  • reconstruct signals                           │   │
│   │  • document-level event delegation               │   │
│   │  • lazy import handler chunks                    │   │
│   │  • call server actions via fetch                 │   │
│   └──────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────┘
```

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for a deep dive.

**Security:** [`docs/SECURITY.md`](docs/SECURITY.md) — CSRF, headers, rate limits, production checklist.

**Backend patterns:** [`docs/BACKEND.md`](docs/BACKEND.md) — live in `examples/todo`.

**All docs:** [`docs/README.md`](docs/README.md) · `cargo run -p example-website`

**Publishing:** [`docs/PUBLISHING.md`](docs/PUBLISHING.md) — crates.io release checklist

## Project layout

```
Resuma/
├── crates/
│   ├── resuma-core/        # Signals, Effects, View tree, Component trait
│   ├── resuma-macros/      # view!, #[component], #[server], #[island], js!
│   ├── resuma-ssr/         # SSR renderer + resumability payload
│   ├── resuma-rs2js/       # Rust → JS subset compiler (handler bodies)
│   ├── resuma-server/      # axum HTTP server (pages, actions, runtime)
│   ├── resuma-router/      # file-based routing scanner
│   ├── resuma-cli/         # `resuma new|dev|build|routes`
│   └── resuma/             # umbrella facade users depend on
├── runtime/                # TypeScript source for the ~3KB client runtime
└── examples/
    ├── counter/            # minimal counter
    ├── todo/               # full showcase + backend security reference
    ├── flow-demo/          # FlowApp with loaders & streaming
    ├── flow-pages/         # file-based routing
    └── website/            # docs site (this documentation)
```

**Docs:** [`docs/README.md`](docs/README.md) · live site: `cargo run -p example-website`

## Getting started

> **Pre-requisites:** Rust 1.74+ ([rustup](https://rustup.rs)).

### Install from crates.io (recommended)

```sh
cargo install resuma
resuma new my-app --template todo
cd my-app
resuma dev
```

Library only (no CLI binary):

```toml
[dependencies]
resuma = { version = "0.1", default-features = false }
tokio = { version = "1", features = ["full"] }
```

### From source (development)

```sh
git clone https://github.com/resuma/resuma
cd resuma
cargo install --path crates/resuma --features cli

# Examples
cargo run -p example-counter   # http://127.0.0.1:3000
cargo run -p example-todo      # full-stack + security showcase
cargo run -p example-website   # docs site
```

## What works in v0.1

✅ `Signal<T>`, `use_signal`, `use_effect`, `use_computed`
✅ `view!{}` macro with JSX-like syntax (no `$` noise)
✅ `#[component]` with auto-generated props builder
✅ `#[server]` async actions with JSON-RPC endpoint
✅ `#[island]` interactive component boundary
✅ `js!{}` escape hatch for raw JS handlers
✅ Rust → JS compiler for common handler patterns
✅ SSR with resumability payload embedded in HTML
✅ ~3KB client runtime (lazy event delegation + signals + RPC)
✅ axum-based server with built-in `/_resuma/*` routes
✅ File-based routing scanner (`src/routes/[id].rs` → `/users/:id`)
✅ `resuma` CLI: `new`, `dev`, `build`, `routes`

## Roadmap (v0.2+)

- [ ] Hot Module Reload via `resuma-cli` + websocket bridge
- [ ] Build-time pre-rendering for static sites
- [ ] Partial pre-rendering (PPR) — server shell + dynamic islands
- [ ] `#[island(load = "visible")]` lazy load policies
- [ ] Devtools extension for resumability payload inspection
- [ ] First-class TypeScript bindings for `js!{}` blocks
- [ ] WASM-backed islands for compute-heavy code (opt-in)

Already shipped in v0.1: streaming SSR (Flow), layouts, file-based routing, security defaults.

## Why "Resuma"?

Spanish for both *resumes* (continues) and *summary* — fitting because the framework's superpower is **resuming** execution from a serialised summary of the server-side render.

## License

MIT OR Apache-2.0
