# Contributing to Resuma

Thanks for helping improve Resuma. This repo is the **framework** (crates, runtime, examples, benchmark). The **docs site** lives in a separate repository deployed at [resuma-docs.fly.dev](https://resuma-docs.fly.dev).

## Before you start

1. Read [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the resumability model
2. Run `cargo test -p resuma --all-targets` and `cargo clippy -p resuma -p resuma-macros -- -D warnings`
3. Format with `cargo fmt --all`

## Development setup

```bash
git clone https://github.com/GolfredoPerezFernandez/resuma
cd resuma
cargo build --workspace
cd runtime && npm ci && npm run build
```

Run examples:

```bash
cargo run -p example-counter
cargo run -p example-todo
cargo run -p example-flow-demo
```

Install CLI from source:

```bash
cargo install --path crates/resuma --features cli --force
```

## Pull requests

1. Fork and create a feature branch from `main`
2. Keep changes focused — one concern per PR
3. Add or update tests when behavior changes
4. Update docs in `docs/` or the external docs-site repo if user-facing behavior changes
5. Ensure CI passes (format, clippy, tests, runtime build)

## Benchmark changes

If you change bundle sizes or add a framework to the comparison:

```bash
node benchmark/run.mjs
```

Commit updated `benchmark/results.json` and document methodology in `benchmark/README.md`.

## Publishing (maintainers)

See [`docs/PUBLISHING.md`](docs/PUBLISHING.md). CI runs `cargo publish --dry-run` on `main`.

## Code of conduct

Be respectful in issues and PRs. Report unacceptable behavior to maintainers privately.

## Questions

Open a [GitHub Issue](https://github.com/GolfredoPerezFernandez/resuma/issues) for bugs and feature requests. For security issues, see [SECURITY.md](SECURITY.md).
