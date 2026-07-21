---
name: resuma
description: >-
  Build and debug Resuma Rust SSR apps — view!, signals, Flow routing, #[server],
  #[load], #[submit], Resuma OS (workers, queue, scheduler, flow.js widgets).
  Use for reactivity bugs, exec/HTTP integration, dynamic Flow panels, tests/E2E,
  and resuma-docs live demos.
---

# Resuma framework skill

Resuma is a **resumable SSR** Rust web framework (Qwik-like, Rust-native). Components run **once on the server**; the client resumes signals and lazy handler JS — no WASM hydration by default.

**Repos:** framework `resuma/` · docs site `resuma-docs/` · live docs https://resuma-docs.fly.dev/docs

## When to use this skill

| Area | Triggers |
|------|----------|
| **UI / reactivity** | `view!`, signals, `<Show>`, `<For>`, effects, islands, `js!` |
| **Flow** | `FlowApp`, `src/pages/`, `#[load]`, `#[submit]`, middleware, NavLink SPA |
| **Server** | `#[server]`, `#[submit]`, CSRF, origin checks, `ResumaApp` vs `FlowApp` |
| **Resuma OS** | `#[worker]`, `FlowEngine`, `/_resuma/*`, queue, scheduler, webhooks |
| **Flow widgets** | `flow.js`, execution graph, event stream, ops dashboard |
| **Tests** | `crates/resuma/tests/`, `exec/tests.rs`, `e2e/run.mjs`, Playwright |
| **Docs** | `resuma-docs` live demos, `exec_demo.rs`, sidebar pages |

---

## Critical reactivity rules

### Interpolate signals without `.get()` in `view!`

```rust
// ✅ Client updates
view! { <p>{count}</p> }

// ❌ SSR snapshot only — UI frozen after click
view! { <p>{count.get()}</p> }
```

Exception: `<Show when={flag.get()}>` — `.get()` in `when={}` is intentional and reactive.

### Do not use Rust `if` for client-toggled UI

Use `<Show>` or a string signal `{label}`, not `{if signal.get() { ... }}`.

### Inputs

Prefer `onInput` with `js!` — avoid `value={signal.get()}` (one-way SSR snapshot).

```rust
<input onInput={js! { state.q.set(event.target.value); }} />
```

### `js!` and signals

- `+=` on signals in `js!` must compile to `.update()` (rs2js).
- Wait for `window.__resumaCoreReady` before calling `__resuma.action` / `safeAction` if core may still be loading.

### Effects & islands

- Avoid effect dependency cycles (A→B→A deadlocks).
- `refreshIsland` must re-bind the subtree after swap.
- `registerMountCleanup` in JS bindings — clean up listeners on SPA nav.

---

## App entry points

### Minimal `ResumaApp` (component routes)

```rust
ResumaApp::new()
    .component("/", Home)
    .serve(ServeOptions::default())
    .await
```

### `FlowApp` (file-based pages)

```rust
FlowApp::new()
    .with_seo_kit(SeoKit::new("My App", "https://example.com"))
    .auto_pages(path_to_pages, PagesRegistry)
    .serve(FlowServeOptions::default())
    .await
```

```bash
resuma routes --generate --path src/pages   # → src/pages/_registry.rs
```

Each page: `pub fn page(req: &FlowRequest) -> View` in `src/pages/...`.

---

## Server actions & forms

```rust
#[server]
async fn echo(msg: String) -> Result<String> {
    Ok(format!("Echo: {msg}"))
}

// In view!:
<button onClick={js! {
    const r = await __resuma.safeAction("echo", ["hi"]);
    if (r.ok) state.out.set(r.value);
}}>"Call"</button>
```

- Prefer `safeAction` in demos — returns `{ ok, value, error }`.
- Forms: `<Form submit={handler}>` or `data-r-submit` + CSRF token from `#resuma-state`.
- Mutations: CSRF on by default (`RESUMA_CSRF`); origin check via `RESUMA_ORIGIN` / `SecurityConfig`.

---

## Resuma OS (exec layer)

Self-hosted workers, durable graphs, queue, scheduler — **no Redis**. Routes mount when `#[worker]` registered **or** `RESUMA_EXEC_ENABLED=1`.

### Define a worker

```rust
use resuma::prelude::*;
use resuma::worker;

#[worker(intent = "process items", resources = "extended")]
pub async fn my_worker(input: MyInput, ctx: WorkerContext) -> Result<Value> {
    ctx.log("started");
    let out = ctx
        .run_blocking_with_progress(|p| {
            p(10);
            let mesh = heavy_cpu(&input);
            p(100);
            mesh
        })
        .await?;
    // Large results: store as artifact instead of returning a huge JSON Value.
    let art = ctx.artifact_json(&out)?;
    Ok(json!({ "artifact_id": art.id, "bytes": art.bytes }))
}
```

- Register at compile time via `#[worker]` (`mod workers;` in `main.rs`).
- Manual: `WorkerRegistry::new().register(name, meta, run_fn).install()` — `run` must be **`fn` pointer**, not a capturing closure.
- Timeouts: `resources = "auto"` (default 30s / `RESUMA_WORKER_TIMEOUT_SECS`), `"extended"` (300s), `"none"` (unlimited), or `"600"` (seconds).
- Poll progress: `GET /_resuma/graph/{id}/status` → `{ status, progress }` (also on full snapshot). SSE progress events are throttled (~10 Hz); snapshot progress is not.
- Uploads: `POST /_resuma/upload` multipart field `file` → `{ id, url }`, or `#[upload(mime = "image/png")]` → `POST /_resuma/upload/{name}`.
- Artifacts from `ctx.artifact_*` are **bound to the graph** — fetch with `?token=` (same as SSE). Unbound `artifact_put` remains a capability URL.
- SSE lag emits a named `resync` event; Flow UI refetches replay/status.

### Start a graph

```rust
let started = FlowEngine::start("my_worker", json!({ "topic": "x" })).await?;
// started.graph_id, started.access_token, started.plan
```

### HTTP surface (`/_resuma/*`)

| Route | Auth | Notes |
|-------|------|-------|
| `POST /worker/{name}` | API key | Start graph |
| `POST /queue/{name}` | API key | Enqueue job |
| `GET /queue/{name}/stats` | API key | Queue depth |
| `GET|POST /scheduler` | API key | Cron jobs |
| `POST /scheduler/tick` | API key | Fire due jobs |
| `GET /status`, `GET /metrics` | API key (or public flags) | Ops |
| `GET /graph/{id}` | Graph token | Snapshot (+ `progress`) |
| `GET /graph/{id}/status` | Graph token | Lightweight `{status,progress}` |
| `GET /graph/{id}/replay` | Graph token | Event JSON array |
| `GET /graph/{id}/events` | Graph token (query OK) | SSE |
| `POST /graph/{id}/pause\|resume\|cancel` | Graph token **header** or API key | **No query token** on mutations |
| `POST /upload` | API key (or public) | Multipart `file` |
| `POST /upload/{name}` | API key (or public) | Named `#[upload]` handler |
| `GET /uploads/{id}` | Unguessable id | Private TTL blob |
| `GET /artifact/{id}` | Graph token (if bound) or id | Large worker result |

**Auth headers:** `Authorization: Bearer $RESUMA_EXEC_API_KEY` or `X-Resuma-Exec-Key`.  
**Graph token:** `X-Resuma-Graph-Token` (required for control POSTs); `?token=` allowed on GET/SSE only.

### Env vars (exec)

| Var | Purpose |
|-----|---------|
| `RESUMA_EXEC_API_KEY` | Admin routes (required unless public dev) |
| `RESUMA_EXEC_PUBLIC=1` | Dev-only open admin routes (ignored in production) |
| `RESUMA_DEV=1` | Dev mode; pair with `EXEC_PUBLIC` locally |
| `RESUMA_EXEC_ENABLED=1` | Mount exec routes without workers |
| `RESUMA_DATA_DIR` | Durable graphs, queue, scheduler, artifacts on disk |
| `RESUMA_WORKER_TIMEOUT_SECS` | Default worker timeout (0 = none) |
| `RESUMA_ACTION_MAX_INPUT` | Action JSON size (default 2 MiB) |
| `RESUMA_BODY_LIMIT` | HTTP body (default 10 MiB) |
| `RESUMA_UPLOAD_MAX_BYTES` | Multipart max (default 8 MiB) |
| `RESUMA_CSP_WEBGPU=1` | Add `worker-src` for WebGPU ClientComponents |

Fail-closed: no API key and not public → 401 on worker/queue/scheduler.

### Graph lifecycle

`running` → `paused` (resumable) → `done` | `failed` (cancel = failed, blocks resume).

In-memory bus dropped on terminal status; SSE falls back to durable replay. Snapshots always via durable storage.

---

## Flow widgets (`flow.js`)

Lazy-loaded: `import("/_resuma/flow.js")`. Mounts `[data-r-flow-dashboard]`, `[data-r-flow-graph]`, `[data-r-event-stream]`, `[data-r-worker-panel]`.

### SSR helpers (`resuma-flow`)

```rust
use resuma_flow::{flow_styles, flow_dashboard_poll};

view! {
    {flow_styles()}
    {flow_dashboard_poll(4000, Some(exec_status))}
}
```

### Dynamic exec panel (docs / demos pattern)

**Do not** use `core.mountFlowWidgets` for dynamic HTML — import `flow.js` directly.

```javascript
// 1. Start worker via server action → { graph_id, access_token }
// 2. Tear down previous panel widgets (children, not parent!)
const flow = await import("/_resuma/flow.js");
if (prev) flow.disconnectFlowWidgets(prev);
slot.innerHTML = "";
// 3. Build panel HTML with data-r-flow-graph, data-r-event-stream, data-r-worker-panel
slot.appendChild(panel);
// 4. Scoped mount — do NOT flush global cleanups
flow.initFlowWidgets(slot, { flush: false });
```

**Widget HTML attributes:** `data-r-flow-graph="{id}"`, `data-r-flow-graph-live="true"`, `data-r-graph-token="{token}"`, `data-r-event-stream="{id}"`, `data-r-worker-panel="{id}"`.

### Event stream — common bugs

| Symptom | Cause | Fix |
|---------|-------|-----|
| "Loading graph…" forever | `refreshGraph` errors swallowed; bad token | Check network tab; ensure token returned from `FlowEngine::start` |
| Events duplicated 2×–8× | `loadReplay` + SSE history; EventSource reconnect after graph done | Client: replay once via HTTP; SSE only while running; `es.close()` on `graph_done`; server SSE live stream = new events only |
| Stale SSE after re-run | `resuma:disconnect` on parent only | `disconnectFlowWidgets(prev)` before `innerHTML = ""` |
| Global widget leak | `initFlowWidgets(doc)` after dynamic panel | Use `{ flush: false }` scoped to slot |

`initFlowWidgets(scope, { flush: true })` (default) — full page nav, tears down all widgets.  
`initFlowWidgets(scope, { flush: false })` — dynamic panel only; calls `disconnectFlowWidgets(scope)` first.

---

## resuma-docs conventions

- **Interactive demos** live on `/docs/...` pages via `{crate::site::demos::...()}`, **not** on the marketing home.
- Worker showcase: `src/site/exec_demo.rs` + `src/site/workers.rs` (`docs_showcase`).
- Home: ultralight, links to "Try it in the docs".
- Deploy: push to GitHub → CI → Fly (avoid manual `fly deploy` unless asked).
- `RESUMA_EXEC_PUBLIC=1`, `RESUMA_DATA_DIR`, `RESUMA_TRUSTED_PROXY_CIDRS` on Fly.

---

## Testing

### Rust integration — `crates/resuma/tests/exec_http.rs`

Pattern for exec HTTP tests:

```rust
let _guard = exec_http_lock();           // global mutex — tests serialize
let _root = temp_durable("name");        // temp dir + durable + scheduler roots
enable_exec_routes();
configure_test_exec_security();          // API key + csrf: false, origin_check: false
register_echo_worker(&worker_name);      // fn pointer worker
let app = ResumaApp::new().into_router();
```

Cover: API key auth, graph token gate, replay, SSE post-completion, pause/cancel HTTP, queue enqueue, scheduler CRUD, metrics.

`WorkerRegistry::register(..., |input, ctx| ...)` works **inside** the crate; integration tests need a **top-level `fn`** worker.

### E2E — `e2e/run.mjs` + `examples/e2e`

```bash
npm run e2e          # example-e2e on :3217
npm run e2e:all      # + example-todo
```

Server env for exec E2E:

```
RESUMA_DEV=1
RESUMA_EXEC_ENABLED=1
RESUMA_ENV=development
```

`/exec` page: `#[worker] e2e_showcase`, dynamic Flow panel, assert graph leaves "Loading graph…", event list has exactly one `[start]` per run.

### What tests miss (don't assume covered)

- Webhook HTTP + outbound delivery
- `scheduler/tick` firing a real worker
- `flow.ts` unit tests (browser E2E only)
- Live SSE race during execution (partially covered)
- `resuma-flow` SSR component snapshots

### Run before shipping

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings   # if CI expects it
cd runtime && npm run build && cp dist/flow.js ../crates/resuma/assets/flow.js
npm run e2e
```

---

## Debugging checklist

1. **Click does nothing** — `__resumaCoreReady`; handler chunk loaded; console errors.
2. **Text frozen** — `{x.get()}` in interpolation → `{x}`.
3. **Show stuck** — reactive `when={signal.get()}` or `when={signal}`.
4. **Form 403** — missing CSRF header/cookie.
5. **Exec worker 403** — missing API key; or CSRF/origin on POST (disable in tests).
6. **Exec worker 401** — `RESUMA_EXEC_API_KEY` not set.
7. **Graph 401** — missing/invalid graph token.
8. **Graph control 401** — used `?token=` on POST; must use `X-Resuma-Graph-Token`.
9. **Route 404** — `resuma routes --generate`; check `_registry.rs`.
10. **Flow widgets stuck** — see event stream table above.

---

## CLI

| Command | Purpose |
|---------|---------|
| `resuma new` | Scaffold (basic, todo, flow, …) |
| `resuma dev` | Hot reload |
| `resuma routes --generate` | Regenerate page registry |
| `resuma add sqlx` / `turso` / `tailwind` | Integrations |
| `resuma install skill` | Copy this skill to `.cursor/skills/` |
| `resuma doctor` | Toolchain health |

Build runtime assets: `cd runtime && npm run build` (copies to `crates/resuma/assets/` via CLI build step).

---

## SEO / GEO

```rust
SeoKit::new("Site", "https://example.com")
    .with_default_json_ld()
    .with_llms_summary("…")  // → /llms.txt
```

---

## Docs & references

- https://resuma-docs.fly.dev/docs
- https://docs.rs/resuma/1.2.0
- In-repo: `docs/SECURITY.md`, `ROADMAP.md`, `CHANGELOG.md`
