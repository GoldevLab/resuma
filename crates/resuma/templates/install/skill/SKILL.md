---
name: resuma
description: >-
  Build Resuma Rust SSR apps — view!, signals, Flow, #[server], #[load], #[submit].
  Use when writing or debugging Resuma code, scaffolding pages, or fixing client
  reactivity (resuma-dyn, resuma-show, handler chunks).
---

# Resuma framework skill

Resuma is a **resumable SSR** Rust web framework (like Qwik, but Rust-native). Components run **once on the server**; the client resumes signals and lazy handler JS — no WASM hydration by default.

## When to use this skill

- Creating or editing `view!`, `#[component]`, `FlowApp`, `ResumaApp`
- File-based routing under `src/pages/`
- Server actions (`#[server]`), form submits (`#[submit]`), loaders (`#[load]`)
- Debugging interactivity (clicks, signals, Show, islands)
- SEO/GEO (`SeoKit`, robots.txt, llms.txt)

## Critical reactivity rules

### Interpolate signals without `.get()` in `view!`

```rust
// ✅ Client updates
view! { <p>{count}</p> }

// ❌ SSR snapshot only — UI frozen after click
view! { <p>{count.get()}</p> }
```

### Reactive `<Show>`

```rust
let logged_in = signal(false);
view! {
    <Show when={logged_in.get()}>
        <p>"Welcome!"</p>
    </Show>
    <Show when={!logged_in.get()} fallback={view! { <span>"Sign in"</span> }}>
        <span></span>
    </Show>
}
```

Also works with `when={logged_in}` (signal reference). Do **not** use `{if signal.get() { ... }}` for UI that must update on the client — use `<Show>` or a string signal `{label}`.

### Inputs

Prefer `onInput` with `js!` or signal binding — avoid `value={signal.get()}` (one-way SSR snapshot).

```rust
<input onInput={js! { state.q.set(event.target.value); }} />
```

## Project patterns

### Minimal app

```rust
use resuma::prelude::*;

#[component]
fn App() {
    let n = signal(0);
    view! {
        <button onClick={n.update(|v| *v += 1)}>{n}</button>
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    ResumaApp::new()
        .component("/", App)
        .serve(ServeOptions::default())
        .await
}
```

### Flow (file-based pages)

```rust
FlowApp::new()
    .with_seo_kit(SeoKit::new("My App", "https://example.com").with_llms_summary("…"))
    .auto_pages(path_to_pages, PagesRegistry)
    .serve(FlowServeOptions::default())
    .await
```

Generate route registry:

```bash
resuma routes --generate --path src/pages
```

### Server action from UI

```rust
#[server]
async fn echo(msg: String) -> String {
    format!("Echo: {msg}")
}

// In view! — use js! for async actions:
<button onClick={js! {
    const r = await __resuma.action("echo", ["hi"]);
    state.result.set(r);
}}>"Call"</button>
<p>{result}</p>
```

## CLI commands

| Command | Purpose |
|---------|---------|
| `resuma new` | Scaffold project (basic, todo, flow, …) |
| `resuma dev` | Hot reload dev server |
| `resuma routes --generate` | Regenerate `src/pages/_registry.rs` |
| `resuma add sqlx` / `turso` | DB integration scaffold |
| `resuma install skill` | Install this skill for Cursor/agents |
| `resuma doctor` | Toolchain + project health |

## SEO / GEO

```rust
let kit = SeoKit::new("Site", "https://example.com")
    .with_meta_pixel("PIXEL_ID")
    .with_default_json_ld()
    .with_llms_summary("What this app does for humans and LLMs.");

FlowApp::new().with_seo_kit(kit) // auto-serves /robots.txt and /llms.txt
```

## Debugging checklist

1. **Click does nothing** — wait for `__resumaCoreReady`; ensure handler chunk loaded; check race before core preload.
2. **Text frozen** — replace `{x.get()}` with `{x}` in view!.
3. **Show stuck** — use signal in `when={}`, not a plain bool variable evaluated once.
4. **Form submit** — `data-r-submit` + CSRF; or `<Form submit={handler}>`.
5. **Dynamic route 404** — run `resuma routes --generate`; check `_registry.rs`.

## Docs

- https://resuma-docs.fly.dev/docs
- https://docs.rs/resuma
