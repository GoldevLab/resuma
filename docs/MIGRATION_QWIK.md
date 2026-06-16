# Migrating from Qwik to Resuma

Qwik and Resuma share the same goal: **serialize server state and resume interactivity without hydration**. The vocabulary differs but maps cleanly.

## Concept mapping

| Qwik | Resuma |
|------|--------|
| `$` reactivity | `Signal` / `use_signal` |
| `useSignal` | `use_signal` |
| `useTask$` / `useVisibleTask$` | `use_visible_task` / `use_effect` |
| `component$` | `#[component]` |
| `routeLoader$` | `#[load]` (Flow) |
| `routeAction$` | `#[submit]` or `#[server]` |
| Qwik City pages | Flow `src/pages/` + `FlowApp` |
| `useLocation` | `current_request()` / `current_location_href()` |
| Resumability core | `/_resuma/loader.js` + `core.js` |

## Templates

```tsx
// Qwik
export default component$(() => {
  const count = useSignal(0);
  return <button onClick$={() => count.value++}>{count.value}</button>;
});
```

```rust
// Resuma
#[component]
fn Counter() -> View {
    let count = use_signal(0);
    view! {
        <button onClick={ move |_| count.update(|n| n + 1) }>{count}</button>
    }
}
```

## Loaders & actions

Qwik City `routeLoader$` / `routeAction$` → Resuma Flow:

```rust
#[load]
async fn user(id: Path<u64>) -> User { ... }

#[submit]
async fn save(form: FormData, req: &FlowRequest) -> Result<(), SubmitError> { ... }
```

Typed extractors (`Path`, `Query`) replace manual parsing of `requestEvent`.

## Islands & lazy boundaries

| Qwik | Resuma |
|------|--------|
| Implicit lazy boundaries | `#[component]` handler chunks (default) |
| Heavy client bundles | `#[island]` + `load = "visible"` |
| Prefetch | NavLink hover prefetch + `invalidate_href` |

## Control flow

```rust
<Show when={logged_in}>{ "Dashboard" }</Show>

<Match value={status}>
    <When is={"pending"}>{ "Loading…" }</When>
    <When is={"ok"}>{ "Ready" }</When>
    <Default>{ "Unknown" }</Default>
</Match>

<For each={items} key="id" let:item>
    <li>{ item.title.clone() }</li>
</For>
```

## Runtime size

Both target a tiny first paint bundle. Resuma enforces loader/core gzip budgets in CI (`npm run size`).

## Deployment

- Qwik: Node adapter, static + serverless
- Resuma: single Axum binary (`ResumaApp` / `FlowApp`), see [DEPLOY.md](DEPLOY.md)

## What you give up

- JSX/TSX in `.tsx` files (Rust `view!` instead)
- Qwik's npm ecosystem (`@builder.io/qwik`, Qwik City plugins)
- Automatic `$` serialization of closures (Resuma uses explicit `js!` / rs2js macros)

## What you gain

- One language (Rust) for UI + server
- Native Axum middleware, SQLx, and typed extractors
- No separate Qwik optimizer / Vite plugin pipeline

Start from `resuma new --template flow` or compare [benchmark/README.md](../benchmark/README.md) bundle numbers.
