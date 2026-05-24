# Resuma Flow

**Resuma Flow** is the full-stack layer: pages, layouts, server data, form submissions, and middleware — included in the `resuma` crate.

**Interactive docs:** [/docs/flow](https://resuma-docs.fly.dev/docs/flow)

## Data path

```
#[load]  →  SSR render  →  resumability payload  →  user interaction  →  #[submit]
```

## API map

| Concept | Resuma Flow API |
|---------|-----------------|
| App builder | `FlowApp` |
| Request context | `FlowRequest` |
| Server data (pre-render) | `#[load]` |
| Form mutation | `#[submit]` |
| Ad-hoc RPC | `#[server]` (+ optional `&FlowRequest`) |
| Middleware | `#[middleware]` |
| Shared chrome | `#[layout]` |
| Page files | `src/pages/` + `resuma routes --generate` |

## Examples

| Crate | Command | Focus |
|-------|---------|-------|
| `flow-pages` | `cargo run -p example-flow-pages` | File routing, layouts |
| `flow-demo` | `cargo run -p example-flow-demo` | Loaders, streaming |

Docs site source lives in the separate **`site-docs`** repository (`../site-docs`). See [docs/README.md](./README.md).

## Bootstrap

```rust
mod pages;
use pages::PagesRegistry;

FlowApp::new()
    .with_title("My App")
    .streaming(true)
    .auto_pages("src/pages", PagesRegistry)
    .not_found(|| not_found_page())
    .serve(FlowServeOptions::default())
    .await
```

Generate routes:

```bash
resuma routes --generate --path src/pages
```

## Key features

### `#[load]` + cache

```rust
#[load(cache = "public, max-age=60")]
async fn home(req: &FlowRequest) -> HomeData { ... }
```

Cache-Control is merged on the HTML response. See `/docs/flow/caching`.

### `#[load(stream)]` + streaming SSR

Enable with `FlowApp::streaming(true)`. Shell streams first; loader HTML arrives in follow-up chunks. See `/docs/flow/streaming` and `examples/flow-demo`.

### `#[submit]` + forms

Progressive enhancement: `POST /_resuma/submit/:name` works without JS. See `/docs/flow/submits`.

### Middleware

`#[middleware]` runs before pages, loaders, submits, and actions. See `/docs/flow/middleware` and `/docs/security/middleware` (auth patterns).

### SEO

Flow serves `/robots.txt` and `/sitemap.xml`. Configure via `FlowSeoConfig`.

## HTTP endpoints (Flow)

| Method | Path | Handler |
|--------|------|---------|
| POST | `/_resuma/submit/:name` | `#[submit]` |
| POST | `/_resuma/action/:name` | `#[server]` |
| GET | `/robots.txt` | Auto |
| GET | `/sitemap.xml` | Auto |

Full API: `/docs/api` on the docs site.

## Modules (inside `resuma`)

| Module | Role |
|--------|------|
| `resuma::core` | Signals, views, resumability |
| `resuma::flow` | Pages, loads, submits, routing |
| `resuma::server` | axum HTTP + `/_resuma/*` |
| `resuma::router` | File scanner for `src/pages/` |

## Cookbook recipes

| Recipe | Docs |
|--------|------|
| Portal | `/docs/cookbook/portals` |
| View transitions | `/docs/cookbook/view_transitions` |
| Theme | `/docs/cookbook/theme` |
| Streaming loaders | `/docs/cookbook/streaming_loaders` |
| Docker / Fly deploy | `/docs/cookbook/docker` |
