<div align="center">

<img src="assets/logo.svg" alt="Resuma Rust web framework logo" width="80" height="80" />

# Resuma — resumable SSR web framework for Rust

**Instantly-interactive Rust web apps — without hydration.** Not a résumé/CV builder.

Ship HTML from the server. The browser **resumes** serialized state and lazy handler chunks — it never re-runs your components. Static pages ship **zero client JS**.

**Official repo:** [github.com/GolfredoPerezFernandez/resuma](https://github.com/GolfredoPerezFernandez/resuma) · **Docs:** [resuma-docs.fly.dev](https://resuma-docs.fly.dev/docs) · **Crate:** [crates.io/crates/resuma](https://crates.io/crates/resuma)

[![Crates.io](https://img.shields.io/crates/v/resuma.svg)](https://crates.io/crates/resuma)
[![docs.rs](https://img.shields.io/docsrs/resuma)](https://docs.rs/resuma)
[![CI](https://github.com/GolfredoPerezFernandez/resuma/actions/workflows/ci.yml/badge.svg)](https://github.com/GolfredoPerezFernandez/resuma/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/resuma.svg)](https://github.com/GolfredoPerezFernandez/resuma)
[![Rust](https://img.shields.io/badge/rust-1.91%2B-orange.svg)](https://rustup.rs)
[![Initial JS](https://img.shields.io/badge/initial%20JS-901%20B-712cf9.svg)](benchmark/README.md)

**[Get started](https://resuma-docs.fly.dev/docs/getting_started)** · **[Benchmark](https://resuma-docs.fly.dev/docs/benchmark)** · **[Docs](https://resuma-docs.fly.dev/docs)** · **[API](https://docs.rs/resuma)**

```bash
cargo install resuma
resuma new my-app --template todo
cd my-app && resuma dev --open
```

</div>

---

## Measured: smallest interactive runtime in class

Same UX everywhere — SSR counter (heading + increment button). Production builds, gzip transfer sizes. [Full methodology →](benchmark/README.md)

| Framework | Initial load | First click | Static page |
|-----------|-------------:|------------:|------------:|
| **Resuma** | **907 B** | **5.08 KiB** | **0 B** |
| Leptos | 79.02 KiB | 79.02 KiB | — |
| Next.js (App Router) | 142.43 KiB | 142.43 KiB | — |
| React (Vite SPA) | 57.99 KiB | 57.99 KiB | — |
| Astro (React island) | 57.76 KiB | 57.76 KiB | — |
| SvelteKit | 27.71 KiB | 27.71 KiB | — |
| Qwik | 1.96 KiB | 22.32 KiB | — |
| SolidStart | 16.75 KiB | 16.75 KiB | — |
| templ + HTMX | 16.21 KiB | 16.21 KiB | — |

Regenerate locally: `node benchmark/run.mjs` · Independent sources in [benchmark/README.md](benchmark/README.md#external-validation)

> If you like [Qwik's resumability](https://qwik.dev/docs/concepts/resumable/) and a Rust-first stack, Resuma is that model with native SSR, server actions, and **no WASM bundle by default**.

---

## Why Resuma?

| Classic SSR + hydration | **Resuma** |
| --- | --- |
| Re-run components to attach listeners | **Resume** serialized state and handlers |
| JS grows with the whole app upfront | **907 B** loader + lazy handler chunks |
| Manual interactive boundaries | Every `#[component]` is resumable by default |
| Custom server RPC wiring | `#[server]` + built-in Axum routes |
| Static docs/marketing pages still ship JS | **0 B** on non-interactive pages |

**Mental model:** components run once on the server. SSR embeds signals and handler refs in HTML; a tiny client runtime resumes on first click or when an island enters the viewport.

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

**Templates:** `basic` · `todo` · `flow` · `flow-booking` · `flow-fullstack` · `production`

### AI assistants (Cursor, Codex, Gemini)

Teach your editor Resuma patterns — signals, `view!`, Flow, `#[server]` — with one command:

```bash
resuma install skill              # ~/.cursor/skills/resuma/ (global)
resuma install skill --project    # .cursor/skills/resuma/ (this repo)
resuma install skill --target agents  # ~/.agents/skills/ (Codex-style)
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
resuma = "1.0"
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
| **Resuma** | `resuma` | Core — signals, resumability, `ResumaApp` |
| **Resuma Flow** | `resuma::flow` | Pages, routing, loaders, `FlowApp` |
| **Resuma Macros** | `resuma-macros` | `view!`, `#[component]`, rs2js |
| **Resuma Runtime** | `runtime/` | Browser loader + core |
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
│              Browser (907 B loader, lazy core)           │
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
│   └── resuma-macros/    # view!, #[component], rs2js
├── runtime/              # TypeScript client (~907 B loader)
├── benchmark/            # measured comparisons vs Qwik, Next, etc.
├── examples/             # runnable demos
└── docs/                 # markdown reference
```

Docs site (separate repo): [resuma-docs.fly.dev](https://resuma-docs.fly.dev)

**From source:**

```bash
git clone https://github.com/GolfredoPerezFernandez/resuma
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

## What ships in v0.4

- `view!{}` — JSX-like templates
- `#[component]` — resumable by default
- `#[server]` / `#[submit]` / `#[load]` — server RPC and Flow data
- `#[island]` — optional lazy client bundles
- rs2js — Rust handlers → lazy JS chunks
- axum server with `/_resuma/*` routes
- File-based routing · static export scaffold
- CLI: `new`, `add`, `dev`, `build`, `routes`, `update`, `doctor`
- Production security defaults (CSRF, CSP, rate limits)

---

## Community

- **Issues & ideas:** [GitHub Issues](https://github.com/GolfredoPerezFernandez/resuma/issues)
- **Security:** see [SECURITY.md](SECURITY.md) — please do not file public issues for exploitable bugs

Contributions welcome — read [CONTRIBUTING.md](CONTRIBUTING.md) first.

---

## Why "Resuma"?

Spanish for both *resumes* (continues) and *summary* — the framework **resumes** execution from a serialized summary of the server-side render.

> **Looking for a resume/CV app?** This repository is a **Rust full-stack web framework** (SSR, Axum, server actions). Resume-builder projects on GitHub use the same name but are unrelated.

## Find this project (search & links)

| If you searched for… | You want |
|----------------------|--------|
| `resuma github` + Rust / SSR / framework | **This repo** — [GolfredoPerezFernandez/resuma](https://github.com/GolfredoPerezFernandez/resuma) |
| `resuma rust framework` | [Getting started](https://resuma-docs.fly.dev/docs/getting_started) · `cargo install resuma` |
| `rust web framework no hydration` | [Architecture](https://resuma-docs.fly.dev/docs/architecture) · [Benchmark](https://resuma-docs.fly.dev/docs/benchmark) |
| `qwik rust` / resumability | [Flow guide](https://resuma-docs.fly.dev/docs/flow) · Qwik-style lazy handlers in Rust |

**Topics:** Rust · SSR · resumability · Axum · server actions · islands · zero hydration · `resuma` on [crates.io](https://crates.io/crates/resuma).

## License

Dual-licensed under **MIT OR Apache-2.0**.
