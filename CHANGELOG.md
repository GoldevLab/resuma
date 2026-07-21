# Changelog

All notable changes to this project will be documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [1.2.16] - 2026-07-21

### Added

- **Worker timeouts** — `#[worker(resources = "extended" | "none" | "600")]`,
  `Resources::extended()` / `unlimited()`, env `RESUMA_WORKER_TIMEOUT_SECS`.
  `timeout_secs = 0` disables wall-clock timeout (cooperative cancel only).
- **`WorkerContext::run_blocking` / `run_blocking_with_progress`** — CPU work on
  `spawn_blocking` without starving Tokio; progress updates the graph snapshot.
- **`GraphSnapshot.progress`** + **`GET /_resuma/graph/{id}/status`** — poll progress
  without replaying SSE event history.
- **Artifact store** — `ctx.artifact_put` / `artifact_json` → `ArtifactRef`;
  `GET /_resuma/artifact/{id}` serves large results outside durable graph JSON.
- **Multipart uploads** — `POST /_resuma/upload` (field `file`) → `UploadReceipt`;
  `GET /_resuma/uploads/{id}`; env `RESUMA_UPLOAD_MAX_BYTES` (default 8 MiB).
- **`#[upload]`** — named handlers at `POST /_resuma/upload/{name}` with optional
  `max_bytes` / `mime` allow-list (`UploadedFile` arg).
- **Action vs exec input limits** — `validate_action_input` + `RESUMA_ACTION_MAX_INPUT`
  (default 2 MiB); exec stays at `RESUMA_EXEC_MAX_INPUT` (512 KiB).
- **CSP WebGPU** — `CspConfig::webgpu()`, `FlowServeOptions::with_webgpu_csp()`,
  `RESUMA_CSP_WEBGPU=1`, `worker-src` directive.
- **`RESUMA_PUBLIC_DISK=1`** — large `public/` files served from disk (not full RAM load);
  `PublicAsset::bytes()` for memory or disk-backed bodies.
- **`Redirect::with_cookie` / `with_session_cookie` / `clear_cookie`** — attach `Set-Cookie`
  on `#[submit]` / `#[server]` PRG (303) and JSON responses so session cookies can be
  HttpOnly without `document.cookie`.
- **`set_cookie` / `clear_cookie` / `cookie_value` / `CookieOptions`** — public cookie helpers
  for session middleware.
- **Packaging** — track workspace `Cargo.lock`; CI asserts version alignment and that
  `crates/resuma/assets/*.js` match `runtime/dist` after build.

### Changed

- **Artifacts from workers** are bound to the graph id; `GET /_resuma/artifact/{id}` requires
  that graph's access token (or strict API key). Unbound `artifact_put` (capability URL) remains.
- Progress SSE/log events from `ctx.progress` are throttled (~100 ms); `GraphSnapshot.progress`
  still updates on every call.
- Default `RESUMA_BODY_LIMIT` is **10 MiB** (was 1 MiB) so multipart uploads work out of the box.
- Default `RESUMA_RATE_EXEC_GRAPH` is **600/min** (was 180) for long-job status polling.
- Runtime cache-bust query `?v=` bumped to **1.2.16**.

### Fixed

- **SSE lag** — broadcast `Lagged` now emits a named `resync` event so the flow UI refetches
  snapshot/replay instead of silently dropping events.
- **Auth redirect footgun** — a JSON field named `redirect` next to other app data
  (e.g. `{ "token", "redirect" }`) no longer triggers navigation. Only typed `Redirect`
  (`__resuma_redirect`) or a sole legacy `{"redirect":"/…"}` object counts.
- **`public/` docs** — `.html` / `.svg` are served as non-executable types (`text/plain` /
  `application/octet-stream`); use `/_resuma/upload` for user content.

## [1.2.15] - 2026-07-13

### Fixed

- **Worker Resume** — `mark_running` now sets `GraphStatus::Running` (resume previously left status as `paused`, so Pause/Cancel stayed disabled and only Replay looked active).
- **Resume EventBus reuse** — pause/resume keeps the same SSE bus so Progress/log events continue without remounting the panel.
- **Soft-pause finalizer race** — the cancelled worker no longer overwrites durable status after a newer resume has taken over.

## [1.2.14] - 2026-07-13

### Fixed

- **Runtime asset caching** — `/_resuma/{loader,core,flow,runtime}.js` no longer ship
  `Cache-Control: immutable` on fixed URLs (browsers kept stale Flow widgets for a year).
  Imports now cache-bust with `?v=1.2.14`.
- **Worker Cancel button** — enabled only while `running` or `paused` (not while syncing / done).
- **Worker controls sync** — MutationObserver mirrors graph status into Pause/Resume/Cancel
  when the sibling graph widget updates.

## [1.2.13] - 2026-07-12

### Added

- **`chunk_digests` in resumability payload** — server-side SHA digests per lazy chunk; client invalidates/warms on digest change; `ETag` on handler chunks.
- **`data-r-nav-exclusive`** — only the longest matching NavLink stays active inside exclusive nav groups (e.g. docs sidebar).
- **Structured action errors** — `field_errors` on `ActionResponse`; `safeAction` returns them; `ResumaError::validation_fields`.
- **Unified island chunk loader** — islands share generation counter, digest cache-bust, and SPA invalidation with handlers.

### Fixed

- **Docs sidebar multiple active links** — SPA nav no longer highlights every prefix-matching Overview + current page.

## [1.2.12] - 2026-07-11

### Fixed

- **Handler chunk cache on SPA navigation** — generation counter ignores stale in-flight imports after invalidation; `warmHandlerChunks()` cache-busts page chunks on mount; non-bust prefetch can no longer overwrite a fresh bust load.

## [1.2.7] - 2026-07-11

### Fixed

- **Flow worker controls** — `applyWorkerControlState` / `syncWorkerControls` now ship in built `flow.js` (duplicate TS declarations had blocked the v1.2.6 bundle). Pause/Resume/Cancel enable from live graph status; done graphs disable terminal actions.
- **Flow event stream** — `teardownGraph` + `eventStreamOwners` cleanup on panel replace; dedupe `node_done` by node label; `syncFlowControls` export for dynamic mounts.
- **Paused graphs** — no longer treated as terminal (SSE + controls stay active while paused).

## [1.2.1] - 2026-07-11

### Added

- **`GET /_resuma/flow.css`** — static Flow widget stylesheet (CSP-safe alternative to inline `flow_styles()` on dynamically injected panels).
- **`flow_styles_link()`** — `<link rel="stylesheet" href="/_resuma/flow.css">` for pages that mount Flow HTML after load.
- **`flow_execution_panel_auth`** — execution panel without embedded styles (pair with `flow_styles_link()` or layout `flow_styles()`).

### Fixed

- **Flow `flow_styles()` CSP** — inline `<style>` tags now receive the per-request CSP nonce during SSR.
- **Flow graph 401 noise** — terminal graphs show “Graph finished.” instead of token errors; completed graphs skip refresh polling.
- **Flow widget remounts** — `initFlowWidgets` dedupes graph/stream/panel mounts per `graph_id` to prevent duplicate SSE replays.
- **Handler chunk 404 MIME** — missing `/_resuma/handler/{chunk}.js` returns `application/javascript` so dynamic `import()` fails with a clear error instead of a MIME block.

## [1.2.0] - 2026-07-09

### Fixed

- **Effect deadlock on cyclic dependencies** — `Effect::run()` now routes through `run_effect` so mutual A→B→A effect cycles break cleanly instead of deadlocking on the shared callback `RwLock`.
- **`rs2js` compound assignments** — `count += 1` on signals translates to `.update()` instead of corrupting the cell in JS.
- **`rs2js` `Signal::update` closures** — block-bodied `|c| { *c += 1; }` now return the mutated param so client updates are not silently dropped.
- **`use_computed` double execution** — initializer runs the compute closure exactly once on first effect run.
- **Serialization failures** — `ResumePayload` sets `serialization_error: true` when encoding fails; client logs a clear error instead of mounting a broken page.
- **`register_client_effect` re-registration** — updates `body`, `kind`, `target`, and `debounce_ms` when the same effect id is registered again.
- **Context keys** — serialized contexts use stable `type_name::<T>()` keys instead of opaque `TypeId` debug strings.
- **`refreshIsland`** — re-binds only the swapped island subtree against existing signal cells (preserves live client state).
- **`values_equal` floats** — NaN/`-0` handling aligned with client `Object.is`.
- **`EffectId(0)` collision** — atomic fallback ids when no `RenderContext` is active (tests/direct calls).
- **SPA binding leaks** — `registerMountCleanup` unsubscribes text/attr/Show/For/Match listeners on remount.
- **`run_effect` nested tracking** — restores `current_effect` after nested runs so parent dependency tracking is not lost.
- **`<Match>` client parity** — `matchValueString()` now mirrors Rust `match_value_string` (JSON text for objects/arrays/null).
- **`visible_task!` / captures** — visible tasks register signal name→id captures; runtime builds `state.todos`-style locals. New `visible_task!` macro; `use_visible_task_with_captures` API.
- **Todo scaffold** — `templates/todo/` synced with `examples/todo` (`<For>`, `visible_task!`, reactive patterns).
- **SPA mount cleanup** — `flushMountCleanups()` tears down IntersectionObservers (visible tasks, islands, lazy chunks) and visible-task teardown callbacks on navigation.
- **Deferred stream loaders** — prefetch continues after a loader failure; remaining slots emit error chunks instead of staying pending.
- **Shared portal targets** — each portal owner mounts into a scoped `[data-r-portal-slot]` wrapper; hiding one `<Show>` no longer clears siblings in the same target.
- **Deferred stream loaders** — `#[load(stream)]` handlers now run once per request: results are prefetched before the HTTP stream starts and reused for chunks (fixes double `dispatch_load` and ensures loader failures return HTTP 500 before headers are sent).
- **`use_visible_task`** — tasks now defer until viewport (`IntersectionObserver`); eager run only when IO is unavailable.
- **View Transitions (`data-r-vt`)** — same-origin links use SPA `navigate()` instead of full page reload.
- **NavLink prefetch** — re-fetches on every hover (dedupes in-flight only); `invalidate()` clears the prefetch cache.
- **SPA remount fallback** — uses full `mountPage()` pipeline when no mounter is registered.
- **Legacy `runtime.js`** — exposes `navigate`, `buildUrl`, and `invalidate` on `__resuma`; handles `serialization_error` in payload.
- **Handler refs** — split on first `#` only (chunk ids may contain `#`).
- **`provide_context`** — returns `bool`; serialization failure no longer silently registers nothing.
- **`<Show when={…}>`** — compile error for compound expressions mixing multiple signal paths.
- **`view!` attrs** — compile error for bare `attr={signal.get()}` (SSR snapshot); nested one-shot uses like `theme_css_vars(&theme.get())` remain allowed.

### Added

- **E2E coverage** — Playwright tests for compound assign, `<Show>`, `<For>`, SPA effects replay, and todo server-action round-trip.
- **Reentrancy tests** — mutual effect cycle and `use_computed` init-count regression tests.
- **TypeScript** — shared `types.ts`, module shims, `mount-cleanups.ts`, `portals.ts`, and `tsc`-clean runtime sources.

### Removed

- **Redis rate-limit backend** — Resuma no longer depends on Redis. Production uses the built-in **disk** backend (`{RESUMA_DATA_DIR}/rate-limit/`, multi-process safe via file locks). Dev uses **memory**. Set `RESUMA_RATE_BACKEND=memory|disk`.

### Changed

- Runtime bundles rebuilt (`core.js`, `loader.js`, `flow.js`, `runtime.js`).

## [1.1.0] - 2026-07-05

### Fixed

- **`try_use_load` no longer panics** — returns `LoaderError` when called outside a Flow render scope; `ResumaApp` page renders now wrap in `with_request()` so `#[load]` works on simple apps too.
- **Conditional effect/computed dependencies** — `run_effect` clears stale deps, re-tracks on each run, and client effects subscribe to all listed capture signals (fixes branch-switching reactivity bugs).
- **CSRF validation timing** — short tokens no longer leak length via early-return before constant-time compare.
- **Route params** — percent-decoded per segment in `match_route`.
- **`provide_context`** — serialization failures log an error instead of panicking the request.

### Added

- **`try_use_context()`** — fallible context accessor returning `Option<T>`.

### Changed

- Runtime bundles rebuilt (`core.js`, `loader.js`, `flow.js`, `runtime.js`) with effect dependency fixes.
- Security, exec, and server hardening across the workspace (limits, deny policy, rate-limit, SSRF guards).

## [1.0.2] - 2026-06-16

### Fixed

- **Client component CSP** — module scripts from [`ClientComponent`](crates/resuma/src/client/mod.rs) now receive the per-request CSP nonce so they load under production `strict-dynamic` policies.

## [1.0.1] - 2026-06-16

### Fixed

- **`js!` async handlers** — when `js!{ async (...) => { ... } }` already contains a full arrow function, do not double-wrap it (fixes handlers that never ran, e.g. docs site server-function demo).

## [1.0.0] - 2026-06-16

**Stable release** — see [docs/STABILITY.md](docs/STABILITY.md) for semver, MSRV, and runtime budgets.

### Added

- **`#[derive(Store)]`** — generates `{Struct}Store` trait with field getters and `set_*` helpers on `Store<T>`.
- **`<For each={items} let:item>`** — JSX sugar over `.into_iter().map()` in `view!`.
- **Loader invalidation API** — `invalidate_href`, `invalidate_href_now`, `invalidate_link`, and `__resuma.invalidate()` (SPA re-fetch with cache-bust).
- **NavLink prefetch** — hover prefetch of route HTML before click.
- **`production` template** — Flow + security stub + `Dockerfile` + `fly.toml` + `.env.example` (`resuma new --template production`).
- **Security HTTP tests** — CSRF, origin, rate limit, action 403.
- **Docs** — [DEPLOY.md](docs/DEPLOY.md), [STABILITY.md](docs/STABILITY.md).
- **CI** — benchmark smoke (`node benchmark/run.mjs --skip-build`), Dependabot (cargo + npm).

### Fixed (since 0.4.8)

- **`debounce!` client replay** — `initEffects` honors `debounce_ms`.
- **Benchmark honesty** — first-interaction includes Counter handler chunk; version from `Cargo.toml`.
- **Compile-time lint** — bare `{signal.get()}` in `view!` interpolations fails with a reactivity hint.

### Changed

- **`resuma doctor`** — runtime bundle budgets, `_registry.rs` drift, `RESUMA_ENV` hint.
- **Auto route generation** on `resuma dev` / `resuma build`.
- **Runtime size gate** in `npm run size` (loader ≤ 1 KiB gzip, core ≤ 5 KiB gzip).
- **Unified rs2js error messages** across handler/effect/computed/debounce macros.

## [0.4.8] - 2026-06-08

### Added

- **Reactive `<Show>`** — client toggles branches via `<resuma-show>` (fixes P0 conditional UI bug).
- **`SeoKit` auto routes** — `with_seo_kit()` serves `/robots.txt` and `/llms.txt` on `ResumaApp` and `FlowApp`.
- **`resuma install skill`** — installs the Resuma agent skill for Cursor (`~/.cursor/skills/`), project (`.cursor/skills/`), or agents (`~/.agents/skills/`).
- **`example-resuma-audit`** — interactive audit app (~88 routes) with test registry, matrix, SQLx todo demo, Playwright + smoke scripts.
- Docs: [AI assistant guide](https://resuma-docs.fly.dev/docs/integrations/ai_assistant), SEO/GEO page updates.

### Fixed

- **`view!` Show macro** — correct quote expansion for `signal.get()` receivers (was generating invalid field access).
- **Portals / Show runtime** — portal target cache, `replaceChildren` on close, safer mount via `cloneNode`.
- **`use_visible_task` + ASI** — handler bodies wrapped so arrow functions execute reliably on the client.
- **Flow `public/`** — `with_public_dir` for static assets without duplicate route panics.

### Changed

- **`example-todo`** — drag-and-drop reorder (client-side) in the full showcase app.

## [0.4.7] - 2026-06-03

Stability release: SPA navigation parity, working client-replay macros, and security hardening.

### Fixed

- **SPA navigation now replays the full mount pipeline.** `<NavLink>` / `__resuma.navigate` previously re-bound only reactive text/attrs and islands, silently dropping `effect!` / `computed!` / `debounce!`, visible tasks, lazy handler chunks, portals, stream slots, and view transitions on the destination page. Both the default `core.js` and the legacy `runtime.js` now register a single per-page mounter reused on first load and every SPA navigation.
- **Client-replay macros were non-functional.** `effect!` referenced an unexported `use_effect` and, with `debounce!` / `computed!`, built capture keys via `signal.to_string()` (requires `Display`, which `Signal` does not implement) — none compiled. They now capture the variable name (`stringify!`), auto-clone listed signals into the closure so originals stay renderable, and read signal ids before the move.
- **Effect/computed bodies never executed on the client.** `initEffects` did `new Function("state", "__resuma", body)` where `body` is an arrow expression, so the arrow was created but never invoked. It is now invoked, and `computed!`'s `target` signal is assigned from the returned value.

### Security

- **Rate-limit buckets are swept.** `RATE_BUCKETS` only pruned the key being hit, so one entry per distinct client IP accumulated forever. A throttled global sweep (≤ once per window) evicts fully-expired buckets.
- **Constant-time CSRF comparison.** Token vs. cookie now uses a length-independent constant-time compare instead of `!=`, removing a token-position timing side channel.

## [0.4.6] - 2026-06-02

Flow DX release: query navigation, `public/`, and booking scaffold.

### Added

- **`__resuma.navigate` / `__resuma.buildUrl`** on the default `core.js` runtime for SPA reloads when query params change (server `#[load]` re-runs).
- **`loader_refresh_input`**, **`loader_refresh_form`**, **`query_nav_link`**, **`build_query_href`** — Rust helpers for query-driven pages.
- **`public/` auto-serve** on `FlowApp` (defaults to `{CARGO_MANIFEST_DIR}/public`), with paths merged into PWA precache.
- **`with_theme_pwa(Theme)`** — maps theme primary/background into auto PWA colors.
- **PWA icons from `public/`** — `icons/icon-192.png` etc. override generated SVG manifest entries when present.
- **`resuma new --template flow-booking`** — minimal appointments sample.
- **`resuma dev --kill-stale`** (Linux) and `cargo watch` on `public/`.
- Docs: [docs/FLOW_COOKBOOK.md](./docs/FLOW_COOKBOOK.md).

### Fixed

- **CSP `strict-dynamic`** — `loader.js` and `pwa-register.js` now receive the same per-request nonce as the resumability state script (handlers and PWA registration work in production).
- **NavLink `active` after SPA navigation with query** — `/reservar` stays active on `/reservar?fecha=…` (Rust + `core.js`).
- **PWA manifest icons from `public/`** — use the file’s real `Content-Type` (PNG/SVG).
- **Clippy** — simplified CSP `from_env` toggle (CI `-D warnings`).

## [0.4.2] - 2026-05-31

CLI onboarding patch release.

### Fixed

- `resuma new --template basic` now generates a page function returning `View`, so a fresh app compiles on the first `cargo check`.
- `resuma new --template flow-fullstack` now compiles without a live SQLx database at build time by using runtime SQLx queries instead of `query!` / `query_as!`.
- The fullstack users page imports the generated loader/submit handlers correctly and avoids the `Result` alias collision with `SubmitError`.

### Changed

- Generated projects now pin `resuma` to the same version as the CLI that created them.
- README and package metadata now reflect the current `907 B` loader and `5.08 KiB` first-interaction benchmark.

## [0.4.0] - 2026-05-28

Production hardening release: ops endpoints, graceful shutdown, request tracing,
non-panicking loaders, a unified default client runtime, and two security/stability fixes.

### Fixed

- **Origin/Referer check on non-standard ports** — `Origin: http://host:PORT` (always sent by browsers) was compared against the port-stripped `Host`, rejecting same-origin `POST` submits and `#[server]` actions with `403` on any non-80/443 port (all local dev, and direct non-proxied deploys). Ports are now stripped on both sides.
- **Production WebSocket reconnect loop** — `core.js` opened the dev HMR socket (`/_resuma/dev/ws`) unconditionally and retried every 500 ms; in production that route does not exist, causing an endless reconnect loop. The dev bridge now activates only when the dev-reload script (injected with `RESUMA_DEV=1`) sets `window.__resumaDev`.
- **Loader failures no longer abort the request** — a failed `#[load]` accessed via the panicking `use_*_load()` accessor is now caught during render and turned into the Flow error page instead of dropping the connection.

### Added

- **Default client runtime parity** — `core.js` (the default lazy path) now wires NavLink SPA navigation (`initNavLinks`), follows submit/action redirects (`followRedirect`), and exposes `__resuma.safeAction()`. Previously these v0.3.3 features only worked when apps overrode `runtime_src` to the legacy `runtime.js`. The loader also eagerly loads `core.js` when `<NavLink>` is present.
- **Ops endpoints** — built-in `GET /health` (liveness) and `GET /ready` (readiness) on `ResumaApp`/`FlowApp` (skipped if the app defines its own).
- **Graceful shutdown** — `serve()` drains connections on `Ctrl+C` and `SIGTERM` (Fly.io / Kubernetes rolling deploys).
- **Request tracing** — `x-request-id` middleware generates/propagates a correlation id (echoed on the response) and emits a `tracing` span with method, path, status, and latency. `RequestId` is available via request extensions.
- **`try_<name>_load()`** — `#[load]` now also generates a fallible accessor returning `Result<T, LoaderError>` alongside the panicking `use_<name>_load()`.
- **Flash-after-redirect** — `redirect_with_flash(path, msg)` + `flash_message(&req)`: stateless one-shot messages over a query param that survive PRG redirects (no-JS) and SPA navigation.
- **NavLink polish** — scroll-to-top on new navigations, focus management for assistive tech after an SPA swap, and a safe `remountPage` (full reload if the core has not bootstrapped).
- **Real `resuma build --static-export`** — crawls a running server over HTTP to emit actual SSR HTML (replacing the previous placeholder), with a `--base-url` flag (`RESUMA_EXPORT_BASE_URL`). `resuma build` now prints a pre-deploy checklist.

### Changed

- Runtime rebuilt: `loader.js` ~907 B gzip, `core.js` ~4.2 KiB gzip (now includes navigation + redirect + safeAction on the default path).

## [0.3.3] - 2026-05-24

### Added

- **Redirects** — `redirect()` / `Redirect` for `#[submit]` and `#[server]`; 303 PRG without JS, JSON `redirect` hint with runtime
- **NavLink SPA navigation** — client fetches SSR HTML and swaps `#resuma-root` without full reload
- **`<Show>`** — conditional rendering in `view!` (Leptos-style `when` / `fallback`)
- **`load_boundary` / `error_boundary`** — explicit loader and Result fallback UI helpers
- **`__resuma.safeAction()`** — server RPC with `{ ok, value | error }` instead of throw-only

### Changed

- Runtime rebuilt (~10.4 KiB) with navigation module and form/action redirect follow

## [0.3.2] - 2026-05-24

### Added

- **Resuma Client** — `ClientComponent`, `client_component()`, `FlowApp::client_asset()` / `static_asset()` for TypeScript widget bundles
- **`client-sdk/resuma-client.ts`** — shipped in the `resuma` crate; `bootClientComponent()` mount contract
- **`FlowApp::into_router()`** — testable axum router builder
- **Product naming guide** — `docs/NAMING.md` (Resuma / Resuma Flow / Macros / Runtime / Client / CLI)
- CLI commands on crates.io source: `resuma update`, `resuma add`, `resuma doctor`

### Fixed

- **JSON-LD XSS** — `json_ld_script()` sanitizes `</script>` breakouts
- **Stylesheet href** — `PageOptions::stylesheet` URLs HTML-escaped at SSR
- **Client component ids** — restricted to `[a-zA-Z0-9_-]`; invalid ids emit nothing
- **CSP nonces in `with_head()`** — inline `<style>` / `<script>` tags receive per-request nonces
- **Island auto-chunks** — no longer append no-op `resume()` stub (pre-registered TS islands work)
- **Static asset caching** — `Cache-Control: public, max-age=31536000, immutable` on embedded bundles
- **Clippy** — `resuma update` module clean under `-D warnings`

### Changed

- Benchmark table order: Resuma first, then Leptos, then by popularity
- README / PACKAGE.md / SECURITY.md aligned with official product names
- Security docs: trust boundaries, rate limits, CSP + `with_head()` patterns

## [0.3.1] - 2026-05-24

### Changed

- **docs.rs:** crate-level quick start, v0.3 resumability model, expanded `prelude` and module docs
- Document `ResumePayload`, `for_client()`, `ServeOptions::from_env`, `page_with_request`
- Fix `computed!` docs (remove obsolete `use_computed!` reference)
- `#[component]` / `#[island]` macro docs aligned with resumability-first model

## [0.3.0] - 2026-05-23

Major release since v0.2.2: resumability-first model, client effect replay, dev tooling, and Flow improvements.

### Added

- **Resumability everywhere:** each `#[component]` is a lazy handler boundary (`<resuma-boundary>`)
- Handler chunks externalized from HTML payload — fetched from `/_resuma/handler/{Component}.js`
- Viewport prefetch for lazy chunks via `IntersectionObserver` (`runtime/boundaries.ts`)
- Client effect replay: `computed!`, `debounce!`, and `effect!` macros (rs2js → payload `effects` → runtime)
- `payload.lazy_chunks` — chunk ids referenced on the page
- `#[island(load = "visible")]` — lazy island hydration via IntersectionObserver
- `GET /_resuma/island/:instance` — serves cached island HTML for HMR refresh
- Dev WebSocket at `/_resuma/dev/ws` when `RESUMA_DEV=1` (`resuma dev` sets this)
- `resuma build --static --out dist` — static HTML export scaffold from `src/pages/`
- HTTP integration tests (`crates/resuma/tests/integration.rs`, `lazy_chunks.rs`)
- `ServeOptions::from_env()` / `FlowServeOptions::from_env()` — bind via `RESUMA_ADDR` or `HOST`+`PORT`
- `ResumaApp::page_with_request()` / `fallback_with_request()` — HTTP context in page factories
- Flow static routes pass full `FlowRequest` (query, headers, method)
- SSR auto-registers lazy handler/island chunks from the resumability payload
- `resuma new --template flow` — file-based pages starter under `src/pages/`
- Island chunk route `GET /_resuma/island-chunk/:chunk.js` (fixes collision with HMR refresh path)
- Cryptographically random CSRF tokens (`getrandom`)
- Expanded CI: workspace check, runtime build, `cargo publish --dry-run`

### Changed

- `ResumePayload::for_client()` strips external handler sources (≤256 B `__page__` handlers stay inline)
- `#[island]` reframed as optional — resumability is the default for every `#[component]`
- Runtime `core.js` initializes client effects, boundary prefetch, and dev bridge
- `use_computed` / `use_effect` / plain `use_debounce` remain SSR-only; use macros for client replay
- `resuma build` copies JS assets to `.resuma/assets/` outside the monorepo (or `crates/resuma/assets/` in-tree)
- Scaffold templates target `resuma = "0.3"`
- `merge_payload_handlers` registers all chunks including `__page__` when oversized

### Fixed

- Missing workspace deps (`async-trait`, `ctor`) that broke fresh checkouts
- Flow pages could not read request query/headers on static routes

## [0.2.3] - 2026-05-24

### Changed

- crates.io `documentation` metadata points to the guide site (https://resuma-docs.fly.dev/docs); API remains on docs.rs

## [0.2.2] - 2026-05-23

### Changed

- Docs frame Resuma as **resumability vs hydration** — no third-party framework comparisons
- Showcase post draft: `docs/BLOG_RUST_SSR_WITHOUT_HYDRATION.md` (r/rust / Show HN templates)
- Architecture and landing pages updated on the docs site

## [0.2.1] - 2026-05-23

### Changed

- README and docs updated with crates.io / docs.rs links
- Removed third-party framework comparisons from public docs
- Benchmark endpoint reports Resuma asset sizes only

## [0.2.0] - 2026-05-23

### Changed

- **Breaking:** Consolidated 7 internal crates into a single `resuma` runtime crate (unified one-package DX).
- Only **2 crates** are published: `resuma` + `resuma-macros` (proc-macros must stay separate in Rust).
- `resuma-rs2js` merged into `resuma-macros` as an internal module.

### Fixed

- Each crate on crates.io includes a README.

## [0.1.1] - 2026-05-23

### Fixed

- `repository` and `homepage` metadata now point to `https://github.com/GoldevLab/resuma`
- All published crates include a crate-specific `README.md` on crates.io

## [0.1.0] - 2025-05-23

### Added

- Resumable SSR framework: signals, `view!`, `#[component]`, `#[island]`
- Server actions (`#[server]`) with CSRF, rate limits, and security headers
- Resuma Flow: `#[load]`, `#[submit]`, `#[middleware]`, file-based pages
- CLI: `resuma new`, `resuma dev`, `resuma build`, `resuma routes --generate`
- Examples: counter, todo (backend security reference), flow-demo, flow-pages, website
- Documentation site and markdown guides under `docs/`

[0.4.8]: https://github.com/GoldevLab/resuma/releases/tag/v0.4.8
[0.4.7]: https://github.com/GoldevLab/resuma/releases/tag/v0.4.7
[0.4.6]: https://github.com/GoldevLab/resuma/releases/tag/v0.4.6
[0.4.2]: https://github.com/GoldevLab/resuma/releases/tag/v0.4.2
[0.4.1]: https://github.com/GoldevLab/resuma/releases/tag/v0.4.1
[0.4.0]: https://github.com/GoldevLab/resuma/releases/tag/v0.4.0
[0.3.3]: https://github.com/GoldevLab/resuma/releases/tag/v0.3.3
[0.3.2]: https://github.com/GoldevLab/resuma/releases/tag/v0.3.2
[0.3.1]: https://github.com/GoldevLab/resuma/releases/tag/v0.3.1
[0.3.0]: https://github.com/GoldevLab/resuma/releases/tag/v0.3.0
[0.2.3]: https://github.com/GoldevLab/resuma/releases/tag/v0.2.3
[0.2.2]: https://github.com/GoldevLab/resuma/releases/tag/v0.2.2
[0.2.1]: https://github.com/GoldevLab/resuma/releases/tag/v0.2.1
[0.2.0]: https://github.com/GoldevLab/resuma/releases/tag/v0.2.0
[0.1.1]: https://github.com/GoldevLab/resuma/releases/tag/v0.1.1
[0.1.0]: https://github.com/GoldevLab/resuma/releases/tag/v0.1.0
