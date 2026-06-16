# Resuma roadmap — production readiness

**Current release: v1.0.0 (Stable)** — see [STABILITY.md](docs/STABILITY.md).

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
- [x] Distributed rate limiting (Redis backend)
- [ ] Per-action rate limit buckets

---

## Explicitly out of scope

- WASM / Leptos-style hydration
- Pure CSR SPA without SSR
- Built-in auth product (middleware + templates only)
- First-party ORM

---

*Last updated: 2026-06-16 (post-1.0 minors)*
