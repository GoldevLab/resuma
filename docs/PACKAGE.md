# Resuma package — one install, two layers

Users depend on a **single crate** — core and full-stack Flow ship together.

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
resuma = { version = "0.3", default-features = false }
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

resuma new my-app                    # static SSR (default)
resuma new my-app --template todo    # full Resuma showcase

cd my-app && resuma dev
```

From the monorepo: `cargo install --path crates/resuma --features cli`

## Internal modules (for contributors)

| Module | Layer | Role |
|--------|-------|------|
| `resuma::core` | Core | Signals, View, resumability |
| `resuma::ssr` | Core | HTML + streaming |
| `resuma::server` | Core | axum, `/_resuma/*` |
| `resuma::flow` | Flow | `FlowApp`, routing, loads |
| `resuma::router` | Flow | File scanner for `src/pages/` |
| `resuma-macros` | Macros | `view!`, `#[component]`, rs2js |
| **`resuma`** | **Public** | **Runtime — depend on this** |

## Published crates

Only **`resuma`** + **`resuma-macros`** ship on [crates.io](https://crates.io/crates/resuma) (proc-macros must be a separate crate in Rust).

| Crate | crates.io | docs.rs |
|-------|-----------|---------|
| `resuma` | [crates.io/crates/resuma](https://crates.io/crates/resuma) | [docs.rs/resuma](https://docs.rs/resuma) |
| `resuma-macros` | [crates.io/crates/resuma-macros](https://crates.io/crates/resuma-macros) | [docs.rs/resuma-macros](https://docs.rs/resuma-macros) |

## Full-stack API map

| Concept | Resuma |
|---------|--------|
| Component | `#[component]` + `view!` |
| Server data loader | `#[load]` |
| Form mutation | `#[submit]` |
| Server RPC | `#[server]` |
| Request middleware | `#[middleware]` |
| File-based pages | `src/pages/` |
| Core + routing + CLI | `resuma` (one runtime crate) |

See the live docs: https://resuma-docs.fly.dev · or `cargo run -p example-website` → http://127.0.0.1:3000
