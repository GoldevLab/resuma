# Resuma roadmap — production readiness

Internal planning doc. Not linked from README or docs site unless we decide to later.

**Baseline:** v0.4.0 — v0.3.3 + production hardening (health/ready, graceful shutdown, request tracing), default-runtime parity (NavLink/redirect/safeAction), non-panicking loaders, flash-after-redirect, real static export, and two security/stability fixes (origin-port check, prod WS reconnect loop).

---

## Done (do not re-build)

- SSR + resumability (signals, `view!`, lazy handlers)
- Flow: pages, `#[load]`, `#[submit]`, `#[middleware]`, layouts, streaming loaders
- `#[server]` RPC + CSRF + rate limits + origin check
- Progressive-enhancement forms
- CSP nonces, HTML/JSON-LD escaping, client id validation
- Resuma Client (TypeScript widgets), optional islands
- v0.3.3: redirects/PRG, NavLink SPA, `<Show>`, `load_boundary` / `error_boundary`, `__resuma.safeAction()`
- v0.4.0: `/health` + `/ready`, graceful shutdown (SIGTERM), request-id + latency tracing, default-runtime parity (NavLink/redirect/safeAction in `core.js`), non-panicking loaders (`try_*_load` + render `catch_unwind`), flash-after-redirect, real `resuma build --static-export`, NavLink scroll/focus, **fixes:** origin-port CSRF check + prod WS reconnect loop
- crates.io publish, CLI, examples, benchmark, docs site (separate repo)

---

## Phase 1 — 0.4 Production hardening

**Goal:** deploy without surprises.

### Ops
- [x] `/health` and `/ready` endpoints (optional DB probe — apps override `/ready` for deps)
- [x] Graceful shutdown on SIGTERM (Fly/K8s)
- [x] Structured tracing: `request_id`, route, latency (OpenTelemetry hooks: future)
- [ ] Unified deploy guide (Fly, Docker, non-root) — lives in docs repo when written
- [ ] Document `FLY_API_TOKEN` setup for `resuma-docs` CI deploy

### Runtime / Flow
- [x] NavLink SPA: scroll restoration, `document.title`, focus, full-reload fallback on fetch errors
- [x] Default `core.js` parity: NavLink + redirect follow + `safeAction` (was legacy-only)
- [x] Flash messages after redirect (query-param helper `redirect_with_flash` / `flash_message`)
- [x] Integration tests: submit 303 + JSON `redirect`
- [x] Non-panicking loaders (`try_*_load` + render `catch_unwind` → error page)
- [ ] Optional session helper / template (not full auth framework)

### DX / testing
- [x] `render_view` snapshot testing — covered in `tests/ops_flow.rs`
- [x] Loader/submit error + ops integration tests without full HTTP server bind
- [ ] Minimal public test harness for loaders/submits (ergonomic wrapper)
- [ ] CI: all examples + integration tests on every PR

### Housekeeping (next convenient commit)
- [ ] Mention `../site-docs` sibling path in CONTRIBUTING only (no docs app inside this repo)

---

## Phase 2 — 0.5–0.6 Developer experience

**Goal:** ergonomics competitive with Leptos/SolidStart (without WASM).

### API
- [ ] `#[derive(Store)]` (today: manual `use_store`)
- [ ] Optional `<For>` / `<Match>` sugar over `.map()` / `match`
- [ ] Typed extractors for `#[load]` / `#[submit]` / `#[server]`
- [ ] Stable loader invalidation API (cookbook → feature)
- [ ] NavLink + prefetch (hover / viewport)

### CLI / templates
- [~] `resuma build` — pre-deploy checklist + real HTTP-crawl static export shipped in 0.4.0; remaining: asset-embed verification, size regression gate
- [ ] `resuma doctor` — crates.io vs local, `_registry.rs`, npm client bundles
- [ ] `production` template: auth middleware stub, env, Dockerfile, fly.toml
- [ ] Dev hot reload: fewer full refreshes; predictable island HMR

### Client
- [ ] Optional npm package for `resuma-client` (in addition to crate-bundled TS)
- [ ] Unified client error surface: `safeAction` + field errors + server `Result`

---

## Phase 3 — 0.7–0.8 Ecosystem & trust

**Goal:** adoption beyond early adopters.

### Integrations (CI-proven)
- [ ] SQLx / Turso example against ephemeral DB in CI
- [ ] Auth guide + template (sessions, OAuth stub) — middleware + docs, not monolithic auth crate
- [ ] One golden-path example each: i18n, Tailwind, validator
- [ ] Playwright E2E on `examples/todo` or `flow-demo` in CI

### Security
- [ ] External audit or reduced OWASP ASVS checklist
- [ ] RustSec / Dependabot in CI
- [ ] CSP report-only mode for staging
- [ ] Per-action rate limits (not only global per IP)
- [ ] Public security disclosure process (extend SECURITY.md)

### Documentation (docs repo only — not this file's scope for README links)
- [ ] Production checklist page
- [ ] Migration guides (Leptos SSR, raw Axum, Qwik)
- [ ] Full reactivity internals appendix
- [ ] 1–2 open-source apps built on Resuma

---

## Phase 4 — 1.0 Stable API

**Release when Phase 1–3 exit criteria are met.**

| Criterion | Target |
|-----------|--------|
| Semver | Strict; deprecations with ~2 minor warning |
| Tests | Integration + ≥1 E2E in CI; high coverage on security + flow routes |
| Docs | Getting started → deploy in <30 min; docs.rs current |
| Performance | Benchmark reproducible; regression gate in CI |
| Production | docs site + ≥1 real app deployed reliably |
| Runtime size | Loader/core budget documented; no silent regressions |
| MSRV | Written policy (e.g. Rust N-2) |

### Explicitly out of 1.0 scope
- WASM / Leptos-style hydration
- Pure CSR SPA without SSR
- Built-in auth product (middleware + templates only)
- First-party ORM

---

## Priority queue (top 10)

1. ~~Health + graceful shutdown~~ — done (0.4.0)
2. ~~NavLink SPA polish + flash after redirect~~ — done (0.4.0)
3. E2E in CI (todo example)
4. ~~`resuma build` for production~~ — checklist + static export done (0.4.0); embed/size gate remaining
5. Production template (auth stub + Docker + Fly)
6. ~~Observability (request_id + tracing)~~ — done (0.4.0)
7. `#[derive(Store)]` or definitive state-management story
8. SQLx integration in CI
9. npm `resuma-client` (optional)
10. Security audit / OWASP checklist

**Next up (0.5):** E2E in CI · production template · `#[derive(Store)]` · SQLx in CI · OpenTelemetry hooks

---

## Local / repo hygiene (optional)

- [ ] Remove Windows junction `apps/site-docs` → `docs-site` when folder is renamed for real
- [ ] Revert or keep doc-only commits about `site-docs` naming (`c8c9354` resuma, `c13f9b2` resuma-docs) — cosmetic only

---

*Last updated: 2026-05-28 (post v0.4.0)*
