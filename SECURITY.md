# Security Policy

## Supported versions

| Version | Supported |
| ------- | --------- |
| 1.x     | Yes       |
| 0.3.x   | Best effort |
| < 0.3   | No        |

## Reporting a vulnerability

**Do not open a public GitHub issue** for security vulnerabilities.

Instead:

1. Open a [GitHub Security Advisory](https://github.com/GoldevLab/resuma/security/advisories/new) (preferred), **or**
2. Email the maintainers privately (see GitHub profile contact).

Include:

- Description of the issue and impact
- Steps to reproduce
- Affected versions
- Suggested fix (if any)

We aim to acknowledge reports within **72 hours** and will coordinate disclosure once a fix is available.

## Built-in protections

Resuma ships secure defaults for production apps: CSRF on actions/submits, Origin/Referer checks, security headers (HSTS, CSP with nonces, COOP, CORP), rate limiting, SSR escaping, and middleware that blocks unauthorized requests.

Full details: [`docs/SECURITY.md`](docs/SECURITY.md) · [docs site /security](https://resuma-docs.fly.dev/docs/security)

## Deployment checklist

- Set `RESUMA_ENV=production` and `RESUMA_TRUST_PROXY=1` behind a reverse proxy
- Force HTTPS at the edge
- Add auth `#[middleware]` for protected routes
- Validate input in every `#[server]` / `#[submit]` handler
- Keep secrets in environment variables — never commit `.env`
