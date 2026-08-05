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

Live dashboards: `loader_poll("/", 8_000)` re-runs loaders via SPA `__resuma.invalidate`
(prefer over `<meta refresh>`). See [FLOW_COOKBOOK.md](./FLOW_COOKBOOK.md).

### `#[load(stream)]` + streaming SSR

Enable with `FlowApp::streaming(true)`. Shell streams first; loader HTML arrives in follow-up chunks. See `/docs/flow/streaming` and `examples/flow-demo`.

### `#[submit]` + forms

Progressive enhancement: `POST /_resuma/submit/:name` works without JS. See `/docs/flow/submits`.

### Middleware

`#[middleware]` runs before pages, loaders, submits, and actions. See `/docs/flow/middleware` and `/docs/security/middleware` (auth patterns).

To gate a page behind login and bounce anonymous visitors with a real HTTP 302
(instead of the error shell), return `Err(ResumaError::Redirect("/login".into()))`.
Attach session identity with `req.set_extension(...)` so handlers and layouts can
read `req.is_authenticated()` / `req.user_id()`.

`#[layout]` bodies only see the request via `current_request()` (the macro discards
its `req` argument). After middleware runs, Flow republishes the updated request
with `set_current_request`, so a layout nav can show the logged-in user:

```rust
#[middleware]
async fn attach_session(mut req: FlowRequest) -> resuma::Result<FlowRequest> {
    if let Some(user) = verify_cookie(&req) {
        req.set_extension("authenticated", serde_json::json!(true));
        req.set_extension("user_id", serde_json::json!(user.id));
        req.set_extension("username", serde_json::json!(user.name));
    }
    if req.path == "/" && !req.is_authenticated() {
        return Err(ResumaError::Redirect("/login".into()));
    }
    Ok(req)
}

#[layout("/")]
fn RootLayout() -> View {
    let authed = current_request().is_some_and(|r| r.is_authenticated());
    // …
}
```

Session cookies on `#[submit]` success: `Redirect::to("/").with_session_cookie("sid", &token, ttl_secs)`.
See [SECURITY.md](./SECURITY.md) for CSRF, cookie helpers, and rate limits.

### Workers, artifacts, uploads

Long-running jobs use `#[worker]` (exec / Resuma OS). Large results go through
`ctx.artifact_put` and `FlowEngine::last_artifact` / `last_result_field` instead of
scanning the full event replay by hand. Multipart uploads use `POST /_resuma/upload`
or `#[upload]`. Env knobs and auth for these routes are documented in
[SECURITY.md](./SECURITY.md).

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
| `resuma::realtime` | WebSocket rooms / peers — [REALTIME.md](./REALTIME.md) |
| `resuma::exec` | Workers, artifacts, queue |

## Cookbook recipes

| Recipe | Docs |
|--------|------|
| Portal | `/docs/cookbook/portals` |
| View transitions | `/docs/cookbook/view_transitions` |
| Theme | `/docs/cookbook/theme` |
| Streaming loaders | `/docs/cookbook/streaming_loaders` |
| Docker / Fly deploy | `/docs/cookbook/docker` |
| Query loaders + `public/` + booking | [FLOW_COOKBOOK.md](./FLOW_COOKBOOK.md) |
