# Resuma bundle benchmark

Compare **Resuma vs Qwik vs Leptos vs Astro vs Next.js vs SvelteKit vs SolidStart vs React (Vite) vs templ + HTMX** on the same UX: SSR counter page (heading + increment button).

## Run all benchmarks

```bash
node benchmark/run.mjs
```

Writes `benchmark/results.json` and prints a summary table.

### Prerequisites

- Node.js (all JS framework builds + Resuma runtime)
- Rust + `wasm32-unknown-unknown` + `wasm-pack` (Leptos counter)
- Go + `templ` (optional — HTMX-only measurement works without Go)

First Leptos run compiles ~200 crates and can take several minutes. Subsequent runs are fast.

## Latest results (gzip transfer sizes)

| Framework | Initial load | First interaction | Static page |
|-----------|-------------:|--------------------:|------------:|
| **Resuma** | 901 B | 4.20 KiB | **0 B** |
| **Qwik** | 1.96 KiB | 22.32 KiB | — |
| **templ + HTMX** | 16.21 KiB | 16.21 KiB | — |
| **SolidStart** | 16.75 KiB | 16.75 KiB | — |
| **SvelteKit** | 27.71 KiB | 27.71 KiB | — |
| **Astro** (React island) | 57.76 KiB | 57.76 KiB | — |
| **React** (Vite SPA) | 57.99 KiB | 57.99 KiB | — |
| **Leptos** | 79.02 KiB | 79.02 KiB | — |
| **Next.js** (App Router) | 142.43 KiB | 142.43 KiB | — |

Measured from production build artifacts in `benchmark/` (May 2026).

## Methodology

1. Same UX: SSR heading + one interactive counter button.
2. Compare **minified** artifact sizes with **gzip** and **brotli** simulated in `run.mjs`.
3. **Initial load** — JS required before the page can resume/hydrate interactivity.
4. **First interaction** — total JS transferred when the user clicks `+` (includes lazy chunks).

### What each framework ships

| | Resuma | Qwik | templ + HTMX | SolidStart | SvelteKit | Astro | React | Leptos | Next.js |
|---|---|---|---|---|---|---|---|---|---|
| Initial | `loader.js` | preloader | `htmx.min.js` | client chunks | entry + runtime | React island + client | SPA bundle | WASM + glue | firstLoadChunkPaths |
| First click | loader + core | preloader + core + route + chunk | same (server RT) | same | same | same | same | same | same |
| Static pages | 0 B | — | — | — | — | — | — | — | — |

## Reproduce individually

```bash
# Resuma runtime bundles
cd runtime && npm run build && npm run size

# Live JSON from a running server
curl -s http://127.0.0.1:3000/_resuma/benchmark.json

# Counter apps (benchmark/*-counter)
cd benchmark/qwik-counter && npm run build
cd benchmark/leptos-counter && wasm-pack build --target web --release
cd benchmark/astro-counter && npm run build
cd benchmark/next-counter && npm run build
cd benchmark/sveltekit-counter && npm run build
cd benchmark/solidstart-counter && npm run build
cd benchmark/react-counter && npm run build
```

## Takeaways

- **Resuma** keeps the initial payload under 1 KiB gzip; full interactivity is ~4 KiB gzip with no WASM.
- **Qwik** preloader is small (~2 KiB gzip) but the core chunk adds ~20 KiB gzip on first interaction.
- **templ + HTMX** ships ~16 KiB gzip (HTMX only); clicks round-trip to the server instead of hydrating client-side.
- **SolidStart / SvelteKit** hydrate on load (~17–28 KiB gzip for a minimal counter).
- **Astro / React** ~58 KiB gzip — React runtime cost is similar whether island or SPA.
- **Leptos** ships the framework runtime as WASM (~73 KiB gzip) plus glue (~6 KiB gzip).
- **Next.js** ~142 KiB gzip first-load JS on default App Router scaffold.
- **Static-first:** only Resuma skips all client JS on pages with no interactivity.
