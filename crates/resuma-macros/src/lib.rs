//! Resuma procedural macros.
//!
//! Re-exported by the [`resuma`](https://docs.rs/resuma) crate. Typical surface:
//!
//! | Macro | Role |
//! |-------|------|
//! | [`view!`] | JSX-like templates → [`View`](https://docs.rs/resuma/latest/resuma/enum.View.html) |
//! | [`#[component]`](component) | Resumable component + props builder (lazy handler boundary) |
//! | [`#[server]`](server) | Async RPC at `POST /_resuma/action/:name` |
//! | [`#[data]`](data) | Concise typed DTO/model helper |
//! | [`computed!`](computed) | Client-replayable derived signal (rs2js) |
//! | [`effect!`](effect) | Client-replayable side effect (rs2js) |
//! | [`debounce!`](debounce) | Debounced client reaction |
//! | [`#[island]`](island) | Optional heavy lazy boundary (`load = "visible"`) |
//! | [`js!`](js) | Raw JavaScript handler escape hatch |

mod component_macro;
mod computed_macro;
mod data_macro;
mod debounce_macro;
mod effect_macro;
mod extract_codegen;
mod island_macro;
mod js_macro;
mod layout_macro;
mod load_macro;
mod middleware_macro;
mod rs2js;
mod server_macro;
mod store_macro;
mod submit_macro;
mod view_macro;

use proc_macro::TokenStream;

/// `view!` — JSX-like template macro.
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    view_macro::expand(input.into()).into()
}

/// `#[component]` — resumable component with generated props builder.
///
/// Wraps output in a lazy handler boundary; event handlers register under
/// `/_resuma/handler/{ComponentName}.js`. For heavy optional lazy bundles, see [`island`].
#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    component_macro::expand(args.into(), input.into()).into()
}

/// `#[server]` — exposes an async fn as a server action.
#[proc_macro_attribute]
pub fn server(args: TokenStream, input: TokenStream) -> TokenStream {
    server_macro::expand(args.into(), input.into()).into()
}

/// `#[data]` — concise DTO/model helper for Resuma data structs and enums.
#[proc_macro_attribute]
pub fn data(args: TokenStream, input: TokenStream) -> TokenStream {
    data_macro::expand(args.into(), input.into()).into()
}

/// `#[island]` — optional interactive boundary for heavy or visibility-gated JS.
///
/// Most UI only needs [`component`]. Use islands for large client bundles,
/// `#[island(load = "visible")]`, or dev HMR refresh via `GET /_resuma/island/:instance`.
#[proc_macro_attribute]
pub fn island(args: TokenStream, input: TokenStream) -> TokenStream {
    island_macro::expand(args.into(), input.into()).into()
}

/// `#[load]` — Resuma Flow server data loader.
#[proc_macro_attribute]
pub fn load(args: TokenStream, input: TokenStream) -> TokenStream {
    load_macro::expand(args.into(), input.into()).into()
}

/// `#[submit]` — Resuma Flow form submission handler.
#[proc_macro_attribute]
pub fn submit(args: TokenStream, input: TokenStream) -> TokenStream {
    submit_macro::expand(args.into(), input.into()).into()
}

/// `#[layout]` — Resuma Flow layout wrapper.
#[proc_macro_attribute]
pub fn layout(args: TokenStream, input: TokenStream) -> TokenStream {
    layout_macro::expand(args.into(), input.into()).into()
}

/// `#[middleware]` — Resuma Flow request middleware.
#[proc_macro_attribute]
pub fn middleware(args: TokenStream, input: TokenStream) -> TokenStream {
    middleware_macro::expand(args.into(), input.into()).into()
}

/// `js!` — raw JavaScript escape hatch for event handlers.
#[proc_macro]
pub fn js(input: TokenStream) -> TokenStream {
    js_macro::expand(input.into()).into()
}

/// `computed!([deps…], move || …)` — derived signal with client replay (rs2js-translated).
///
/// Runs during SSR and replays in the browser when dependencies change.
/// For SSR-only derived state, use [`use_computed`](https://docs.rs/resuma/latest/resuma/fn.use_computed.html) instead.
#[proc_macro]
pub fn computed(input: TokenStream) -> TokenStream {
    computed_macro::expand(input.into()).into()
}

/// `effect!([signals…], move || { … })` — client-replayable side effect (rs2js).
#[proc_macro]
pub fn effect(input: TokenStream) -> TokenStream {
    effect_macro::expand(input.into()).into()
}

#[proc_macro_derive(Store)]
pub fn derive_store(input: TokenStream) -> TokenStream {
    store_macro::derive(input)
}

/// `debounce!([deps…], ms, move || …)` — debounced client reaction (rs2js-translated).
///
/// ```ignore
/// debounce!([search], 300, move || { /* runs after search settles */ });
/// ```
#[proc_macro]
pub fn debounce(input: TokenStream) -> TokenStream {
    debounce_macro::expand(input.into()).into()
}
