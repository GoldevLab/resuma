# Resuma product naming

Official names for layers, crates, and docs — modeled after **Qwik / Qwik City**, **Solid / SolidStart**, and **Svelte / SvelteKit**.

## Brand hierarchy

| Order | Brand name | Crate / path | What it is |
|------:|------------|--------------|------------|
| 1 | **Resuma** | `resuma` | Core framework — signals, `view!`, resumability, SSR, server |
| 2 | **Resuma Flow** | `resuma::flow` + `resuma-flow` | Full-stack layer in `resuma`; UI widgets crate (`flow_graph`, dashboard, `worker_panel`) |
| 3 | **Resuma Macros** | `resuma-macros` | Compile-time macros — `view!`, `#[component]`, rs2js |
| 4 | **Resuma Runtime** | `runtime/` → `/_resuma/*.js` | Browser resumability (loader + core + flow) |
| 5 | **Resuma Client** | `client-sdk/resuma-client.ts` | TypeScript mount contract for prebuilt widgets |
| 6 | **Resuma CLI** | `resuma` feature `cli` | `resuma new`, `dev`, `build`, `update` |

## Analogies

| Ecosystem | Core | Full-stack / meta |
|-----------|------|-------------------|
| Qwik | Qwik | Qwik City |
| Solid | Solid | SolidStart |
| Svelte | Svelte | SvelteKit |
| **Resuma** | **Resuma** | **Resuma Flow** |

## Rust module map

```
resuma/                         ← depend on this (runtime crate)
├── core/                       ← Resuma (signals, View, resumability)
├── ssr/                        ← Resuma (HTML + payload)
├── server/                     ← Resuma (axum, /_resuma/*)
├── flow/                       ← Resuma Flow (FlowApp, pages, loads)
├── client/                     ← Resuma Client (ClientComponent API)
└── cli/                        ← Resuma CLI (feature flag)

resuma-macros/                  ← Resuma Macros (proc-macro crate, required at build)
resuma-flow/                    ← Flow UI widgets (depends on resuma; optional)
runtime/                        ← Resuma Runtime (TypeScript, embedded in resuma assets)
client-sdk/                     ← Resuma Client (copy into app client/ dir)
```

## Naming rules

1. **“Resuma” alone** = core resumability (components, signals, runtime). Not “Resuma Core” in user-facing copy unless contrasting with Flow.
2. **“Resuma Flow”** = full-stack layer. Always two words when referring to the product. Code: `FlowApp`, `resuma::flow`; optional widgets crate: `resuma-flow`.
3. **“Resuma Macros”** = the macro crate. Code: `resuma-macros`. Never shorten to “macros crate” in docs titles.
4. **“Resuma Runtime”** = browser loader/core bundles. Paths: `/_resuma/loader.js`, `/_resuma/core.js`.
5. **“Resuma Client”** = optional TypeScript widgets (`ClientComponent`, `bootClientComponent`). Not the same as Runtime.
6. **Avoid** mixing “Flow” without “Resuma” in headings (e.g. prefer “Resuma Flow routing” over “Flow routing” alone).

## Public API names (stable)

| Concept | Type / fn | Layer |
|---------|-----------|-------|
| App (simple) | `ResumaApp` | Resuma |
| App (full-stack) | `FlowApp` | Resuma Flow |
| Page registry | `FlowPageRegistry` | Resuma Flow |
| Component | `#[component]` | Resuma |
| Template | `view!` | Resuma Macros |
| Server RPC | `#[server]` | Resuma |
| Data loader | `#[load]` | Resuma Flow |
| Form action | `#[submit]` | Resuma Flow |
| TS widget | `ClientComponent`, `client_component()` | Resuma Client |
| TS bundle route | `FlowApp::client_asset()` | Resuma Flow + Client |

## Published crates (crates.io)

| Crate | Role |
|-------|------|
| `resuma` | Runtime — **only required dependency** for most apps |
| `resuma-macros` | Proc-macros — pulled in automatically via `resuma` |
| `resuma-flow` | Flow UI widgets (`flow_graph`, dashboard, `worker_panel`) — published alongside `resuma` |

Flow **routing / `FlowApp` / `#[load]` / `#[submit]`** live in `resuma::flow`. The separate **`resuma-flow`** crate (`crates/resuma-flow/`) is the optional Flow UI widgets package, published with the same workspace version.

## Docs-site URLs

| Topic | Path |
|-------|------|
| Package overview | `/docs/package` |
| Resuma Flow | `/docs/flow` |
| Client (TypeScript) | `/docs/components/client` |
| Architecture | `/docs/architecture` |
