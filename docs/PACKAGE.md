# Resuma package — one install, two layers

Like **Qwik + Qwik City**, but unified: users depend on a single crate.

```
┌─────────────────────────────┐     ┌─────────────────────────────┐
│  Resuma¹ (core)             │  +  │  Flow² (full-stack)         │
│  Components, signals, SSR   │     │  Pages, loads, submits      │
│  #[server], #[island]       │     │  FlowApp, src/pages/        │
└─────────────────────────────┘     └─────────────────────────────┘
                    ╲               ╱
                     ` resuma crate '
```

## Install

```toml
[dependencies]
resuma = { version = "0.1", default-features = false }
tokio  = { version = "1", features = ["full"] }
```

```rust
use resuma::prelude::*;
// Everything: ResumaApp, FlowApp, view!, #[load], #[submit], …
```

## When to use what

| You need | Use |
|----------|-----|
| Single page, widget, island demo | `ResumaApp` |
| Multi-page app, forms, server data | `FlowApp` + `src/pages/` |

## CLI

```bash
cargo install resuma

resuma new my-app                    # counter (core only)
resuma new my-app --template flow    # Flow + file-based pages

cd my-app && resuma dev
```

From the monorepo: `cargo install --path crates/resuma --features cli`

## Internal crates (for contributors)

| Crate | Layer | Role |
|-------|-------|------|
| `resuma-core` | Core | Signals, View, resumability |
| `resuma-macros` | Core | `view!`, `#[component]`, `#[load]` |
| `resuma-ssr` | Core | HTML + streaming |
| `resuma-server` | Core | axum, `/_resuma/*` |
| `resuma-flow` | Flow | `FlowApp`, routing, loads |
| `resuma-router` | Flow | File scanner for `src/pages/` |
| **`resuma`** | **Public** | **Re-exports everything** |

## Qwik → Resuma map

| Qwik / Qwik City | Resuma |
|------------------|--------|
| `component$` | `#[component]` + `view!` |
| `routeLoader$` | `#[load]` |
| `routeAction$` | `#[submit]` |
| `server$` | `#[server]` |
| `plugin.ts` | `#[middleware]` |
| `src/routes/` | `src/pages/` |
| Qwik + Qwik City (2 packages) | `resuma` (1 package) |

See the live docs: `cargo run -p example-website` → http://127.0.0.1:3000
