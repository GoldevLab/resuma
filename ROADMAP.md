# Resuma roadmap — production readiness

**Current release: v2.0.0 (Stable)** — see [STABILITY.md](docs/STABILITY.md).

---

## Done (1.0)

### Core & DX (Phase 2)
- [x] `#[derive(Store)]` — `{Struct}Store` trait + field setters
- [x] `<For>` sugar in `view!`
- [x] Stable loader invalidation (`invalidate_href`, `__resuma.invalidate`)
- [x] NavLink hover prefetch
- [x] `resuma doctor` — registry drift, runtime sizes, env hints
- [x] Auto `routes --generate` on dev/build
- [x] Runtime size regression gate in CI
- [x] `production` template (Docker + Fly + security stub)
- [x] Compile-time lint for `{signal.get()}` in interpolations
- [x] Unified rs2js error messages
- [x] Benchmark includes handler chunk

### Trust (Phase 3 partial)
- [x] Security HTTP tests (CSRF, origin, rate limit)
- [x] Dependabot (cargo + npm)
- [x] Deploy guide ([DEPLOY.md](docs/DEPLOY.md))
- [x] E2E in CI (`example-e2e`)

### 1.0 criteria
- [x] Semver policy documented
- [x] MSRV policy (Rust 1.91+)
- [x] Integration + E2E + security tests in CI
- [x] Benchmark reproducible in CI
- [x] Runtime loader/core budgets enforced

---

## Post-1.0 (1.x minors)

### API
- [x] Typed extractors for `#[load]` / `#[submit]` / `#[server]`
- [x] `<Match>` sugar
- [x] Reactive `<For>` (keyed client diffing)

### CLI / runtime
- [ ] Granular dev HMR (island-level)
- [ ] Static export v2 (assets + dynamic route list)
- [ ] Retire legacy `runtime.js`

### Ecosystem
- [x] SQLx / Turso in CI against ephemeral DB
- [x] E2E on `examples/todo`
- [x] Migration guides (Leptos, Qwik, raw Axum)
- [ ] npm `resuma-client` package
- [ ] Optional OpenTelemetry hooks

### Security
- [ ] External audit / OWASP checklist
- [x] Multi-process rate limiting (disk backend; Redis removed)
- [x] Per-action rate limit buckets

### Derived from False World dogfooding
(a real Resuma app hitting these gaps first — see `apps/falseworld/ROADMAP.md` Fase 1)
- [x] `resuma::realtime` — `Room`/`RoomRegistry`/`Peer`/`WsWriter` reusable WebSocket room primitives, extracted from `falseworld::multiplayer`'s hand-rolled `axum::extract::ws` room/presence/rate-limit code
- [x] `resuma doctor` — orphaned `static/client/*.js` detection (asset registered nowhere via `client_asset`/`include_bytes!`)
- [x] `FlowEngine::last_artifact` / `last_result_field` — last-result-with-artifact lookup over `FlowEngine::replay`, replacing hand-rolled graph event scans in worker-heavy apps
- [x] `ResumaError::Redirect(String)` — `#[middleware]` can now bounce a page `GET` to a real `302 Location:` (e.g. unauthenticated → `/login`) instead of only rendering the error shell with a 4xx/5xx status; needed once `falseworld` grew real session auth (`falseworld::auth`) and had to gate the game page server-side
- [x] `flow::runtime::set_current_request` — `render_with_flow` republishes the task-local `FlowRequest` right after `#[middleware]` runs, so `current_request()` (the only way a `#[layout]` body can see the request) reflects middleware-attached extensions instead of a pre-middleware snapshot; without it, `falseworld`'s `RootLayout` nav bar never showed the logged-in user despite `req.is_authenticated()` being true on the page itself

### Derived from JobBot dogfooding
(worker + SQLite + live queue UI — `apps/jobbot`)
- [x] `loader_poll(path, interval_ms)` — first-class live-dashboard poll via `__resuma.invalidate` (replaces crude `<meta refresh>` full reloads)
- [x] Soft invalidate — `replaceState`, no scroll, strip visible `_r`, bypass prefetch (poll + same-path submit redirects)
- [x] Prefetch-safe `followRedirect` after `#[submit]` / actions
- [x] `Redirect::revalidate()` — soft revalidate current path after mutations
- [x] Form pending (`aria-busy` / disable submit) during enhanced form posts
- [x] `load_all2` / `load_all3` / `load_all4` — multi-loader boundary helper (JobBot queue + profile)
- [x] Native `<Popup>` / `<Modal>` / `<GestureView>` (HTML popover + dialog; lazy ui.js fallback)
- [x] Virtual `<For>` for large lists
- [x] `__resuma.announce` / presence / storage / measure; `set_page_dir` RTL
- [ ] Optional partial loader refresh (invalidate one loader without full page SPA swap)
- [ ] Document concurrent background `tokio` worker + Flow SSR SQLite patterns (WAL / busy_timeout)

### Long-running jobs & blobs (ORBIS-driven)
- [x] Configurable worker wall-clock timeout (`extended` / `none` / secs / env)
- [x] `ctx.run_blocking` + progress on `GraphSnapshot` / `/status`
- [x] Artifact store for large worker results (not inline durable JSON)
- [x] Multipart upload API separate from trusted `public/`
- [x] Separate action vs exec JSON size limits
- [x] CSP WebGPU preset (`worker-src`)
- [x] Optional disk-backed large `public/` assets (`RESUMA_PUBLIC_DISK`)
- [x] First-class `#[upload]` macro (`POST /_resuma/upload/{name}`)
- [x] Artifact auth scoped to graph token (bound via `ctx.artifact_put`)
- [x] SSE lag → named `resync` event + client replay/status refresh
- [x] Progress emission throttle (~10 Hz; snapshot always updates)

---

## Explicitly out of scope

- WASM / Leptos-style hydration
- Pure CSR SPA without SSR
- Built-in auth product (middleware + templates only)
- First-party ORM

---

*Last updated: 2026-08-04 (Unreleased: loader_poll + load_all from JobBot dogfooding)*
