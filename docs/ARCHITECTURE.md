# Resuma Architecture

This document explains *how* Resuma turns a Rust component into an instantly-interactive HTML page without ever shipping a hydration step. The high-level model is **resumability**: serialize server state into HTML and resume interactivity on demand — implemented end-to-end in Rust.

## 1. The resumability promise

Most SSR frameworks split work into two phases:

1. **Render** components on the server, emit HTML.
2. **Hydrate** by re-running the same components on the client to attach event listeners.

Resumability collapses phase 2. The server emits HTML *plus* a serialized blob describing every signal, handler reference and island on the page. The client *never* re-runs a component. It only resumes interactions that the user actually triggers.

```
┌────────────────────┐        HTML + payload        ┌────────────────────┐
│ Server (Rust)      │ ───────────────────────────► │ Browser (~3KB JS)  │
│ render components  │                              │ parse payload      │
│ serialize state    │                              │ delegate events    │
└────────────────────┘                              │ lazy import handler│
                                                    └────────────────────┘
```

## 2. The pipeline of one click

Take a single counter button:

```rust
view! {
    <button onClick={ move |_| count.update(|c| *c += 1) }>"+"</button>
}
```

### 2.1 Macro expansion

The `view!` macro tokenizes the JSX-ish input and walks it. When it sees `onClick={...}`, it:

1. Parses the closure with `syn`.
2. Hands the body to `rs2js::translate_handler(...)` (inside `resuma-macros`).
3. Receives back a `{ js: String, captures: BTreeSet<String>, ... }` translation.
4. Generates Rust code that registers the chunk + symbol with the active `RenderContext`, returning an `AttrValue::Handler(HandlerRef { … })`.

The Rust expansion looks roughly like:

```rust
View::element("button")
    .attr_runtime((
        "onClick".into(),
        register_handler(
            "click",
            "__page__",
            "h_8d3a9c…",
            "(_) => { state.count.update((c) => (c + 1)) }",
            vec![ResumeCapture::Signal(count.id())],
            vec![],
        ),
    ))
    .child(Child::Text("+".into()))
    .build()
```

### 2.2 SSR rendering

`resuma::ssr` walks the resulting `View` tree.

```html
<button data-r-on:click="__page__#h_8d3a9c…"
        data-r-cap:click="s1"
        data-r-inline:click="(_)=>{state.count.update((c)=>(c+1))}">+</button>
```

At the bottom of the page, the renderer also emits the resumability payload:

```html
<script type="resuma/state" id="resuma-state">
{"signals":[{"id":{"0":1},"value":0}],"handlers":{"__page__":{"h_8d3a…":"(_)=>{...}"}},"islands":[],"actions":[]}
</script>
<script type="module" src="/_resuma/runtime.js"></script>
```

### 2.3 The runtime resumes interactions

When the user clicks the button:

1. The runtime's document-level click delegator finds the closest ancestor with `data-r-on:click`.
2. It loads the inline JS source from `data-r-inline:click` and compiles it once via `new Function(...)`.
3. It calls the function with `(event, state, __resuma)` where `state` exposes `state.count` as a `SignalCell`.
4. `state.count.update((c) => c + 1)` triggers any subscribers, including the reactive `<resuma-dyn>` text node bound to `count`, which updates the visible count.

Total round-trip on first interaction: zero network requests for the inline path, **one** dynamic `import()` for non-inline chunks. Every subsequent interaction is essentially free.

## 3. Server actions

`#[server] async fn search(q: String) -> Vec<String>` expands into:

* The original `async fn` (kept on the server).
* A typed JSON dispatcher `__resuma_action_dispatch_search(args)` that deserialises arguments via `serde_json` and calls `search`.
* A `#[ctor::ctor]` initializer that registers the dispatcher in the global action registry.

In templates, calling `actions::search(q).await` is detected by rs2js and translated to:

```js
(await __resuma.action('search', [q]))
```

The runtime POSTs `/_resuma/action/search` with `{ "args": [q] }` and returns the JSON body.

## 4. Islands

An island is a component whose JavaScript travels independently from the rest of the page. The `#[island]` macro wraps the rendered `View` inside an `IslandView` node carrying:

* `chunk_id` — file-system stable identifier of the JS chunk.
* `instance_id` — per-instance suffix so multiple `<Counter />` on the page are addressable.
* `signal_ids` — signals that belong to this island; the runtime hands them to `resume(props, signals, root)`.
* `props` — JSON-encoded props passed to the island's resume entry.

The runtime walks `<resuma-island>` elements after bootstrap, dynamically imports `/_resuma/island/<chunk>.js`, and calls `resume(...)` if exported.

## 5. Reactivity

`resuma::core` mirrors a fine-grained reactivity model on the server:

* `Signal<T>` allocates a stable id from the active `RenderContext`.
* `use_effect` subscribes to signals captured during execution.
* `use_computed` returns a derived signal recomputed when dependencies change.

The client runtime mirrors this model in ~80 lines of TypeScript. There is **no virtual DOM diffing** — updates are surgical, driven directly by signal subscriptions.

## 6. Friendly JS bridge

Rust is great at correctness. JavaScript is great at the DOM. Resuma's `js!{}` macro lets you drop down to raw JS for cases the `rs2js` subset cannot express:

```rust
onInput={ js! {
    if (event.key === 'Enter') {
        state.query.set(event.target.value);
        await __resuma.action('search', [event.target.value]);
    }
}}
```

Inside `js!{}` you have:

* `event` — the DOM event.
* `state` — every captured signal as a reactive cell.
* `__resuma` — the runtime: `__resuma.action(...)`, `__resuma.signals`, `__resuma.refreshIsland(...)`.

## 7. Resumability vs hydration

| Concern | Classic SSR + hydration | Resuma |
| --- | --- | --- |
| Initial bundle | Framework runtime + app code up front | ~3KB loader; core and handlers on demand |
| Time to interactive | Wait for client bundle + hydration pass | Static HTML is interactive-ready; handlers load on first use |
| Server mutations | Custom API routes | First-class `#[server]` + `POST /_resuma/action/:name` |
| Partial interactivity | Split bundles manually | First-class `#[island]` boundary |
| Escape hatches | Framework-specific | `js!{}` with `state` + `__resuma.action(...)` bridge |
| Tooling | Separate build pipelines | `resuma` CLI: `new`, `dev`, `build`, `routes` |

## 8. File-based routing

`resuma::router::discover(path)` walks `src/pages/` and turns:

```
src/pages/index.rs            -> /
src/pages/about.rs            -> /about
src/pages/users/[id].rs       -> /users/:id
src/pages/blog/[...slug].rs   -> /blog/*slug
src/pages/layout.rs           -> shared layout
```

into a `Vec<DiscoveredRoute>` that the CLI can dump (`resuma routes`) or feed into a generated `app.register_routes()` call.

## 9. Roadmap

See the README for a list of planned features. The core architecture above is stable; everything else builds on top of it.
