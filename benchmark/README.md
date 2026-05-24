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
| **Leptos** | 79.02 KiB | 79.02 KiB | — |
| **Next.js** (App Router) | 142.43 KiB | 142.43 KiB | — |
| **React** (Vite SPA) | 57.99 KiB | 57.99 KiB | — |
| **Astro** (React island) | 57.76 KiB | 57.76 KiB | — |
| **SvelteKit** | 27.71 KiB | 27.71 KiB | — |
| **Qwik** | 1.96 KiB | 22.32 KiB | — |
| **SolidStart** | 16.75 KiB | 16.75 KiB | — |
| **templ + HTMX** | 16.21 KiB | 16.21 KiB | — |

Measured from production build artifacts in `benchmark/` (May 2026).

### Reading the table

- **Hydration frameworks** (SvelteKit, SolidStart, Astro, React, Leptos, Next.js): initial load = first interaction — all client JS arrives on first paint.
- **Resumability** (Resuma, Qwik): initial load is tiny; first interaction adds lazy runtime chunks.
- **Next.js 142 KiB** uses the default `create-next-app` scaffold (Tailwind, fonts, Turbopack). Optimized App Router apps often land at **67–78 KiB** first-load JS.

## External validation

Independent sources align with our numbers (same ranking, same order of magnitude):

| Framework | Ours | Published | Verdict |
|-----------|-----:|----------:|---------|
| Leptos | 79.02 KiB | [WASM binary size docs](https://book.leptos.dev/deployment/binary_size.html) — few public minimal benchmarks | Plausible |
| Next.js 16 | 142.43 KiB | [67 kB optimized App Router](https://markaicode.com/vs/stop-choosing-wrong-nextjs-15-app-router-vs-pages-router-performance-reality-check/) vs default scaffold | High (scaffold) |
| React 19 Vite | 57.99 KiB | [~59 kB Vite scaffold](https://github.com/facebook/react/issues/29913), [49 kB React 18](https://dev.to/sendotltd/solidjs-port-gzip-833-kb-react-83-because-fine-grained-reactivity-means-no-virtual-dom-353) | Matches |
| Astro + React | 57.76 KiB | [58.86 kB client.js](https://github.com/withastro/astro/issues/13378) | Matches |
| SvelteKit | 27.71 KiB | [32.50 kB SendOT portfolio](https://dev.to/sendotltd/sveltekit-port-3250-kb-gzip-72-over-plain-svelte-meta-framework-tax-round-two-288c) | Close |
| Qwik | 1.96 / 22.32 KiB | [preloader ~2 KiB](https://github.com/QwikDev/qwik/pull/7519), [core ~20–24 KiB](https://dev.to/sendotltd/qwik-city-port-two-bundle-numbers-2860-kb-first-paint-4492-kb-total-because-resumability-4a8i) | Matches |
| SolidStart | 16.75 KiB | [Solid SPA 8.33 KB](https://dev.to/sendotltd/solidjs-port-gzip-833-kb-react-83-because-fine-grained-reactivity-means-no-virtual-dom-353) + meta-framework | Reasonable |
| templ + HTMX | 16.21 KiB | [HTMX ~16 KB gzip](https://github.com/bigskysoftware/htmx/issues/3239) | Matches |

See also the [SendOT portfolio series](https://dev.to/sendotltd/qwik-city-port-two-bundle-numbers-2860-kb-first-paint-4492-kb-total-because-resumability-4a8i) — same UX, production builds, gzip, across React/Vue/Svelte/Solid/Nuxt/SvelteKit/Qwik.

## Methodology

1. Same UX: SSR heading + one interactive counter button.
2. Compare **minified** artifact sizes with **gzip** and **brotli** simulated in `run.mjs`.
3. **Initial load** — JS required before the page can resume/hydrate interactivity.
4. **First interaction** — total JS transferred when the user clicks `+` (includes lazy chunks).

### What each framework ships

| | Resuma | Leptos | Next.js | React | Astro | SvelteKit | Qwik | SolidStart | templ + HTMX |
|---|---|---|---|---|---|---|---|---|---|
| Initial | `loader.js` | WASM + glue | firstLoadChunkPaths | SPA bundle | React island + client | entry + runtime | preloader | client chunks | `htmx.min.js` |
| First click | loader + core | same | same | same | same | same | preloader + core + route + chunk | same | same (server RT) |
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
