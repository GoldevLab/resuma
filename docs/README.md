# Resuma documentation

Two ways to read the docs:

| Format | How |
|--------|-----|
| **Docs site** (recommended) | https://resuma-docs.fly.dev/docs · local: `../site-docs` and `cargo run` |
| **API reference** | [docs.rs/resuma](https://docs.rs/resuma) · [docs.rs/resuma-macros](https://docs.rs/resuma-macros) |
| **Crates.io** | [resuma](https://crates.io/crates/resuma) · [resuma-macros](https://crates.io/crates/resuma-macros) |
| **Markdown** (this folder) | GitHub / offline reference |

## Markdown index

| Doc | Topic |
|-----|--------|
| [GETTING_STARTED.md](./GETTING_STARTED.md) | Install CLI, templates, first app |
| [BLOG_RUST_SSR_WITHOUT_HYDRATION.md](./BLOG_RUST_SSR_WITHOUT_HYDRATION.md) | Technical post draft (Showcase / Dev.to / HN) |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | Resumability, SSR payload, runtime |
| [PACKAGE.md](./PACKAGE.md) | Product map — Resuma, Resuma Flow, Macros, Client |
| [NAMING.md](./NAMING.md) | Official brand names and naming rules |
| [FLOW.md](./FLOW.md) | Resuma Flow — FlowApp, `#[load]`, `#[submit]`, pages |
| [FLOW_COOKBOOK.md](./FLOW_COOKBOOK.md) | Query loaders, `public/`, PWA, booking template (v0.4.6+) |
| [SECURITY.md](./SECURITY.md) | CSRF, headers, rate limits, production |
| [BACKEND.md](./BACKEND.md) | NestJS + Next.js patterns → Rust (`examples/todo`) |
| [PUBLISHING.md](./PUBLISHING.md) | Publish to crates.io (production release) |

## Examples

Full table on the docs site: **`/docs/examples`**

```bash
cargo run -p example-todo        # Full showcase + backend security
cargo run -p example-flow-demo   # Loaders + streaming
cargo run -p example-flow-pages  # File-based routing
cargo run -p example-counter     # Minimal counter
```

Docs site source lives in the separate **`site-docs`** repository (sibling folder `../site-docs` next to this monorepo). GitHub: [resuma-docs](https://github.com/GoldevLab/resuma-docs).

## Docs site map

- **Introduction** — `/docs`, getting started, **`/docs/examples`**, project structure, FAQ
- **Security** — `/docs/security` (start with `/docs/security/todo`)
- **Components** — signals, islands, `#[server]`, `js!`
- **Resuma Flow** — routing, loaders, middleware, streaming
- **Cookbook** — debouncer, theme, Docker deploy, query-driven loaders
- **Flow** — `/docs/flow/pwa` (PWA + `public/`), `/docs/flow/query_params` (SPA reload)
- **Reference** — architecture, CLI, benchmark, **API** (`/docs/api`)
