# Changelog

All notable changes to this project will be documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

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

## [Unreleased]

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

- `repository` and `homepage` metadata now point to `https://github.com/GolfredoPerezFernandez/resuma`
- All published crates include a crate-specific `README.md` on crates.io

## [0.1.0] - 2025-05-23

### Added

- Resumable SSR framework: signals, `view!`, `#[component]`, `#[island]`
- Server actions (`#[server]`) with CSRF, rate limits, and security headers
- Resuma Flow: `#[load]`, `#[submit]`, `#[middleware]`, file-based pages
- CLI: `resuma new`, `resuma dev`, `resuma build`, `resuma routes --generate`
- Examples: counter, todo (backend security reference), flow-demo, flow-pages, website
- Documentation site and markdown guides under `docs/`

[0.4.8]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.4.8
[0.4.7]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.4.7
[0.4.6]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.4.6
[0.4.2]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.4.2
[0.4.1]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.4.1
[0.4.0]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.4.0
[0.3.3]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.3.3
[0.3.2]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.3.2
[0.3.1]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.3.1
[0.3.0]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.3.0
[0.2.3]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.2.3
[0.2.2]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.2.2
[0.2.1]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.2.1
[0.2.0]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.2.0
[0.1.1]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.1.1
[0.1.0]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.1.0
