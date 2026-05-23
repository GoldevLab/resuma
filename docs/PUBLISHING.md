# Publishing to crates.io

Resuma is a Cargo workspace. Publish **internal crates first**, then the umbrella `resuma` crate.

Official reference: [Cargo — Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html)

## Prerequisites

1. [crates.io](https://crates.io) account + API token
2. Clean git tree (tag `v0.1.1` recommended before publish)
3. `cargo login <token>` — create token at https://crates.io/settings/tokens
4. Crate names available on crates.io (`resuma`, `resuma-core`, …)
5. Push repo to GitHub (docs.rs builds from the tagged commit)

## First production publish

```bash
git add -A && git commit -m "chore: release v0.1.1"
git tag v0.1.1
git push origin main --tags

cargo login   # paste API token once

# Publish one crate at a time; wait ~90s between each
cargo publish -p resuma-rs2js
# … repeat for each crate in the table below
```

Or use GitHub Actions: **Actions → Publish to crates.io → Run workflow** (set `CRATES_IO_TOKEN` secret first).

## Publish order

| # | Crate | Command |
|---|--------|---------|
| 1 | `resuma-rs2js` | `cargo publish -p resuma-rs2js` |
| 2 | `resuma-core` | `cargo publish -p resuma-core` |
| 3 | `resuma-macros` | `cargo publish -p resuma-macros` |
| 4 | `resuma-ssr` | `cargo publish -p resuma-ssr` |
| 5 | `resuma-router` | `cargo publish -p resuma-router` |
| 6 | `resuma-server` | `cargo publish -p resuma-server` |
| 7 | `resuma-flow` | `cargo publish -p resuma-flow` |
| 8 | `resuma-cli` | `cargo publish -p resuma-cli` |
| 9 | `resuma` | `cargo publish -p resuma` |

Wait for each crate to appear on crates.io before publishing dependents (~1–2 min).

## Dry run (recommended first)

`cargo publish --dry-run` resolves workspace dependencies from **crates.io**, so only the first crate (`resuma-rs2js`) can be dry-run before anything is published. After each real publish, dry-run the next crate in order.

```bash
# PowerShell (repo root) — validates packaging for crate #1
.\scripts\publish-crates.ps1 -DryRun -AllowDirty

# After resuma-rs2js is on crates.io, dry-run the next crate:
cargo publish -p resuma-core --dry-run

# Local compile check for all crates (no crates.io needed):
cargo check -p resuma-rs2js -p resuma-core -p resuma-macros -p resuma-ssr \
  -p resuma-router -p resuma-server -p resuma-flow -p resuma-cli -p resuma
```

## After publish

Users install the CLI:

```bash
cargo install resuma
resuma new my-app --template todo
```

Library-only dependency (no CLI):

```toml
[dependencies]
resuma = { version = "0.1", default-features = false }
tokio = { version = "1", features = ["full"] }
```

## Docs.rs

Documentation builds automatically for each published crate. Main entry: https://docs.rs/resuma

## Examples

Workspace examples (`examples/*`) have `publish = false` and are **not** uploaded to crates.io.

CLI templates live in `crates/resuma-cli/templates/` (embedded via `include_str!`). When you change `examples/todo`, copy the updated files into `templates/todo/`.

## Version bumps

1. Bump `[workspace.package] version` in root `Cargo.toml`
2. Bump all `[workspace.dependencies] resuma-*` version keys to match
3. Update `CHANGELOG.md`
4. Tag: `git tag v0.1.1 && git push origin v0.1.1`
5. Re-run publish order (only changed crates if using cargo-yank strategy for mistakes)
