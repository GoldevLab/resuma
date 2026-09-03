---
name: resuma
description: >-
  Build and debug Resuma Rust SSR apps — view!, signals, Flow routing, #[server],
  #[load], #[submit], HtmlTheme / ThemeSwitch, Popup / Modal / GestureView,
  SeoKit / SITE_URL (SEO GEO AEO), deploy (Fly/Docker, not Lambda/Workers),
  Resuma OS (workers, queue, scheduler). Use for reactivity bugs, live palettes,
  native overlays, exec/HTTP integration, dynamic Flow panels, tests/E2E,
  and resuma-docs live demos.
---

# Resuma framework skill

Resuma is a **resumable SSR** Rust web framework (Qwik-like, Rust-native). Components run **once on the server**; the client resumes signals and lazy handler JS — no WASM hydration by default.

**Repos:** framework `resuma/` · docs site `resuma-docs/` · live docs https://resuma-docs.fly.dev/docs

**This file is the install template.** `resuma install skill` embeds it via `include_str!`. After changing it, copy to `resuma-docs/.cursor/skills/resuma/SKILL.md` and tell users `resuma install skill --force` so Cursor / agents pick up the new traps.

## When to use this skill

| Area | Triggers |
|------|----------|
| **UI / reactivity** | `view!`, signals, `<Show>`, `<For>` / `virtual`, effects, islands, `js!`, `ClientComponent` |
| **Overlays / theme** | `<Popup>`, `<Modal>`, `<GestureView>`, `HtmlTheme`, `<ThemeSwitch>`, `data-r-theme`, `html[data-theme]`, `with_html_theme` |
| **Desktop UI** | `__resuma.announce`, `measure`, `storage`, `online`, `presence`, `set_page_dir`, `set_page_theme` |
| **Flow** | `FlowApp`, `src/pages/`, `#[load]`, `#[submit]`, middleware, NavLink SPA |
| **Server** | `#[server]`, `#[submit]`, CSRF, origin checks, `ResumaApp` vs `FlowApp` |
| **SEO / GEO / AEO** | `SeoKit` (prelude), `SITE_URL`, `set_page_title`, `llms.txt`, sitemap, canonicals, FAQ JSON-LD |
| **Deploy** | Fly, Docker, `RESUMA_ADDR`, trust proxy — **not** Lambda, Cloudflare Workers, or Vercel JS adapters |
| **Resuma OS** | `#[worker]`, `FlowEngine`, `/_resuma/*`, queue, scheduler, webhooks |
| **Flow widgets** | `flow.js`, execution graph, event stream, ops dashboard |
| **Tests** | `crates/resuma/tests/`, `exec/tests.rs`, `e2e/run.mjs`, Playwright |
| **Docs** | `resuma-docs` live demos, Theme/Explore/Search chrome, sidebar pages |

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

### `js!` events (`target` vs `currentTarget`)

Handlers are delegated from `document`. The runtime sets `event.currentTarget` to the node that declared the handler (`data-r-on:*`).

- `event.target` — the node that was clicked or typed into (a child `<span>` inside a button).
- `event.currentTarget` — the handler element. Use `.dataset` and `.closest("form")` here.

Chrome in a layout (`src/pages/_layout.rs`) lives in a lazy chunk (`/_resuma/handler/__page__.js`). The first click waits on that import. For theme/nav that must work instantly, use `FlowApp::with_html_theme` + `data-r-theme` / `<ThemeSwitch>`, a boot script in `<head>`, or `data-r-inline`. Do **not** wrap document theme swaps in `startViewTransition` while a popover is open (Chromium skips the update callback).

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
use resuma::prelude::*; // SeoKit, HtmlTheme, FlowApp, set_page_*

let origin = std::env::var("SITE_URL")
    .unwrap_or_else(|_| "https://my-app.fly.dev".into());

let kit = SeoKit::new("My App", &origin)
    .with_locale("en_US")
    .with_llms_summary("My App does X for Y. Disambiguate lookalikes.")
    .with_llms_section("Docs", format!("{origin}/docs — how to use it."));

FlowApp::new()
    .with_title("My App")
    .with_description("Homepage snippet — unique per site, override per page.")
    .with_site_url(&origin)
    .with_og_image("/og.png") // PNG/JPEG 1200×630 in public/; avoid SVG for Facebook/LinkedIn
    .with_seo_kit(kit)       // /robots.txt + /llms.txt + sitemap hint — call once
    .with_sitemap_exclude(["/search"]) // noindex / thin URLs
    .with_html_theme(
        HtmlTheme::new(["paper", "slate", "midnight"])
            .dark(["midnight"])
            // .cookie("my_app_theme")      // default: resuma_theme
            // .storage_key("my-app-theme") // default: resuma-theme
    )
    .auto_pages(path_to_pages, PagesRegistry)
    .serve(FlowServeOptions::default())
    .await
```

```bash
resuma routes --generate --path src/pages   # → src/pages/_registry.rs
```

Each page: `pub fn page(req: FlowRequest) -> View` in `src/pages/...` (owned `FlowRequest`, not a reference).

`SeoKit` is in `resuma::prelude`. Kit routes stay on `ResumaApp::into_router`; Flow only adds a fallback `/robots.txt` when **no** kit is set. Do not remount robots yourself.

---

## Overlays, live theme, desktop UI

Public API is Rust `view!`. Overlay JS is lazy `/_resuma/ui.css` + `/_resuma/ui.js` only. Do **not** put overlay logic in `loader.js`. Do **not** `import("./ui.js")` from core (that bundles ui into core and blows the gzip budget).

Docs: https://resuma-docs.fly.dev/docs/components/popup · [modal](https://resuma-docs.fly.dev/docs/components/modal) · [theme](https://resuma-docs.fly.dev/docs/cookbook/theme) · [desktop UI](https://resuma-docs.fly.dev/docs/components/desktop_ui) · [virtual For](https://resuma-docs.fly.dev/docs/components/virtual_for)

### Popup (anchored popover)

SSR emits `popovertarget` + `[popover=auto]` + CSS `anchor-name` / `position-anchor` / `position-try-fallbacks`. Chromium needs no overlay JS. Safari without CSS anchor loads `ui.js`.

```rust
view! {
    <Popup id="menu" positions="bottom right top left" dismissIfShown>
        <button slot="anchor" type="button">"Menu"</button>
        <div>
            <a href="/docs">"Docs"</a>
        </div>
    </Popup>
}
```

- No `slot="anchor"` → first child is the trigger.
- Framework forces `type="button"` on the trigger (last-write-wins attrs). Host `style` is **concatenated** with `anchor-name`, not replaced.
- Do **not** put `position: relative` / `overflow: hidden` on the popup **panel** (breaks top-layer hit-testing). `.r-popup` already sets `position: absolute` + `overflow: visible`.
- Native `toggle` updates `aria-expanded` on invokers.
- Programmatic: `__resuma.showPopup("menu")` / `hidePopup("menu")`.

### Modal (stacked `<dialog>`)

```rust
view! {
    <Modal id="confirm" closedBy="any">
        <button slot="trigger" type="button">"Delete"</button>
        <h2>"Delete this job?"</h2>
        <form method="dialog">
            <button type="submit">"Cancel"</button>
        </form>
    </Modal>
}
```

- No `slot="trigger"` → a **leading `<button>`** becomes the opener. Non-button first children stay in the dialog (programmatic `showModal`).
- Chrome: Invoker Commands, no JS. Safari: click fallback in `ui.js`.
- `__resuma.showModal` / `hideModal` / `dismissAll`.

### GestureView / virtual For

`<GestureView preferredPan="horizontal">` — pan/pinch/long-press; handlers are **not** in the loader event list (`ui.js` calls `runHandler`).

```rust
view! {
    <div data-r-virtual-scroller="true" style="height: 24rem; overflow: auto">
        <For each={jobs} key="id" virtual itemHeight={48} overscan={6} let:job>
            <div>{job.title.clone()}</div>
        </For>
    </div>
}
```

SSR paints the first window. Row recycle lives in `ui.js`.

### Live palettes (`html[data-theme]`)

Not a layout `onClick`. `with_html_theme` injects a **blocking** head boot that listens for `[data-r-theme]` on **click** (not `pointerdown` — that hid the popover and retargeted the click).

```rust
view! {
    <Popup id="themes">
        <button slot="anchor" type="button">"Theme"</button>
        <ThemeSwitch id="slate">"Slate"</ThemeSwitch>
        <button type="button" data-r-theme="midnight">"Midnight"</button>
    </Popup>
}
```

- Selected chip: `[aria-pressed="true"]` / `.r-theme-on`. The boot toggles them. **Do not** bake a selected class from the SSR cookie (it stays after a live swap).
- Cookie + `localStorage`; HTTPS sets `Secure`. Ids: `[A-Za-z0-9_-]`, max 48. `HtmlTheme::new(["light","dark"])` infers dark `color-scheme` without `.dark()`.
- `prefers-color-scheme` listener when nothing is stored.
- `set_page_theme("slate")` overrides SSR for that response. `provide_theme` / `theme_css_vars` are **one-shot inline snapshots** — they do not follow `html[data-theme]`.
- SPA NavLink **must not copy** `data-theme` from prefetch HTML (the prefetch can be older than a palette pick). `dir` **is** copied so RTL survives.
- Skip View Transitions while `:popover-open` / `dialog[open]`, and on soft `invalidate` / `loader_poll` (`!pushState && !scroll`).

### Desktop helpers (`__resuma.*`)

Every interactive page gets `#r-live` (`role="status"`), including **streaming** SSR (`stream_tail`). Prefer `__resuma.announce("Saved")` over per-widget `aria-live`.

```javascript
__resuma.announce("Saved");
__resuma.measure(el);              // { x, y, width, height }
__resuma.isNavigatingWithKeyboard();
__resuma.contentSizeMultiplier();
__resuma.presence();               // { idle, hidden } — loader_poll skips idle ticks
__resuma.online();
__resuma.storage.set("k", "v");    // no-op if localStorage throws — not the document palette API
__resuma.focus(el);                // deferred; shared with SPA / popup / dialog
```

`FlowApp::with_dir("rtl")` or `set_page_dir("rtl")` sets `<html dir>`. Use logical CSS.

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
| `GET\|POST /scheduler` | API key | Cron jobs |
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
- Header dogfood: **Theme** = `<Popup>` + `HtmlTheme` / `data-r-theme`; **Explore** = `<Popup>`; **Search** = `<Modal>` (`/` shortcut). Apply theme instantly — no View Transition around the swap.
- Worker showcase: `src/site/exec_demo.rs` + `src/site/workers.rs` (`docs_showcase`).
- Home: ultralight, links to "Try it in the docs".
- **Copy page / Copy nav** — docs chrome for pasting markdown into an AI (AEO/GEO).
- Canonical origin is `SITE_URL` (`https://resuma-docs.fly.dev` until a custom domain).
- Deploy: user will push GitHub then `fly deploy` — do not push or deploy unless asked.
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
cd runtime && npm run build && cp dist/loader.js dist/core.js dist/flow.js dist/runtime.js dist/ui.js dist/ui.css ../crates/resuma/assets/
npm run e2e
```

`include_str!` embeds assets — rebuild the docs / app binary after copying. Core gzip budget is **10000** (`runtime/scripts/measure.mjs`).

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
11. **Canonical / OG / sitemap on localhost** — `SITE_URL` must be the public origin (not `http://127.0.0.1`).
12. **Duplicate docs URLs** — 301 the alias; `with_sitemap_exclude` the old path.
13. **Theme click does nothing / first click waits** — switcher `onClick` is in a layout handler chunk. Use `with_html_theme` + `[data-r-theme]`.
14. **Two theme chips look selected** — SSR baked a selected class; use `[aria-pressed]` / `.r-theme-on`.
15. **Theme reverts after NavLink** — SPA copied `data-theme` from a stale prefetch. Framework must leave the live attribute alone.
16. **SPA swap skipped / content stale with menu open** — View Transition + open popover. Skip VT while `:popover-open`.
17. **`FlowApp::with_seo_kit` panic** — `/robots.txt` mounted twice. Kit once on the app; Flow must not remount it.
18. **Popup trigger submits a form / lost `style`** — merge must force `type="button"` and concatenate `style` with `anchor-name`.
19. **No screen-reader “Saved” on streaming pages** — `#r-live` must be in `stream_tail`, not only the non-stream shell.

---

## CLI

| Command | Purpose |
|---------|---------|
| `resuma new` | Scaffold (`basic`, `todo`, `flow`, `flow-booking`, `flow-fullstack`, `production`) |
| `resuma dev` | Hot reload (pidfile reclaims leftover `cargo-watch` for the same project) |
| `resuma dev --no-update` | Skip the “update Resuma?” prompt (`RESUMA_SKIP_UPDATE=1` too) |
| `resuma dev --kill-stale` | Opt-in: free the listen port (`fuser -k`); default is pick the next free port |
| `resuma routes --generate` | Regenerate page registry |
| `resuma add sqlx` / `turso` / `tailwind` | Integrations |
| `resuma install skill` | Copy this skill to `~/.cursor/skills/resuma/` |
| `resuma install skill --force` | Overwrite after a CLI / template upgrade (picks up HtmlTheme, overlays, SEO traps) |
| `resuma install skill --project` | `.cursor/skills/resuma/` in the current repo |
| `resuma install skill --target agents` | `~/.agents/skills/resuma/` |
| `resuma install skill --target all` | Cursor global + agents |
| `resuma doctor` | Toolchain health |

Build runtime assets: `cd runtime && npm run build` then copy `loader.js`, `core.js`, `flow.js`, `runtime.js`, `ui.js`, `ui.css` into `crates/resuma/assets/`.

---

## SEO / GEO / AEO

Canonicals, Open Graph, JSON-LD, sitemap `<loc>`, and `llms.txt` links all follow **`SITE_URL`** (no trailing slash). Ship on `*.fly.dev` today; tomorrow set `SITE_URL=https://docs.example.com` and redeploy. Do not hardcode the host in page bodies — use relative `/docs/…` links.

Guide: https://resuma-docs.fly.dev/docs/integrations/seo_geo

```rust
// Per page (layout or page fn). View is built before the document head, including streaming SSR.
set_page_title("Workers | My App");
set_page_description("Background workers next to SSR — one binary.");

// Search / faceted / thin URLs:
set_page_robots("noindex, follow");

// FAQ JSON-LD only when the same Q&A is visible HTML (not a rich-result guarantee).
set_page_json_ld(faq_graph);

// Alias → canonical (omit the alias from the sitemap):
stage_response_status(301);
stage_response_redirect("/docs/cookbook/deploy");
View::empty()
```

| Control | What it does | What it does **not** do |
|---------|----------------|-------------------------|
| `robots.txt` | Crawler **access** | Indexing guarantee |
| `meta robots` / `X-Robots-Tag` | Index / snippet hints | Ranking |
| Canonical | Preferred URL signal | Merge duplicates by itself |
| Sitemap | Discovery | Indexing |
| `llms.txt` | Assistants / some AI crawlers | Google ranking (Google: neither helps nor harms, 2026-08) |

**GEO crawler split (OpenAI):** `OAI-SearchBot` = ChatGPT Search discovery; `GPTBot` = possible training; `ChatGPT-User` = user-triggered fetch (robots may not apply the same way). Do not treat one directive as a switch for all three.

**AEO:** one H1, a lead paragraph that answers before the first code sample, FAQ as H2+paragraph in SSR HTML. Do not hide the answer behind a client-only fetch.

**Do not:** same description on every URL; list 301 aliases or search URLs in the sitemap; `meta name="og:*"` (use `property`); keyword stuffing; fake Review/AggregateRating; Lambda/Workers “SEO adapters”.

**OG image:** `public/og.png` 1200×630 PNG/JPEG. `FlowApp::with_og_image("/og.png")` + `with_public_dir` (or `CARGO_MANIFEST_DIR=/app` in Docker).

---

## Deploy (long-running process — not JS serverless)

Resuma’s entry is `FlowApp::serve()` / `ResumaApp::serve()` (TCP + Tokio + optional disk). **There is no Lambda, Cloudflare Workers, or Vercel Edge adapter.** Qwik can target those because the app is a JS `fetch` handler; Resuma is not.

Production pattern (Fly / Docker):

```
RESUMA_ADDR=0.0.0.0:8080
RESUMA_TRUST_PROXY=1
RESUMA_TRUSTED_PROXY_CIDRS=fdaa::/16   # Fly 6PN; other fabrics use their proxy CIDR
SITE_URL=https://my-app.fly.dev        # then your custom domain
CARGO_MANIFEST_DIR=/app               # so public/ resolves in the image
RESUMA_DATA_DIR=/data                 # writable by the non-root user
```

Health: `GET /health`. Copy `public/` into the image. Docs: https://resuma-docs.fly.dev/docs/cookbook/deploy

Custom domain cutover: certs + DNS → change `SITE_URL` → deploy → 301 old `.fly.dev` → new Search Console property + sitemap. Do not 301 until HTTPS works on the new host.

---

## ClientComponent

```rust
ClientComponent::new("hero-particles").lazy(true) // idle import(); skip if prefers-reduced-motion
```

Register with `FlowApp::client_asset("hero-particles", include_bytes!("../static/client/hero-particles.js"))`.

---

## Docs & references

- https://resuma-docs.fly.dev/docs
- https://resuma-docs.fly.dev/docs/cookbook/theme
- https://resuma-docs.fly.dev/docs/components/popup
- https://resuma-docs.fly.dev/docs/components/modal
- https://resuma-docs.fly.dev/docs/components/desktop_ui
- https://resuma-docs.fly.dev/docs/integrations/seo_geo
- https://resuma-docs.fly.dev/docs/integrations/ai_assistant
- https://resuma-docs.fly.dev/docs/cookbook/deploy
- https://docs.rs/resuma/1.3.1
- In-repo: `docs/SECURITY.md`, `ROADMAP.md`, `CHANGELOG.md`, `docs/FLOW_COOKBOOK.md`
