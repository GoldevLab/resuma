# Stability policy (1.0)

Resuma **1.0.0** marks a stable public API for production use.

## Semver

We follow [Semantic Versioning 2.0.0](https://semver.org/):

| Bump | When |
|------|------|
| **MAJOR** | Breaking changes to documented public API |
| **MINOR** | New features, backward compatible |
| **PATCH** | Bug fixes, security patches |

**Public API** includes:

- `resuma` crate root and `prelude` re-exports
- Proc macros: `view!`, `#[component]`, `#[server]`, `#[load]`, `#[submit]`, `#[derive(Store)]`, etc.
- `ResumaApp`, `FlowApp`, `ServeOptions`, `FlowServeOptions`
- `/_resuma/*` HTTP routes and resumability payload shape
- CLI commands: `new`, `dev`, `build`, `routes`, `doctor`

**Not stable (may change in minors):**

- `#[doc(hidden)]` modules (`__private`)
- Undocumented internals
- `examples/` and audit apps

## Deprecations

Breaking removals require:

1. Deprecation in a **minor** release with `#[deprecated]` or docs notice
2. At least **one minor** (≈2 release cycles) before removal

## MSRV (Minimum Supported Rust Version)

**Rust 1.91+** (see `rust-version` in workspace `Cargo.toml`).

Policy: MSRV increases only in **minor** releases, aligned with Rust N-2 when practical.

## Runtime size budget

Documented in `runtime/scripts/measure.mjs` and enforced in CI:

| Asset | Gzip budget |
|-------|-------------|
| `loader.js` | ≤ 1024 B |
| `core.js` | ≤ 6600 B |
| `flow.js` | lazy (exec/Flow widgets only) |

Benchmark methodology: `benchmark/README.md`.

## Support

- Security: [SECURITY.md](../SECURITY.md)
- Issues: GitHub Issues on the official repo
- Docs: [resuma-docs.fly.dev](https://resuma-docs.fly.dev/docs)

## Explicitly out of 1.x scope

- WASM / Leptos-style client hydration
- Pure CSR without SSR
- Built-in auth product (middleware + templates only)
- First-party ORM
