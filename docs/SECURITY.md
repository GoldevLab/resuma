# Security

Resuma ships with **secure defaults** comparable to Express + Helmet + rate limiting + CSRF middleware. No extra setup is required for production, but you should understand what is enforced and how to extend it.

## Built-in protections

| Layer | What it does |
|-------|----------------|
| **Security headers** | HSTS (HTTPS only), `X-Frame-Options: DENY`, CSP with per-request nonces, COOP, CORP, `Referrer-Policy`, `Permissions-Policy` |
| **CSRF** | Cryptographically random tokens; double-submit cookie + `X-Resuma-CSRF` header on `POST /_resuma/action/*` and `POST /_resuma/submit/*` |
| **Origin check** | Rejects cross-origin POST when `Origin` / `Referer` do not match `Host` |
| **Rate limiting** | Per-IP sliding window on actions and submits |
| **Body size limit** | 1 MB default on POST bodies (Axum `DefaultBodyLimit`) |
| **SSR escaping** | HTML text/attributes escaped; JSON state payload sanitized against `</script>` breakout |
| **Middleware** | `#[middleware]` errors **block** pages, submits, and actions (401/403/429) |
| **Production mode** | Generic client error messages; hides `/_resuma/benchmark.json` |

## Environment variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `RESUMA_ADDR` | `127.0.0.1:3000` | Bind address (`host:port`) |
| `HOST` / `PORT` | `127.0.0.1` / `3000` | Used when `RESUMA_ADDR` is unset |
| `RESUMA_ENV=production` | off | Sanitized errors, hide benchmark endpoint |
| `RESUMA_TRUST_PROXY=1` | off | Trust Fly/nginx `X-Forwarded-*` for HTTPS + client IP |
| `RESUMA_CSRF=0` | on | Disable CSRF (not recommended) |
| `RESUMA_ORIGIN_CHECK=0` | on | Disable Origin/Referer validation |
| `RESUMA_BODY_LIMIT` | `1048576` | Max POST body bytes |
| `RESUMA_RATE_ACTIONS` | `120` | Action RPC calls per IP per minute |
| `RESUMA_RATE_SUBMITS` | `60` | Form submits per IP per minute |

### Fly.io / Docker

```toml
# fly.toml
[env]
  RESUMA_ENV = "production"
  RESUMA_TRUST_PROXY = "1"
```

## Authentication

Resuma does not bundle auth — use `#[middleware]` and attach session context to `FlowRequest`:

```rust
#[middleware]
async fn require_auth(mut req: FlowRequest) -> resuma::Result<FlowRequest> {
    if req.header("authorization").is_none() {
        return Err(resuma::ResumaError::Unauthorized);
    }
    req.set_extension("authenticated", serde_json::json!(true));
    req.set_extension("user_id", serde_json::json!("user-123"));
    Ok(req)
}
```

Handlers can read `req.is_authenticated()`, `req.user_id()`, and `req.has_role("admin")`.

Returning `Err` now **blocks** the request on pages, submits, and server actions.

## Backend patterns (NestJS + Next.js)

All patterns are live in **`examples/todo`**: Controller/Service split, Guards, ValidationPipe DTOs, Interceptors, Server Actions, and revalidate-style refetch. See [BACKEND.md](./BACKEND.md) and `/docs/security/todo`.

## CSRF flow

1. SSR embeds `csrf_token` in `<script type="resuma/state">` and sets `__resuma-csrf` cookie.
2. Client runtime sends `X-Resuma-CSRF` + `credentials: same-origin` on fetch.
3. Flow `<Form>` helpers include hidden `_csrf` for no-JS submits.

## Deployment checklist

- [ ] Set `RESUMA_ENV=production` and `RESUMA_TRUST_PROXY=1` behind a reverse proxy
- [ ] Force HTTPS at the edge (`force_https = true` on Fly)
- [ ] Add auth middleware for protected routes and sensitive `#[server]` actions
- [ ] Validate input in every `#[server]` / `#[submit]` handler (length, format, authz)
- [ ] Run container as non-root (see docs-site `Dockerfile`)
- [ ] Keep secrets in env / secret manager — never commit `.env`
- [ ] Deploy: see [cookbook/docker](/docs/cookbook/docker) on the docs site (Fly.io + Dockerfile)

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
