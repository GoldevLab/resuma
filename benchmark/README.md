# Resuma vs Qwik — bundle benchmark

Compare **transfer size** (Network tab) for the same UX: SSR page with one interactive counter.

## Quick measure (Resuma)

```bash
cd runtime
npm run build
npm run size
```

## Live sizes from server

```bash
curl -s http://127.0.0.1:3000/_resuma/benchmark.json
```

With compression negotiation:

```bash
curl -s -H "Accept-Encoding: gzip" -I http://127.0.0.1:3000/_resuma/loader.js
curl -s -H "Accept-Encoding: br" -I http://127.0.0.1:3000/_resuma/core.js
```

## Run the apps

| Stack | Command | Port |
|-------|---------|------|
| Resuma counter | `cargo run -p example-counter` | 3000 |
| Resuma docs (static) | `cargo run -p example-website` | 3000 |
| Qwik | Use [qwik.dev](https://qwik.dev) starter or your existing Qwik app |

## What to compare

1. **Static landing** — Resuma docs `/` should load **0 JS**.
2. **Counter page** — compare first load + first click:
   - Resuma: `loader.js` immediately; `core.js` on first click (or eagerly if reactive bindings exist).
   - Qwik: `qwikloader` + preloaded chunks per project config.

## Metrics (same column in DevTools)

| Metric | Meaning |
|--------|---------|
| **Raw** | Uncompressed file size (`Content-Length` without encoding) |
| **Transfer** | Bytes on the wire (`Content-Encoding: gzip` or `br`) |

Always compare transfer size with compression enabled — that is what browsers use in production.

## Reference numbers (May 2026)

### Resuma split runtime

| File | Raw | Gzip | Brotli |
|------|-----|------|--------|
| `loader.js` | 1.8 KiB | 884 B | 730 B |
| `core.js` | 6.6 KiB | 2.6 KiB | 2.3 KiB |
| Static page | 0 | 0 | 0 |

### Qwik (published reference)

| File | Raw | Gzip | Brotli |
|------|-----|------|--------|
| `qwikloader` | ~1 KiB | ~2.4 KiB | ~1.4 KiB |

Sources: [Qwikloader docs](https://qwik.dev/docs/advanced/qwikloader/), [Qwik PR #7519](https://github.com/QwikDev/qwik/pull/7519).

## Updating this benchmark

1. Rebuild runtime: `cd runtime && npm run build`
2. Copy assets to `crates/resuma-server/assets/` (loader.js, core.js, runtime.js)
3. Re-run `npm run size` and update `examples/website/src/pages/docs/benchmark.rs`
