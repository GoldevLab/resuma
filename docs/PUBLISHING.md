# Publishing to crates.io

**Next release:** resuma 0.4.2 · resuma-macros 0.4.2 · [docs.rs/resuma](https://docs.rs/resuma)

Resuma ships as **two crates** (Rust requires proc-macros in a separate crate):

| # | Crate | Role |
|---|--------|------|
| 1 | `resuma-macros` | `view!`, `#[component]`, `#[server]`, … + internal rs2js |
| 2 | `resuma` | Everything else: core, ssr, server, flow, router, CLI |

Users install only:

```bash
cargo install resuma
# or
resuma = "0.4"
```

## Publish order

```bash
cargo publish -p resuma-macros
# wait ~90s
cargo publish -p resuma --dry-run
cargo publish -p resuma
```

`resuma` depends on the matching `resuma-macros` version. A full `cargo publish -p resuma --dry-run`
only works after that macros version is visible on crates.io; before then, rely on workspace tests
and `cargo publish -p resuma-macros --dry-run`, then run the `resuma` dry-run after publishing macros.

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

### `resuma-rs2js` (do not use)

The standalone `resuma-rs2js` crate is **obsolete**. Since v0.2.0 the translator lives inside **`resuma-macros`** as `src/rs2js/` (used by `view!`, `effect!`, `computed!`, `debounce!`, and handler `onClick`).

- **No workspace crate** — nothing to merge or publish again.
- **`resuma` / `resuma-macros` never depend on it** — only `resuma` + `resuma-macros` are published.

To stop new installs from crates.io, **yank every published version** (crates.io does not allow deleting a crate name):

```bash
cargo login   # crates.io token
cargo yank --vers 0.0.0 -p resuma-rs2js   # if that version exists
cargo yank --vers 0.1.0 -p resuma-rs2js   # if that version exists
```

Your [crates.io dashboard](https://crates.io/crates/resuma-rs2js) may still list the crate under “My Crates” after yanking; it will no longer be installable via `cargo add`.
