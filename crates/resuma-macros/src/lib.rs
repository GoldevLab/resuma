//! Resuma procedural macros.
//!
//! These macros are the surface area Resuma exposes to user code:
//!
//!   * [`view!`]        — JSX-like template syntax that builds a `View` tree.
//!   * [`#[component]`] — turns a function into a Resuma component.
//!   * [`#[server]`]    — exposes an async fn as a server action / RPC.
//!   * [`#[island]`]    — marks an interactive island (its handlers ship to JS).
//!   * [`js!`]          — escape hatch for raw JavaScript handler bodies.
//!
//! The interesting one is `view!`: it walks the JSX-ish tokens, recognises
//! `onClick={...}` style attributes, and feeds the closure body to
//! `resuma-rs2js` to produce a JS chunk. The Rust side only stores a
//! `HandlerRef` pointing at that chunk so SSR can emit `data-r-on:click=…`.

mod view_macro;
mod component_macro;
mod server_macro;
mod island_macro;
mod js_macro;

use proc_macro::TokenStream;

/// `view!` — JSX-like template macro.
///
/// ```ignore
/// view! {
///     <div class="card">
///         <h1>"Hello " {name}</h1>
///         <button onClick={move |_| count.update(|c| *c += 1)}>"+"</button>
///     </div>
/// }
/// ```
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    view_macro::expand(input.into()).into()
}

/// `#[component]` — registers a function as a Resuma component.
///
/// ```ignore
/// #[component]
/// fn Greeting(name: String) -> View {
///     view! { <h1>"Hello "{name}</h1> }
/// }
/// ```
#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    component_macro::expand(args.into(), input.into()).into()
}

/// `#[server]` — exposes an async fn as a server action callable from
/// `actions::name(...)` in the browser.
#[proc_macro_attribute]
pub fn server(args: TokenStream, input: TokenStream) -> TokenStream {
    server_macro::expand(args.into(), input.into()).into()
}

/// `#[island]` — marks a component as an interactive island. Its event
/// handlers are extracted and shipped to the client as a single chunk.
#[proc_macro_attribute]
pub fn island(args: TokenStream, input: TokenStream) -> TokenStream {
    island_macro::expand(args.into(), input.into()).into()
}

/// `js!` — raw JavaScript escape hatch for event handlers.
///
/// ```ignore
/// onClick={ js! { state.count.set(state.count.value + 1); } }
/// ```
#[proc_macro]
pub fn js(input: TokenStream) -> TokenStream {
    js_macro::expand(input.into()).into()
}
