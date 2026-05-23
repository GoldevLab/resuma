# Resuma Flow

**Resuma Flow** is the full-stack layer of the Resuma framework. It handles pages, layouts, server data, form submissions, and middleware — with naming and APIs that are **native to Resuma**, not copied from any JS meta-framework.

## Why "Flow"?

Resuma is about **resuming** where the server left off. **Flow** is the path data takes from server → HTML → client resume:

```
#[load]  →  SSR render  →  resumability payload  →  user interaction  →  #[submit]
```

## Naming (Resuma-native)

| Concept | Resuma Flow API | Notes |
|---------|-----------------|-------|
| App builder | `FlowApp` | Wraps `ResumaApp` with routing |
| Request context | `FlowRequest` | Path, params, query, headers |
| Server data (pre-render) | `#[load]` | Runs before page render |
| Form mutation | `#[submit]` | Progressive enhancement |
| Ad-hoc server RPC | `#[server]` | Callable from handlers; optional `&FlowRequest` |
| Middleware | `#[middleware]` | Before loads/submits |
| Shared chrome | `#[layout]` | Wraps nested pages |
| Page files | `src/pages/` | File-based conventions |

We deliberately avoid `$` suffixes, `routeLoader$`, `routeAction$`, or "City" naming.

## Directory layout

```text
my-app/
  src/
    main.rs           # FlowApp bootstrap
    pages/
      index.rs        # GET /
      about.rs        # GET /about
      users/
        [id].rs       # GET /users/:id
        layout.rs     # layout for /users/*
    middleware/
      auth.rs
  Cargo.toml
```

## Example

```rust
use resuma::prelude::*;
use resuma_flow::{FlowApp, FlowRequest, FlowServeOptions};

#[component]
fn Home() -> View {
    view! { <h1>"Welcome to Resuma Flow"</h1> }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    FlowApp::new()
        .with_title("My App")
        .page("/", |_req: FlowRequest| Home::render(HomeProps::default()))
        .page("/users/:id", |req| {
            let id = req.param("id").unwrap_or("?");
            view! { <h1>"User " {id}</h1> }
        })
        .serve(FlowServeOptions::default())
        .await
}
```

## Roadmap

### v0.2 — Core primitives ✅
- [x] `use_store`, `use_context`, `<Slot />`
- [x] `prevent_default` / `stop_propagation`
- [x] `use_task` / `use_visible_task` (runtime executes visible tasks)
- [x] `resuma-flow` crate + `FlowApp` + route matching

### v0.3 — Flow macros ✅
- [x] `#[load]` proc-macro + `use_{name}_load()` + `try_use_load`
- [x] `#[submit]` proc-macro + `<Form submit={...}>` + field errors
- [x] `#[middleware]` pipeline
- [x] `#[layout]` + layout chains via `page_with_layouts`
- [x] `<NavLink>` component
- [x] CLI: `resuma routes --generate` → `mod.rs` + `_registry.rs`
- [x] Auto-wire `src/pages/` via `FlowApp::auto_pages()` + `PagesRegistry`

### v0.4 — Production ✅ (partial)
- [x] Streaming SSR (chunked head → body → tail, `FlowApp::streaming(true)`)
- [x] `stream_slot` / `stream_chunk` for loader regions
- [x] Error pages (`error_page`, `not_found_page`, loader error boundary)
- [x] Cookbook: `portal`, `with_view_transition`, `Theme` + context
- [x] Cache headers per `#[load]` (`#[load(cache = "...")]`, merged `Cache-Control` on HTML)
- [x] Deferred streaming `#[load(stream)]` (shell first, loader chunks after)
- [ ] Generic REST endpoints (`onRequest`)
- [ ] Runtime lazy bootstrap (~200 B)

## Auto-wire pages

```bash
resuma routes --generate --path src/pages
```

Generates `src/pages/mod.rs` and `src/pages/_registry.rs`. Each page file exports:

```rust
pub fn page(req: FlowRequest) -> View { /* ... */ }
```

Bootstrap:

```rust
mod pages;
use pages::PagesRegistry;

FlowApp::new()
    .auto_pages("src/pages", PagesRegistry)
    .not_found(|| not_found_page())
    .streaming(true)
    .serve(FlowServeOptions::default())
    .await
```

See `examples/flow-pages`.

## Cookbook

| Recipe | API |
|--------|-----|
| Portal | `portal("target-id", children)` → `#target-id` or `[data-r-portal-target]` |
| View transitions | `with_view_transition("name", children)` |
| Theme | `provide_theme(Theme::default())` + `use_theme()` + `theme_css_vars(&theme)` |
| Stream slot | `stream_slot("region")` — placeholder for streaming loader HTML |

## `#[server]` + request context (Phase A)

Qwik's `server$(this: RequestEvent)` maps to an optional trailing `&FlowRequest`:

```rust
#[server]
async fn greet(first: String, last: String, req: &FlowRequest) -> String {
    let ua = req.header("user-agent").unwrap_or("unknown");
    format!("Hello {first} {last} ({ua})")
}
```

- `POST /_resuma/action/:name` builds a [`FlowRequest`] (method, path, headers, query, params).
- Global `#[middleware]` runs on **page**, **submit**, and **action** requests (Qwik `plugin.ts` equivalent).
- Layout-only middleware still applies to page renders only.

```rust
#[middleware]
async fn log_all(req: FlowRequest) -> resuma::Result<FlowRequest> {
    println!("[{}] {}", req.method, req.path);
    Ok(req)
}
```

See `examples/todo` (`list_todos` logs User-Agent).

## Loader cache headers

Declare cache policy on a loader — emitted as the page `Cache-Control` header after SSR:

```rust
#[load(cache = "public, max-age=60")]
async fn home(req: &FlowRequest) -> HomeData { ... }
```

Runtime override inside the loader:

```rust
set_load_cache("home", "private, no-cache");
```

When multiple loaders run on one page, Resuma merges hints (shortest `max-age`, `no-store` wins).

Verify with curl:

```bash
curl -I http://127.0.0.1:3000/
# Cache-Control: public, max-age=60
```

## Deferred streaming `#[load(stream)]`

When `FlowApp::streaming(true)` is enabled, mark a loader as deferred so the HTML shell
streams immediately and loader HTML arrives in follow-up chunks:

```rust
#[load(stream, cache = "public, max-age=60")]
async fn home(req: &FlowRequest) -> HomeData { ... }

fn home_stream_view(data: &HomeData) -> View {
    view! { <h1>{data.title.clone()}</h1> }
}
```

In the page component, handle [`LoadValue`] and place a matching [`stream_slot`]:

```rust
match use_home_load() {
    LoadValue::Pending => view! { {stream_slot("home")} },
    LoadValue::Ok(data) => home_stream_view(&data),
    LoadValue::Err(err) => error_page(&FlowError::Loader(err)),
}
```

- The loader **name** (`home`) must match the `stream_slot("home")` id.
- `{name}_stream_view` renders the chunk HTML sent after the loader resolves.
- Cache hints from `#[load(cache = "...")]` are applied to response headers before the stream starts.
- The client runtime replaces placeholders via `template[data-r-stream-chunk]`.

See `examples/flow-demo`.

## Crates

| Crate | Role |
|-------|------|
| `resuma-core` | Signals, views, resumability |
| `resuma-flow` | Pages, loads, submits, routing |
| `resuma-server` | axum HTTP + `/_resuma/*` |
| `resuma-router` | File scanner for `src/pages/` |
