# Publishing to crates.io

Resuma ships as **two crates** (Rust requires proc-macros in a separate crate):

| # | Crate | Role |
|---|--------|------|
| 1 | `resuma-macros` | `view!`, `#[component]`, `#[server]`, … + internal rs2js |
| 2 | `resuma` | Everything else: core, ssr, server, flow, router, CLI |

Users install only:

```bash
cargo install resuma
# or
resuma = "0.2"
```

## Publish order

```bash
cargo publish -p resuma-macros
# wait ~90s
cargo publish -p resuma
```

Or:

```powershell
.\scripts\publish-crates.ps1
```

## Prerequisites

1. [crates.io](https://crates.io) account + verified email + API token
2. Clean git tree (tag recommended, e.g. `v0.2.0`)
3. `cargo login`

## Internal layout (not published separately)

Runtime code lives as modules inside `crates/resuma/src/`:

```
core/    signals, view, components
ssr/     HTML rendering
server/  axum HTTP + assets/
router/  file-based routing scanner
flow/    FlowApp, loads, submits
cli/     resuma new|dev|build  [feature cli]
```

CLI templates: `crates/resuma/templates/` (embedded via `include_str!`).

## Legacy crates (v0.1.x)

`resuma-core`, `resuma-ssr`, etc. were published in v0.1.0 but are **deprecated** — use `resuma` only from v0.2 onward.
