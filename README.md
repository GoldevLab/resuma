<div align="center">

<img src="assets/logo.svg" alt="Resuma Rust web framework logo" width="80" height="80" />

# Resuma — resumable SSR web framework for Rust

**Build instantly interactive Rust web apps without hydration.**

Ship HTML from the server. The browser **resumes** serialized state and lazy handler chunks — it never re-runs your components. Static pages ship **zero client JS**.

**Official repo:** [github.com/GoldevLab/resuma](https://github.com/GoldevLab/resuma) · **Docs:** [resuma-docs.fly.dev](https://resuma-docs.fly.dev/docs) · **Crate:** [crates.io/crates/resuma](https://crates.io/crates/resuma)

[![Crates.io](https://img.shields.io/crates/v/resuma.svg)](https://crates.io/crates/resuma)
[![docs.rs](https://img.shields.io/docsrs/resuma)](https://docs.rs/resuma)
[![CI](https://github.com/GoldevLab/resuma/actions/workflows/ci.yml/badge.svg)](https://github.com/GoldevLab/resuma/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/resuma.svg)](https://github.com/GoldevLab/resuma)
[![Rust](https://img.shields.io/badge/rust-1.91%2B-orange.svg)](https://rustup.rs)
[![Initial JS](https://img.shields.io/badge/initial%20JS-1006%20B-712cf9.svg)](benchmark/README.md)

**[Get started](https://resuma-docs.fly.dev/docs/getting_started)** · **[Benchmark](https://resuma-docs.fly.dev/docs/benchmark)** · **[Docs](https://resuma-docs.fly.dev/docs)** · **[API](https://docs.rs/resuma)**

```bash
cargo install resuma
resuma new my-app --template todo
cd my-app && resuma dev --open
```

</div>

---

## Small client runtime, measured in production builds

Same UX everywhere — SSR counter (heading + increment button). Production builds, gzip transfer sizes. [Full methodology →](benchmark/README.md)

| Framework | Initial load | First click | Static page |
|-----------|-------------:|------------:|------------:|
| **Resuma** | **1006 B** | **10.30 KiB** | **0 B** |
| Leptos | 79.02 KiB | 79.02 KiB | — |
| Next.js (App Router) | 142.43 KiB | 142.43 KiB | — |
| React (Vite SPA) | 57.99 KiB | 57.99 KiB | — |
| Astro (React island) | 57.76 KiB | 57.76 KiB | — |
| SvelteKit | 27.71 KiB | 27.71 KiB | — |
| Qwik | 1.96 KiB | 22.32 KiB | — |
| SolidStart | 16.75 KiB | 16.75 KiB | — |
| templ + HTMX | 16.21 KiB | 16.21 KiB | — |

Regenerate locally: `node benchmark/run.mjs` (or `--resuma-only` to refresh just Resuma). See [the benchmark methodology and sources](benchmark/README.md).

> If you like [Qwik's resumability](https://qwik.dev/docs/concepts/resumable/) and a Rust-first stack, Resuma is that model with native SSR, server actions, and **no WASM bundle by default**.

---

## Why Resuma?

| Classic SSR + hydration | **Resuma** |
| --- | --- |
| Re-run components to attach listeners | **Resume** serialized state and handlers |
| JS grows with the whole app upfront | **~1 KiB** loader + lazy handler chunks |
| Manual interactive boundaries | Every `#[component]` is resumable by default |
| Custom server RPC wiring | `#[server]` + built-in Axum routes |
| Static docs/marketing pages still ship JS | **0 B** on non-interactive pages |

**Mental model:** components run once on the server. SSR embeds signals and handler refs in HTML; a tiny client runtime resumes on first click or when an island enters the viewport. Multiplayer apps can use [`resuma::realtime`](https://docs.rs/resuma/latest/resuma/realtime/) for WebSocket rooms and peers.

---

## Quick start

> **Requires:** Rust 1.91+ ([rustup](https://rustup.rs))

Install the CLI, scaffold an app, and start the development server:

```bash
cargo install resuma
resuma new my-app --template todo
cd my-app
resuma dev --open
```

`resuma new` also opens an interactive project and template picker when run without arguments. Available templates include `basic`, `todo`, `flow`, `flow-booking`, `flow-fullstack`, and `production`.

### AI assistants (Cursor, Codex, Gemini)

Teach your editor Resuma patterns — `view!`, Flow, `HtmlTheme` / `Popup` / `Modal`, `SeoKit`, deploy — with one command:

```bash
resuma install skill              # ~/.cursor/skills/resuma/ (global)
resuma install skill --project    # .cursor/skills/resuma/ (this repo)
resuma install skill --target agents  # ~/.agents/skills/ (Codex-style)
resuma install skill --force      # overwrite after a CLI upgrade
```

**Skill vs MCP:** start with the **skill** (guidance for writing code). An official MCP for docs search / CLI tools may come later.  
Guide: [resuma-docs.fly.dev/docs/integrations/ai_assistant](https://resuma-docs.fly.dev/docs/integrations/ai_assistant)

```rust
use resuma::prelude::*;

#[component]
fn Counter() {
    let count = signal(0);
    view! {
        <main>
            <h1>"Count: " {count}</h1>
            <button onClick={count.update(|c| *c += 1)}>"+"</button>
        </main>
    }
}
```

Handlers compile to JavaScript automatically — lazy-loaded on first interaction, wired to resumed signal state.

**Library only:**

```toml
[dependencies]
resuma = "1.3"
tokio  = { version = "1", features = ["full"] }
```

---

## Server actions

```rust
#[data]
struct SearchResult {
    title: String,
    url: String,
}

#[server]
async fn search(q: String) -> Vec<SearchResult> {
    db::search(&q).await
}
```

`#[server]` registers `POST /_resuma/action/search`. Call from the client with `js!` or Flow forms — no custom wiring.

---

## Resuma Flow

Full-stack layer inside the same crate — file-based pages, loaders, submits (like **Qwik City** or **SolidStart**):

```bash
resuma new my-app --template flow
resuma add sqlx    # optional DB scaffolding
```

| API | Purpose |
| --- | --- |
| `FlowApp` | Resuma Flow app builder |
| `#[load]` | Server data before render |
| `#[submit]` | Form mutations |
| `src/pages/` | File-based routing |

Guide: [resuma-docs.fly.dev/docs/flow](https://resuma-docs.fly.dev/docs/flow)

### Product map

| Product | Crate / path | Role |
|---------|--------------|------|
| **Resuma** | `resuma` | Core — signals, resumability, `ResumaApp`, `resuma::realtime` |
| **Resuma Flow** | `resuma::flow` + `resuma-flow` | Pages/routing in `resuma`; UI widgets crate (`flow_graph`, dashboard, `worker_panel`) |
| **Resuma Macros** | `resuma-macros` | `view!`, `#[component]`, rs2js |
| **Resuma Runtime** | `runtime/` | Browser loader + core + flow |
| **Resuma Client** | `client-sdk/` | TypeScript widgets (`ClientComponent`) |
| **Resuma CLI** | `resuma` + `cli` feature | `new`, `dev`, `build` |

Details: [docs/NAMING.md](./docs/NAMING.md)

---

## CLI

| Command | What it does |
| --- | --- |
| `resuma new` | Scaffold a project |
| `resuma dev` | Hot reload |
| `resuma build` | Release binary + JS bundles |
| `resuma routes --generate` | Discover `src/pages/` → `_registry.rs` |
| `resuma add sqlx` / `turso` | DB scaffolding |
| `resuma update` / `doctor` | Align deps · sanity check |

---

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                   resuma crate                           │
│   core → ssr → server (axum) → flow + router             │
│   resuma-macros (view!, #[component], rs2js → JS)       │
└───────────────────────────┬──────────────────────────────┘
                            │ HTTP
                            ▼
┌──────────────────────────────────────────────────────────┐
│              Browser (~1 KiB loader, lazy core)          │
│   parse resuma/state · delegate events · lazy handlers   │
└──────────────────────────────────────────────────────────┘
```

Deep dive: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) · Security: [`docs/SECURITY.md`](docs/SECURITY.md) · Backend patterns: [`docs/BACKEND.md`](docs/BACKEND.md)

---

## Resources

| | |
|---|---|
| **Documentation** | [resuma-docs.fly.dev/docs](https://resuma-docs.fly.dev/docs) |
| **API reference** | [docs.rs/resuma](https://docs.rs/resuma) |
| **Examples** | [`examples/`](examples/) — counter, todo, flow-demo, flow-pages, **resuma-audit** (full docs QA) |
| **Benchmark** | [`benchmark/`](benchmark/) — multi-framework comparison + `run.mjs` |
| **Markdown docs** | [`docs/`](docs/) — offline / GitHub reference |
| **Security** | [`SECURITY.md`](SECURITY.md) · [security guide](https://resuma-docs.fly.dev/docs/security) |
| **Contributing** | [`CONTRIBUTING.md`](CONTRIBUTING.md) |

---

## Project layout

```
resuma/
├── assets/               # logo + GitHub social preview
├── crates/
│   ├── resuma/           # runtime, SSR, server, flow, CLI
│   ├── resuma-macros/    # view!, #[component], rs2js
│   └── resuma-flow/      # Flow UI widgets (flow_graph, dashboard, worker_panel)
├── runtime/              # TypeScript client (~1 KiB loader; see benchmark/)
├── benchmark/            # measured comparisons vs Qwik, Next, etc.
├── examples/             # runnable demos
└── docs/                 # markdown reference
```

Docs site (separate repo): [resuma-docs.fly.dev](https://resuma-docs.fly.dev)

**From source:**

```bash
git clone https://github.com/GoldevLab/resuma
cd resuma
cargo install --path crates/resuma --features cli --force
cargo run -p example-counter
cargo run -p example-todo
```

Manual apps can mount no-props components without the verbose render path:

```rust
ResumaApp::new()
    .component("/", App)
    .serve(ServeOptions::default())
    .await
```

---

## What's included

- `view!` / `#[component]` — JSX-like templates; resumable by default
- `#[server]` / `#[submit]` / `#[load]` — server RPC, form mutations, and Flow data loaders
- `#[island]` — optional lazy client bundles
- SPA `NavLink` + view transitions
- `ResumaError::Redirect` + middleware auth gating
- Cookies / session helpers (`Redirect::with_session_cookie`)
- Workers, artifacts, and uploads
- `resuma::realtime` — WebSocket rooms / peers
- WebGPU CSP defaults
- Client asset cache-busting
- Versioned runtime assets, immutable caching for current URLs, and selective core preloading
- Minified runtime overlay CSS
- Per-action rate limits
- Flow file-based routing (`FlowApp`, `src/pages/`)
- CLI: `new`, `add`, `dev`, `build`, `routes`, `update`, `doctor`

### Upgrading to 2.0

Version 2.0 changes the resumability payload protocol. Upgrade the server crate
and its embedded browser runtime together; do not mix 1.x server assets with
2.x runtime assets. Rebuild applications and deploy the complete release.

---

## Community

- **Issues & ideas:** [GitHub Issues](https://github.com/GoldevLab/resuma/issues)
- **Security:** see [SECURITY.md](SECURITY.md) — please do not file public issues for exploitable bugs

Contributions welcome — read [CONTRIBUTING.md](CONTRIBUTING.md) first.

---

## The name

Resuma takes its name from the idea of a *summary*: the server renders a page once, then the browser resumes it from the serialized state and handlers.

**Topics:** Rust · SSR · resumability · Axum · server actions · islands · zero hydration · `resuma` on [crates.io](https://crates.io/crates/resuma).

## License

Dual-licensed under **MIT OR Apache-2.0**.
