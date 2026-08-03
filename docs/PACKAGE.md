# Resuma package — one install, layered products

Users depend on a **single crate**. Core and full-stack ship together — same model as **Qwik + Qwik City** or **Solid + SolidStart**.

```
┌─────────────────────────────┐     ┌─────────────────────────────┐
│  Resuma (core)              │  +  │  Resuma Flow (full-stack)   │
│  Signals, view!, resumability│     │  Pages, loads, submits      │
│  ResumaApp, #[server]       │     │  FlowApp, src/pages/        │
└─────────────────────────────┘     └─────────────────────────────┘
                    ╲               ╱
                     ` resuma crate '
                              +
                     resuma-macros (Resuma Macros)
```

See [NAMING.md](./NAMING.md) for the full product map.

## Install

```toml
[dependencies]
resuma = { version = "1.3", default-features = false }
tokio  = { version = "1", features = ["full"] }
```

```rust
use resuma::prelude::*;
// Resuma + Resuma Flow + ClientComponent API
```

## When to use what

| You need | Product | API |
|----------|---------|-----|
| Single page, widget, island demo | **Resuma** | `ResumaApp` |
| Multi-page app, forms, server data | **Resuma Flow** | `FlowApp` + `src/pages/` |
| TypeScript widget (Three.js, chart) | **Resuma Client** | `ClientComponent` + `client_asset()` |

## CLI

```bash
cargo install resuma   # Resuma CLI

resuma new my-app                    # static SSR (default)
resuma new my-app --template todo    # full Resuma showcase

cd my-app && resuma dev
```

From the monorepo: `cargo install --path crates/resuma --features cli`

## Product layers

| # | Brand | Crate / path | Role |
|---|-------|--------------|------|
| 1 | **Resuma** | `resuma` (`core`, `ssr`, `server`) | Components, signals, resumability |
| 2 | **Resuma Flow** | `resuma::flow` + `resuma-flow` | `FlowApp` / routing in `resuma`; UI widgets in `resuma-flow` |
| 3 | **Resuma Macros** | `resuma-macros` | `view!`, `#[component]`, rs2js |
| 4 | **Resuma Runtime** | `runtime/` → `/_resuma/*.js` | Browser loader + core + flow |
| 5 | **Resuma Client** | `client-sdk/resuma-client.ts` | TypeScript widget mount contract |
| 6 | **Resuma CLI** | `resuma` feature `cli` | `new`, `dev`, `build` |

## Internal modules (contributors)

| Module | Product | Role |
|--------|---------|------|
| `resuma::core` | Resuma | Signals, View, resumability |
| `resuma::ssr` | Resuma | HTML + streaming |
| `resuma::server` | Resuma | axum, `/_resuma/*` |
| `resuma::flow` | Resuma Flow | `FlowApp`, routing, loads |
| `resuma::client` | Resuma Client | `ClientComponent`, mount HTML |
| `resuma::router` | Resuma Flow | File scanner for `src/pages/` |
| `resuma-macros` | Resuma Macros | `view!`, `#[component]`, rs2js |

## Published crates

Workspace **1.3.x** ships **`resuma-macros`**, **`resuma`**, and **`resuma-flow`** on [crates.io](https://crates.io/crates/resuma). Most apps depend on **`resuma`** only; add **`resuma-flow`** for Flow UI widgets.

| Crate | crates.io | docs.rs |
|-------|-----------|---------|
| `resuma` | [crates.io/crates/resuma](https://crates.io/crates/resuma) | [docs.rs/resuma](https://docs.rs/resuma) |
| `resuma-macros` | [crates.io/crates/resuma-macros](https://crates.io/crates/resuma-macros) | [docs.rs/resuma-macros](https://docs.rs/resuma-macros) |
| `resuma-flow` | [crates.io/crates/resuma-flow](https://crates.io/crates/resuma-flow) | [docs.rs/resuma-flow](https://docs.rs/resuma-flow) |

## API map

| Concept | Product | API |
|---------|---------|-----|
| Component | Resuma | `#[component]` + `view!` |
| Server data loader | Resuma Flow | `#[load]` |
| Form mutation | Resuma Flow | `#[submit]` |
| Server RPC | Resuma | `#[server]` |
| Request middleware | Resuma Flow | `#[middleware]` |
| File-based pages | Resuma Flow | `src/pages/` |
| TS client widget | Resuma Client | `ClientComponent`, `client_asset()` |

Live docs: https://resuma-docs.fly.dev
