# Rust SSR without hydration: how Resuma serializes state into HTML

> Technical post draft — publish on Dev.to, r/rust (Showcase Saturday), or Hacker News (Show HN).
> No third-party framework comparisons; focus on the mechanism.

## Title options

- **Rust SSR without hydration: serializing signals and handlers in HTML**
- **[Showcase] Resuma — resumable SSR for Rust (zero client re-execution)**
- **How I built a ~3KB client runtime that resumes server-rendered Rust UI**

## Hook (first paragraph)

Most SSR stacks ship HTML, then **re-run your UI on the client** to attach event listeners — that's hydration. Resuma takes a different path: components run **once** on the server, and the SSR pass embeds a resumability payload (signals, handler references, islands) in the page. The browser never re-executes your Rust components; a tiny runtime resumes only the interaction the user actually triggers.

## The problem hydration solves (and what resumability changes)

Hydration exists because the server emits inert HTML. The client needs *some* code to wire `onClick`, `onInput`, and reactive bindings. Classic approach: download a bundle, reconstruct the component tree, diff against DOM, attach listeners.

Resumability asks: *what is the minimum the client needs to resume interactivity?*

1. **Signal state** — current values serialized as JSON.
2. **Handler references** — not the full component, just the closure compiled to a lazy-loaded JS chunk.
3. **Island boundaries** — optional interactive regions with their own chunks.

The client runtime (~3KB loader) parses that payload and delegates events at the document level. First click fetches the handler chunk; no full-tree replay.

## Minimal example

```rust
use resuma::prelude::*;

#[component]
fn Counter() {
    let count = signal(0);
    view! {
        <main>
            <h1>"Count: " {count}</h1>
            <button onClick={count.update(|c| *c += 1)}>"+"</button>
        </main>
    }
}
```

During SSR, `view!` expands the button handler through **rs2js** (inside `resuma-macros`) into a `data-r-on:click` attribute pointing at a lazy chunk. The HTML includes:

```html
<script type="resuma/state" id="resuma-state">
{"signals":[{"id":"…","value":0}],"handlers":{"…":"chunk-0"},…}
</script>
<script type="module" src="/_resuma/loader.js"></script>
```

Static pages with no handlers ship **zero** client JS — no loader, no payload.

## Pipeline of one click

1. **Compile time** — `view!` / rs2js translate Rust closures to JS handler modules.
2. **SSR** — walk the `View` tree, emit HTML + resumability JSON.
3. **First interaction** — loader delegates the event, fetches `/_resuma/handler/:chunk`, runs against resumed signals.
4. **Server actions** — `#[server]` registers `POST /_resuma/action/:name` for RPC from handlers or forms.

## Full-stack layer (one crate)

`resuma` also includes **Flow**: file-based pages (`src/pages/`), `#[load]` for server data, `#[submit]` for forms, `#[middleware]`, streaming SSR. One dependency:

```toml
resuma = "0.2"
```

## Try it

```bash
cargo install resuma
resuma new my-app --template todo
cd my-app && resuma dev
```

- **Docs:** https://resuma-docs.fly.dev
- **Crate:** https://crates.io/crates/resuma
- **API:** https://docs.rs/resuma
- **Repo:** https://github.com/GoldevLab/resuma

## What to show in a GIF (10 seconds)

1. Open DevTools → Network tab.
2. Load a static docs page → **0 JS requests**.
3. Open `example-counter` → only `loader.js` (~884 B gzip).
4. Click the button → handler chunk loads, count updates — no full page reload.

## r/rust post template

**Title:** `[Showcase] Resuma v0.2 — resumable SSR for Rust (no hydration)`

**Body:**

I built a Rust web framework where components run only on the server. SSR embeds signal state and handler refs in HTML; a ~3KB client runtime resumes interactivity on demand.

- Static pages: **0 client JS**
- `view!` macro + `#[server]` + `#[island]` + full-stack Flow in one crate
- `cargo install resuma` → `resuma new my-app`

Live docs: https://resuma-docs.fly.dev  
crates.io: https://crates.io/crates/resuma

Happy to answer questions about the resumability payload format or rs2js compilation.

## Show HN template

**Title:** Show HN: Resuma – Rust SSR with resumability instead of hydration

**URL:** https://resuma-docs.fly.dev (or GitHub repo)

**Comment:** Components execute once on the server. HTML carries a serialized resumability payload; the client never re-runs Rust. ~884 B gzip loader on interactive pages, zero JS on static pages. Single crate includes routing, loaders, and server actions. Feedback welcome — v0.2, APIs may evolve.
