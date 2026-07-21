# Security

Resuma ships with **secure defaults** comparable to Express + Helmet + rate limiting + CSRF middleware. No extra setup is required for production, but you should understand what is enforced and how to extend it.

## Built-in protections

| Layer | What it does |
|-------|----------------|
| **Security headers** | HSTS (HTTPS only), `X-Frame-Options: DENY`, CSP with per-request nonces, COOP, CORP, `Referrer-Policy`, `Permissions-Policy` |
| **CSRF** | Cryptographically random tokens; double-submit cookie + `X-Resuma-CSRF` header on `POST /_resuma/action/*` and `POST /_resuma/submit/*` |
| **Origin check** | Rejects cross-origin POST when `Origin` / `Referer` do not match `Host` |
| **Rate limiting** | Per-IP sliding window on actions and submits |
| **Body size limit** | 10 MiB default on POST bodies (Axum `DefaultBodyLimit`) |
| **SSR escaping** | HTML text/attributes escaped; JSON state payload sanitized against `</script>` breakout |
| **JSON-LD** | Inline `application/ld+json` sanitized the same way as the resumability payload |
| **Client components** | `ClientComponent` ids restricted to `[a-zA-Z0-9_-]`; attributes escaped |
| **Middleware** | `#[middleware]` errors **block** pages, submits, and actions (401/403/429) |
| **Production mode** | Generic client error messages; hides `/_resuma/benchmark.json` |

## Environment variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `RESUMA_ADDR` | `127.0.0.1:3000` | Bind address (`host:port`) |
| `HOST` / `PORT` | `127.0.0.1` / `3000` | Used when `RESUMA_ADDR` is unset |
| `RESUMA_ENV=production` | off | Sanitized errors, hide benchmark endpoint |
| `RESUMA_TRUST_PROXY=1` | off | Trust Fly/nginx `X-Forwarded-*` for HTTPS + client IP (only behind a proxy that overwrites forwarding headers) |
| `RESUMA_CSRF=0` | on | Disable CSRF (not recommended) |
| `RESUMA_ORIGIN_CHECK=0` | on | Disable Origin/Referer validation |
| `RESUMA_BODY_LIMIT` | `10485760` (10 MiB) | Max POST body bytes (actions + multipart) |
| `RESUMA_RATE_ACTIONS` | `120` | Action RPC calls per IP per minute |
| `RESUMA_RATE_SUBMITS` | `60` | Form submits per IP per minute |
| `RESUMA_RATE_BACKEND` | `memory` (dev), `disk` (prod) | `memory` or `disk` |
| `RESUMA_EXEC_API_KEY` | — | **Required** for exec admin routes unless `RESUMA_EXEC_PUBLIC=1` (dev only) |
| `RESUMA_EXEC_PUBLIC` | off | Opt-in unauthenticated exec routes (dev only; ignored in production) |
| `RESUMA_EXEC_ENABLED` | off | Mount `/_resuma/*` exec routes even without registered workers |
| `RESUMA_RATE_EXEC_WORKERS` | `30` | Worker/queue POSTs per IP per minute |
| `RESUMA_RATE_EXEC_GRAPH` | `600` | Graph read/SSE/status per IP per minute |
| `RESUMA_RATE_EXEC_CONTROL` | `60` | Pause/resume/cancel per IP per minute |
| `RESUMA_EXEC_MAX_INPUT` | `524288` | Max worker/queue JSON input bytes |
| `RESUMA_ACTION_MAX_INPUT` | `2097152` | Max `#[server]` action JSON args bytes |
| `RESUMA_EXEC_MAX_DEPTH` | `32` | Max JSON nesting depth for exec input |
| `RESUMA_WORKER_TIMEOUT_SECS` | `30` | Default worker wall-clock timeout (`0` = none) |
| `RESUMA_UPLOAD_MAX_BYTES` | `8388608` | Max multipart upload size |
| `RESUMA_PUBLIC_DISK` | off | Serve large `public/` files from disk |
| `RESUMA_PUBLIC_INLINE_MAX` | `524288` | Inline threshold when `PUBLIC_DISK=1` |
| `RESUMA_FETCH_ALLOWLIST` | — | Optional comma-separated host allowlist for `fetch` tool |
| `RESUMA_FETCH_MAX_BYTES` | `5242880` | Max response body from `fetch` tool (5 MB) |
| `RESUMA_METRICS_PUBLIC` | off | Allow `GET /_resuma/metrics` without API key (VPC only) |
| `RESUMA_WEBHOOK_URL` | — | Single webhook URL for graph events |
| `RESUMA_WEBHOOK_URLS` | — | Comma-separated webhook URLs |
| `RESUMA_WEBHOOK_SECRET` | — | HMAC-SHA256 signing secret (`X-Resuma-Signature`) |
| `RESUMA_CSP` | on | Set `0` to disable CSP entirely |
| `RESUMA_CSP_DEV` | off | With `RESUMA_DEV=1`, CSP is off unless you set this to `1` (Qwik-style dev skip) |
| `RESUMA_CSP_REPORT_ONLY` | off | Emit `Content-Security-Policy-Report-Only` |
| `RESUMA_CSP_STRICT_DYNAMIC` | on | `'strict-dynamic'` on `script-src` when a nonce is present |
| `RESUMA_CSP_WEBGPU` | off | Add `worker-src 'self' blob:` for WebGPU widgets |
| `RESUMA_CSP_WORKER_SRC` | — | Explicit `worker-src` allowlist |
| `RESUMA_CSP_IMG_SRC` | — | Space/comma-separated extra `img-src` hosts |
| `RESUMA_CSP_SCRIPT_SRC` | — | Extra `script-src` hosts |
| `RESUMA_CSP_STYLE_SRC` | — | Extra `style-src` hosts |
| `RESUMA_CSP_CONNECT_SRC` | — | Extra `connect-src` hosts |
| `RESUMA_CSP_FONT_SRC` | — | Extra `font-src` hosts |

## Content Security Policy (Qwik-style)

Each HTML response gets a **cryptographic nonce** (stronger than Qwik’s time-based example). Inline `<style>` / `<script>` in `with_head()` receive the nonce at SSR. Production policy uses:

- `script-src 'self' 'nonce-…' 'strict-dynamic' 'unsafe-eval'` — resumability needs `'unsafe-eval'`; no blanket `'unsafe-inline'` on scripts
- `style-src` with nonce + `'unsafe-inline'` for `style=""` attributes
- `img-src 'self' data: blob:` plus `RESUMA_CSP_IMG_SRC`

In **dev** (`RESUMA_DEV=1`), CSP is **not sent** by default (like Qwik’s `if (isDev) return`). Use `RESUMA_CSP_DEV=1` to test CSP locally.

Rust configuration:

```rust
use resuma::prelude::*;

FlowApp::new()
    .serve(FlowServeOptions {
        security: SecurityConfig {
            csp: CspConfig::production(["https://cdn.example.com"]),
            ..SecurityConfig::from_env()
        },
        ..FlowServeOptions::default()
    })
    .await?;
```

Validate policies: [Google CSP Evaluator](https://csp-evaluator.withgoogle.com/).

### Fly.io / Docker

```toml
# fly.toml
[env]
  RESUMA_ENV = "production"
  RESUMA_TRUST_PROXY = "1"
```

## Authentication

Resuma does not bundle a full auth product — use `#[middleware]` and attach session
context to `FlowRequest`. Prefer **HttpOnly session cookies** set from the server:

```rust
use resuma::prelude::*;

#[submit]
async fn login(form: LoginForm) -> Result<Redirect, SubmitError> {
    let token = create_session(&form).await?;
    Ok(Redirect::to("/dashboard").with_session_cookie("session", &token, 60 * 60 * 24 * 30))
}

#[middleware]
async fn auth_session(mut req: FlowRequest) -> resuma::Result<FlowRequest> {
    if let Some(raw) = req.header("cookie") {
        if let Some(token) = cookie_value(raw, "session") {
            // load user, then:
            req.set_extension("authenticated", serde_json::json!(true));
            req.set_extension("user_id", serde_json::json!("user-123"));
        }
    }
    Ok(req)
}
```

Helpers: `set_cookie`, `clear_cookie`, `cookie_value`, `CookieOptions`.
`Redirect::with_cookie` / `with_session_cookie` attach `Set-Cookie` on both JSON
submits and no-JS 303 PRG responses. Do **not** put a `redirect` field on arbitrary
JSON payloads (e.g. `{ token, redirect }`) — only typed `Redirect` (or a sole
`{"redirect":"/…"}` legacy shape) triggers navigation.

Handlers can read `req.is_authenticated()`, `req.user_id()`, and `req.has_role("admin")`.

Returning `Err` now **blocks** the request on pages, submits, and server actions.

## Backend patterns (NestJS + Next.js)

All patterns are live in **`examples/todo`**: Controller/Service split, Guards, ValidationPipe DTOs, Interceptors, Server Actions, and revalidate-style refetch. See [BACKEND.md](./BACKEND.md) and `/docs/security/todo`.

## CSRF flow

1. SSR embeds `csrf_token` in `<script type="resuma/state">` and sets `__resuma-csrf` cookie.
2. Client runtime sends `X-Resuma-CSRF` + `credentials: same-origin` on fetch.
3. Flow `<Form>` helpers include hidden `_csrf` for no-JS submits.

## Rate limiting (multi-instance)

By default, dev uses an **in-memory** sliding window per IP. In **production** (`RESUMA_ENV=production`), Resuma uses a **disk-backed** rate limiter under `{RESUMA_DATA_DIR}/rate-limit/` so limits survive restarts and are shared across processes on the same volume (no Redis required).

Override with `RESUMA_RATE_BACKEND=memory|disk`. Tune exec limits with `RESUMA_RATE_EXEC_*`. For multi-region or multi-machine deploys, add edge rate limiting (Fly proxy, nginx `limit_req`) in front of Resuma — no external datastore required.

## Static assets (`public/`)

Files under `public/` are indexed at startup. By default bodies are loaded into RAM;
set **`RESUMA_PUBLIC_DISK=1`** so files larger than **`RESUMA_PUBLIC_INLINE_MAX`**
(default 512 KiB) are read from disk on each request.

Content-Types are hardened: `.html` → `text/plain`, `.svg` → `application/octet-stream`
(not executable document types). **Do not store user uploads in `public/`** — use
`POST /_resuma/upload` (multipart field `file`) and serve via `GET /_resuma/uploads/{id}`.

## Dynamic route params

Catch-all and named params from [`match_route`](../../crates/resuma/src/flow/match_route.rs)
are passed **without percent-decoding**. Handlers that use params for filesystem or shell
access must validate and sanitize them explicitly.

## Resuma OS / execution layer (`/_resuma/worker`, `/_resuma/queue`, `/_resuma/graph/*`)

The execution layer is treated as an **admin API**. Routes are **fail-closed by default**:
`RESUMA_EXEC_API_KEY` is required on all exec endpoints unless you explicitly set
`RESUMA_EXEC_PUBLIC=1` for local development (ignored when `RESUMA_ENV=production`).

Exec routes are only mounted when workers are registered (`.workers(...)`) or
`RESUMA_EXEC_ENABLED=1` is set — purely static apps do not expose the ops surface.

Dev-only endpoints (`/_resuma/island/*`, `/_resuma/benchmark.json`, `/_resuma/dev/ws`)
require `RESUMA_DEV=1`.

| Control | What it does |
|---------|----------------|
| **API key** | `RESUMA_EXEC_API_KEY` — required by default for worker/queue/scheduler/webhook admin routes |
| **Graph token** | Per-execution scoped token returned in `StartWorkerResponse.access_token`; pass to `flow_graph_auth(..., Some(token))` for SSE/UI |
| **Rate limits** | Separate buckets for workers, graph reads (default 600/min), and controls |
| **Input limits** | Exec: `RESUMA_EXEC_MAX_INPUT` (512 KiB). Actions: `RESUMA_ACTION_MAX_INPUT` (2 MiB). Body: `RESUMA_BODY_LIMIT` (10 MiB) |
| **Worker timeout** | Default 30s; `#[worker(resources = "extended")]` → 300s; `"none"` → unlimited; env `RESUMA_WORKER_TIMEOUT_SECS` |
| **Artifacts** | `ctx.artifact_put` binds to graph id; `GET /_resuma/artifact/{id}?token=` requires that graph token (or API key). Unbound `artifact_put` stays capability-URL |
| **Uploads** | `POST /_resuma/upload` or `#[upload]` → `POST /_resuma/upload/{name}`; max `RESUMA_UPLOAD_MAX_BYTES` (8 MiB) unless overridden per-macro |
| **SSE lag** | Broadcast lag emits named `resync` event; Flow UI refetches replay/status |
| **SSRF guard** | `fetch` tool blocks private IPs, localhost, metadata hosts; optional `RESUMA_FETCH_ALLOWLIST` |
| **DNS rebinding** | Outbound `fetch`/webhooks resolve DNS and pin connections to validated IPs |
| **Redirect safety** | Root-relative only; rejects encoded `%2f`, `%5c`, and post-decode `//` open redirects |
| **Chunk IDs** | Handler/island/client asset IDs restricted to `[A-Za-z0-9_-]` at register and serve time |
| **Queue disk** | Job IDs validated before path construction (parity with scheduler) |
| **Unguessable IDs** | Graph IDs are cryptographic (`g_<hex>`), not sequential |

```rust
// Start worker (server-side) — returns access_token for UI
let started = FlowEngine::start("my_worker", input).await?;
view! {
    {flow_graph_auth(started.graph_id.0.clone(), true, started.access_token.clone())}
    {worker_panel_auth(started.graph_id.0.clone(), started.access_token)}
}
```

External callers use `Authorization: Bearer <RESUMA_EXEC_API_KEY>` or `X-Resuma-Exec-Key`.

### Scheduler (disk cron)

Recurring jobs without external cron — persisted under `{RESUMA_DATA_DIR}/scheduler/jobs/`:

```bash
# List schedules (admin API key required)
GET /_resuma/scheduler

# Create: run worker every hour via default queue
POST /_resuma/scheduler
{
  "name": "nightly-sync",
  "cron": "@hourly",
  "worker": "sync_worker",
  "input": {},
  "queue": "default"
}

DELETE /_resuma/scheduler/{id}
POST /_resuma/scheduler/tick   # manual fire due jobs (ops)
```

Cron presets: `@hourly`, `@daily`, `@weekly`, `@monthly`, `@every_minute`, or 5-field (`*/5 * * * *`).

`RESUMA_SCHEDULER_TICK_SECS` (default `30`) — background poll interval.

### Ops status

```bash
GET /_resuma/status   # workers, active graphs, queue depths, scheduler (admin API key)
```

Response includes `uptime_ms`, registered worker names, graph counts (`running` / `paused`), per-queue `pending`/`processing`/`done`/`failed`, and scheduler `due` count.

### Prometheus metrics

```bash
GET /_resuma/metrics   # text/plain Prometheus exposition
```

Requires API key unless `RESUMA_METRICS_PUBLIC=1` (use only inside a private network).

Example scrape config:

```yaml
scrape_configs:
  - job_name: resuma
    metrics_path: /_resuma/metrics
    authorization:
      credentials: $RESUMA_EXEC_API_KEY
```

Key series: `resuma_exec_graphs_total`, `resuma_exec_queue_jobs`, `resuma_exec_webhooks_total`.

### Webhooks

Notify external systems when graphs finish, fail, or pause:

```bash
# Env (boot-time)
RESUMA_WEBHOOK_URLS=https://hooks.example.com/resuma
RESUMA_WEBHOOK_SECRET=your-hmac-secret

# Or register at runtime (admin API key, SSRF-checked URLs)
POST /_resuma/webhooks
{ "url": "https://hooks.example.com/resuma", "events": ["graph.done", "graph.failed"] }
```

Payload:

```json
{
  "event": "graph.done",
  "graph_id": "g_abc123",
  "worker": "lead_agent",
  "status": "done",
  "duration_ms": 4200,
  "timestamp_ms": 1710000000000,
  "result": { "ok": true }
}
```

Signed with `X-Resuma-Signature: sha256=<hex>` when `RESUMA_WEBHOOK_SECRET` is set.


## Deployment checklist

- [ ] Set `RESUMA_ENV=production` and `RESUMA_TRUST_PROXY=1` behind a reverse proxy
- [ ] Set `RESUMA_EXEC_API_KEY` to a long random secret (32+ chars)
- [ ] Set `RESUMA_DATA_DIR` to a persistent volume (queue + durable + rate limits)
- [ ] Force HTTPS at the edge (`force_https = true` on Fly)
- [ ] Add auth middleware for protected routes and sensitive `#[server]` actions
- [ ] Validate input in every `#[server]` / `#[submit]` handler (length, format, authz)
- [ ] Run container as non-root (see `site-docs` `Dockerfile`)
- [ ] Keep secrets in env / secret manager — never commit `.env`
- [ ] Deploy: see [cookbook/docker](/docs/cookbook/docker) on the docs site (Fly.io + Dockerfile)

## Rate limiting (multi-instance)

Resuma rate limits are **in-process** (sliding window per IP). They reset when the process restarts. For Fly.io, Kubernetes, or multiple replicas, add edge rate limiting (Fly proxy, Cloudflare, nginx `limit_req`) in front of Resuma. Tune defaults with `RESUMA_RATE_ACTIONS` and `RESUMA_RATE_SUBMITS`.

## Trust boundaries

| API | Trust level | Notes |
|-----|-------------|-------|
| `view!` text / attributes | Safe | Auto-escaped at SSR |
| `View::raw()` / `ClientComponent` HTML | **Trusted** | Only use with static or validated content |
| `with_head()` / `with_json_ld()` | **Trusted** | Developer-controlled; JSON-LD now sanitized |
| User signal values in payload | Safe | `encode_payload()` sanitizes script breakouts |

## Reporting vulnerabilities

Open a private security advisory on GitHub or email maintainers. Do not file public issues for exploitable bugs.

## Learn more (docs site)

Browse on https://resuma-docs.fly.dev:

| Route | Topic |
|-------|--------|
| `/docs/security` | Overview |
| `/docs/security/configure` | `SecurityConfig` + env vars |
| `/docs/security/server_actions` | Validating `#[server]` in Rust |
| `/docs/security/middleware` | Flow auth middleware |
| `/docs/security/authorization` | Authorization in Rust |
| `/docs/security/backend_patterns` | NestJS + Next.js mapping |
| `/docs/security/todo` | Walkthrough of `example-todo` |

Reference implementation: `examples/todo/src/` (`main.rs`, `security.rs`, `todo_store.rs`).

Backend patterns: [BACKEND.md](./BACKEND.md).
