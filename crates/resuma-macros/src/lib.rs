//! Resuma procedural macros.
//!
//! These macros are the surface area Resuma exposes to user code:
//!
//!   * [`view!`]        — JSX-like template syntax that builds a `View` tree.
//!   * [`#[component]`] — turns a function into a Resuma component.
//!   * [`#[server]`]    — exposes an async fn as a server action / RPC.
//!   * [`#[island]`]    — marks an interactive island (its handlers ship to JS).
//!   * [`js!`]          — escape hatch for raw JavaScript handler bodies.

mod component_macro;
mod computed_macro;
mod debounce_macro;
mod effect_macro;
mod island_macro;
mod js_macro;
mod layout_macro;
mod load_macro;
mod middleware_macro;
mod rs2js;
mod server_macro;
mod submit_macro;
mod view_macro;

use proc_macro::TokenStream;

/// `view!` — JSX-like template macro.
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    view_macro::expand(input.into()).into()
}

/// `#[component]` — registers a function as a Resuma component.
#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    component_macro::expand(args.into(), input.into()).into()
}

/// `#[server]` — exposes an async fn as a server action.
#[proc_macro_attribute]
pub fn server(args: TokenStream, input: TokenStream) -> TokenStream {
    server_macro::expand(args.into(), input.into()).into()
}

/// `#[island]` — marks a component as an interactive island.
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

/// `computed!` / `use_computed!` — client-replayable derived signal (rs2js-translated).
#[proc_macro]
pub fn computed(input: TokenStream) -> TokenStream {
    computed_macro::expand(input.into()).into()
}

/// `effect!([signals…], move || { … })` — client-replayable side effect (rs2js).
#[proc_macro]
pub fn effect(input: TokenStream) -> TokenStream {
    effect_macro::expand(input.into()).into()
}

/// `debounce!` — debounced client reaction to a signal.
///
/// ```ignore
/// debounce!(query, 300, move |q| filter.set(q));
/// ```
#[proc_macro]
pub fn debounce(input: TokenStream) -> TokenStream {
    debounce_macro::expand(input.into()).into()
}
