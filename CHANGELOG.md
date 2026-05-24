# Changelog

All notable changes to this project will be documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

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

[0.3.0]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.3.0
[0.2.3]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.2.3
[0.2.2]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.2.2
[0.2.1]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.2.1
[0.2.0]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.2.0
[0.1.1]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.1.1
[0.1.0]: https://github.com/GolfredoPerezFernandez/resuma/releases/tag/v0.1.0
