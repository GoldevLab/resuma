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
resuma new my-app --template flow
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
    ├── counter/            # zero-server-state counter
    └── todo/               # server-action backed todo list
```

## Getting started

> **Pre-requisites:** Rust 1.74+ (`https://rustup.rs`) and Node 18+.

```sh
# Build everything
cargo build

# Run the counter example
cargo run -p example-counter
# → http://127.0.0.1:3000

# Try the todo example with server actions
cargo run -p example-todo

# Use the CLI (from source)
cargo install --path crates/resuma --features cli
# When published: cargo install resuma
resuma new my-app
cd my-app
resuma dev
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

- [ ] Streaming SSR (push body chunks as `routeLoader$` resolves)
- [ ] Hot Module Reload via `resuma-cli` + websocket bridge
- [ ] Layouts and nested routes
- [ ] Build-time pre-rendering for static sites
- [ ] Partial pre-rendering (PPR) — server-rendered shell + dynamic islands
- [ ] `#[island(load = "visible")]` lazy hydration policies
- [ ] Devtools extension to inspect the resumability payload
- [ ] First-class TypeScript bindings for `js!{}` blocks (autocomplete on `state.*`)
- [ ] WASM-backed islands for compute-heavy code (opt-in)

## Why "Resuma"?

Spanish for both *resumes* (continues) and *summary* — fitting because the framework's superpower is **resuming** execution from a serialised summary of the server-side render.

## License

MIT OR Apache-2.0
