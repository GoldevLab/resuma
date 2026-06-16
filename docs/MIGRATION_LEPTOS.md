# Migrating from Leptos to Resuma

Resuma and Leptos both target Rust full-stack UI, but Resuma optimizes for **resumability** (no hydration) rather than fine-grained WASM reactivity on the client.

## Mental model

| Leptos | Resuma |
|--------|--------|
| `view!` + VDOM diff | `view!` → static `View` tree, SSR once |
| `Signal::get()` in templates | `{signal}` or `<Show when={signal}>` |
| `create_resource` / `Resource` | `#[load]` + `use_load()` (Flow) |
| `ServerAction` | `#[server]` → `__resuma.action(...)` |
| `Suspense` / `<ErrorBoundary>` | `load_boundary` / `error_boundary` |
| Client WASM bundle | ~1 KiB loader + lazy handler chunks |

## Components

```rust
// Leptos
#[component]
fn Counter() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    view! { <button on:click=move |_| set_count.update(|n| *n + 1)>{count}</button> }
}

// Resuma
#[component]
fn Counter() -> View {
    let count = use_signal(0);
    view! {
        <button onClick={ move |_| count.update(|n| *n + 1) }>{count}</button>
    }
}
```

Use `js! { ... }` when you need async client logic (fetch, DOM APIs).

## Routing & data

Leptos Router + resources map to **Resuma Flow**:

- File-based pages under `src/pages/`
- `#[load]` for server data (like a resource loader)
- `#[submit]` for forms (like server actions with PRG)
- `FlowRequest` or typed `Path<T>` / `Query<T>` extractors

## Lists & conditionals

| Leptos | Resuma |
|--------|--------|
| `{move || list.iter().map(...)}` | `<For each={list} key="id" let:item>` (reactive when `each` is a signal) |
| `<Show when=...>` | `<Show when={signal}>` |
| `<Match>` / `<Switch>` | `<Match value={signal}><When is={"a"}>...</When></Match>` |

## When to stay on Leptos

- You need **WASM in the browser** for heavy client computation
- Your team already ships Leptos + hydration patterns
- You rely on Leptos ecosystem crates tied to WASM

## When Resuma fits better

- You want **minimal JS** until interaction (marketing + dashboards)
- SEO-first SSR with islands for interactive pockets
- Axum-native deploys without a separate WASM build pipeline

See [examples/todo](examples/todo) and [examples/e2e](examples/e2e) for side-by-side patterns.
