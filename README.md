<div align="center">

# 🌊 Resuma

[![Crates.io](https://img.shields.io/crates/v/resuma.svg)](https://crates.io/crates/resuma)
[![docs.rs](https://img.shields.io/docsrs/resuma)](https://docs.rs/resuma)
[![License](https://img.shields.io/crates/l/resuma.svg)](https://github.com/GolfredoPerezFernandez/resuma)

**The first Rust web framework with SSR + Resumability + Islands + Server Actions + a friendly JS Bridge.**

*Zero hydration, true resumability, native islands, automatic Rust→JS handler compilation.*

**Install:** [`cargo install resuma`](https://crates.io/crates/resuma) · **Docs:** [resuma-docs.fly.dev](https://resuma-docs.fly.dev) · **API:** [docs.rs/resuma](https://docs.rs/resuma) · **Repo:** [GitHub](https://github.com/GolfredoPerezFernandez/resuma)

</div>

---

## What is this?

Resuma is a from-scratch Rust framework for building modern web apps with **resumability** instead of hydration:

## Resumability vs hydration

| Aspect | Classic SSR + hydration | **Resuma** |
| --- | --- | --- |
| Client work after load | Re-run components to attach listeners | **Resume** serialized state and handlers |
| Initial JS | Grows with app size | ~3KB runtime + lazy chunks |
| Interactive boundaries | Often manual | First-class `#[island]` |
| Server RPC | Custom wiring | `#[server] async fn` + built-in endpoint |
| Handler code on client | Ship framework runtime + app logic | Compile handlers to small JS via rs2js |
| Templates | Varies | JSX-like `view!{}` — no extra sigils |

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

That single click handler is **automatically translated to JavaScript** by `resuma-macros` (rs2js), lazy-loaded on first interaction, and runs against the resumed signal state. No hydration, no re-execution, no WASM bundle.

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

**One crate** — `resuma` includes core + Flow in a single dependency.

| Resuma Flow | Purpose |
|-------------|---------|
| `FlowApp` | App builder with page registry |
| `#[load]` | Server data before render |
| `#[submit]` | Form mutations |
| `src/pages/` | File-based pages |

See [`docs/PACKAGE.md`](docs/PACKAGE.md) and [`docs/FLOW.md`](docs/FLOW.md).

**Live docs site:** https://resuma-docs.fly.dev · or `cargo run -p example-website` → http://127.0.0.1:3000

```bash
resuma new my-app                    # static SSR (default)
resuma new my-app --template todo    # full Resuma showcase
```

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                   resuma crate (v0.2)                    │
│                                                          │
│   core ──► ssr ──► server (axum)                         │
│     │              GET  /_resuma/runtime.js              │
│     │              POST /_resuma/action/:name            │
│     └──► flow + router (pages, loads, submits)           │
│                                                          │
│   resuma-macros (separate crate)                         │
│     view! / #[component] / rs2js → JS handlers           │
└──────────────────────────────────────────────────────────┘
                         │ HTTP
                         ▼
┌──────────────────────────────────────────────────────────┐
│                   Browser (~3KB)                         │
│   parse resuma/state · delegate events · lazy handlers   │
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
│   ├── resuma/             # single runtime crate (core, ssr, server, flow, cli)
│   └── resuma-macros/      # proc-macros + rs2js (required separate crate)
├── runtime/                # TypeScript source for the ~3KB client runtime
└── examples/
    ├── counter/
    ├── todo/
    ├── flow-demo/
    ├── flow-pages/
    └── website/            # docs site
```

**Docs:** [`docs/README.md`](docs/README.md) · live site: `cargo run -p example-website`

## Getting started

> **Pre-requisites:** Rust 1.91+ ([rustup](https://rustup.rs)).

### Install from crates.io (recommended)

```sh
cargo install resuma
resuma new my-app --template todo
cd my-app
resuma dev
```

- **Crate:** [crates.io/crates/resuma](https://crates.io/crates/resuma)
- **API docs:** [docs.rs/resuma/0.3.0](https://docs.rs/resuma/0.3.0)
- **Proc-macros:** [docs.rs/resuma-macros/0.3.0](https://docs.rs/resuma-macros/0.3.0)
- **Guide:** [resuma-docs.fly.dev/docs](https://resuma-docs.fly.dev/docs)

Library only (no CLI binary):

```toml
[dependencies]
resuma = { version = "0.3", default-features = false }
tokio = { version = "1", features = ["full"] }
```

### From source (development)

```sh
git clone https://github.com/GolfredoPerezFernandez/resuma
cd resuma
cargo install --path crates/resuma --features cli

# Examples
cargo run -p example-counter   # http://127.0.0.1:3000
cargo run -p example-todo      # full-stack + security showcase
cargo run -p example-website   # docs site
```

## What works in v0.3

✅ `Signal<T>`, `use_signal`, `use_effect`, `use_computed` (effects/computed SSR-only for now)
✅ `view!{}` macro with JSX-like syntax (no `$` noise)
✅ `#[component]` with auto-generated props builder
✅ `#[server]` async actions with JSON-RPC endpoint
✅ `#[island]` interactive component boundary
✅ `js!{}` escape hatch for raw JS handlers
✅ Rust → JS compiler for common handler patterns
✅ SSR with resumability payload embedded in HTML
✅ Lazy handler/island chunks registered automatically from SSR
✅ ~3KB client runtime (lazy event delegation + signals + RPC)
✅ axum-based server with built-in `/_resuma/*` routes
✅ File-based routing scanner (`src/pages/[id].rs` → `/users/:id`)
✅ Flow static routes receive `FlowRequest` (query, headers, method)
✅ `RESUMA_ADDR` / `HOST`+`PORT` bind configuration
✅ `resuma` CLI: `new` (basic/todo/flow), `dev`, `build`, `routes`

## Client-side reactivity (v0.3)

`use_signal` updates work on the client via the resumability payload. `use_effect` and `use_computed` currently run during **SSR only** — use `js!` handlers or manual signal updates for client-driven derived state until full client effect replay lands in a future release.

## Roadmap (v0.4+)

- [ ] Hot Module Reload via `resuma` CLI + websocket bridge
- [ ] Build-time pre-rendering for static sites
- [ ] Partial pre-rendering (PPR) — server shell + dynamic islands
- [ ] `#[island(load = "visible")]` lazy load policies
- [ ] Devtools extension for resumability payload inspection
- [ ] First-class TypeScript bindings for `js!{}` blocks
- [ ] WASM-backed islands for compute-heavy code (opt-in)

Already shipped in v0.3: HTTP context in Flow routes, env-based bind address, flow scaffold template, crypto CSRF, lazy chunk registration, expanded CI. v0.2 brought single-crate layout, streaming SSR (Flow), layouts, file-based routing, security defaults, [crates.io publish](https://crates.io/crates/resuma).

## Why "Resuma"?

Spanish for both *resumes* (continues) and *summary* — fitting because the framework's superpower is **resuming** execution from a serialised summary of the server-side render.

## License

MIT OR Apache-2.0
