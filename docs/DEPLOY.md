# Deploying Resuma

Production checklist for Resuma and Resuma Flow apps.

## Build

```bash
resuma build
# Release binary: target/release/<crate-name>
```

The CLI builds the JS runtime, runs `cargo build --release`, and auto-regenerates Flow routes when `src/pages/` changed.

## Environment

| Variable | Production value |
|----------|------------------|
| `RESUMA_ENV` | `production` |
| `HOST` + `PORT` | `0.0.0.0` and the platform HTTP port (Fly/DO inject `PORT`) |
| `RESUMA_TRUST_PROXY` | `1` behind Fly / App Platform / nginx |
| `RESUMA_TRUSTED_PROXY_CIDRS` | Required with trust-proxy. Fly: `fdaa::/16` |
| `RESUMA_CSRF` | on (default) |
| `RESUMA_CSP` | on (default) |

See [SECURITY.md](./SECURITY.md) for the full matrix.

## Docker (multi-stage)

```dockerfile
FROM rust:1.91-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/my-app /app/server
COPY --from=builder /app/public /app/public
RUN mkdir -p /data && chown 65532:65532 /data
ENV RESUMA_ENV=production
ENV RESUMA_ADDR=0.0.0.0:8080
ENV RESUMA_DATA_DIR=/data
ENV CARGO_MANIFEST_DIR=/app
EXPOSE 8080
USER 65532:65532
CMD ["/app/server"]
```

Use the `production` template: `resuma new my-app --template production`.

## Path dependency / monorepo (Fly + GitHub Actions)

If your app depends on Resuma via a **local path** (`resuma = { path = "../resuma" }`):

1. Prefer **git tags** (or crates.io) for CI/deploy:
   ```toml
   resuma = { git = "https://github.com/GoldevLab/resuma", tag = "v1.3.1" }
   ```
   Keep a `[patch]`/`path` override only for local hacking.
2. If you must stage a sibling checkout (Docker context with `resuma/` + `app/`):
   - Checkout both repos in Actions (`path: resuma` and `path: my-app`).
   - Build with a staging context that copies `Cargo.toml`, crates, and `client-sdk/`.
   - Pin the Resuma checkout to the same tag your `Cargo.toml` expects.
3. Always commit **`Cargo.lock`** in deployable apps; for the Resuma workspace itself the lockfile is tracked so Fly/CI builds stay reproducible.

## Fly.io

```toml
app = "my-resuma-app"

[env]
  RESUMA_ENV = "production"
  RESUMA_TRUST_PROXY = "1"
  RESUMA_TRUSTED_PROXY_CIDRS = "fdaa::/16"
  RESUMA_ADDR = "0.0.0.0:8080"
  RESUMA_DATA_DIR = "/data"
  CARGO_MANIFEST_DIR = "/app"
  SITE_URL = "https://my-resuma-app.fly.dev"

[http_service]
  internal_port = 8080
  force_https = true
  auto_stop_machines = true
  auto_start_machines = true
  min_machines_running = 0

  [[http_service.checks]]
    grace_period = "20s"
    path = "/health"
    interval = "15s"
    timeout = "2s"
```

Walkthroughs for Fly, DigitalOcean, AWS (App Runner / ECS, not Lambda), Cloudflare (proxy or Containers, not Workers), Railway, and Render: https://resuma-docs.fly.dev/docs/cookbook/deploy

Deploy with one machine: `fly launch --no-deploy --ha=false` then `fly deploy --ha=false` (or GitHub Actions `flyctl deploy --remote-only --ha=false`).

Health endpoints: `/health` (liveness), `/ready` (readiness — override for DB deps).

## Static export (marketing only)

```bash
resuma dev &          # or run release binary
resuma build --static-export --out dist
```

Exports static HTML for non-dynamic routes. Copy `public/` and configure CDN separately. Interactive apps need the Rust server.

## Pre-deploy

```bash
resuma doctor
cargo test --workspace
cd runtime && npm run build && npm run size
```
