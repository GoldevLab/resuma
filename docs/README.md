# Resuma documentation

Two ways to read the docs:

| Format | How |
|--------|-----|
| **Docs site** (recommended) | https://resuma-docs.fly.dev/docs · or `cargo run -p example-website` → http://127.0.0.1:3000/docs |
| **API reference** | [docs.rs/resuma](https://docs.rs/resuma) · [docs.rs/resuma-macros](https://docs.rs/resuma-macros) |
| **Crates.io** | [resuma](https://crates.io/crates/resuma) · [resuma-macros](https://crates.io/crates/resuma-macros) |
| **Markdown** (this folder) | GitHub / offline reference |

## Markdown index

| Doc | Topic |
|-----|--------|
| [GETTING_STARTED.md](./GETTING_STARTED.md) | Install CLI, templates, first app |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | Resumability, SSR payload, runtime |
| [PACKAGE.md](./PACKAGE.md) | Crate map (`resuma` umbrella) |
| [FLOW.md](./FLOW.md) | FlowApp, `#[load]`, `#[submit]`, pages |
| [SECURITY.md](./SECURITY.md) | CSRF, headers, rate limits, production |
| [BACKEND.md](./BACKEND.md) | NestJS + Next.js patterns → Rust (`examples/todo`) |
| [PUBLISHING.md](./PUBLISHING.md) | Publish to crates.io (production release) |

## Examples

Full table on the docs site: **`/docs/examples`**

```bash
cargo run -p example-todo        # Full showcase + backend security
cargo run -p example-flow-demo   # Loaders + streaming
cargo run -p example-flow-pages  # File-based routing
cargo run -p example-website     # This docs site
cargo run -p example-counter     # Minimal counter
```

## Docs site map

- **Introduction** — `/docs`, getting started, **`/docs/examples`**, project structure, FAQ
- **Security** — `/docs/security` (start with `/docs/security/todo`)
- **Components** — signals, islands, `#[server]`, `js!`
- **Resuma Flow** — routing, loaders, middleware, streaming
- **Cookbook** — debouncer, theme, Docker deploy
- **Reference** — architecture, CLI, benchmark, **API** (`/docs/api`)
