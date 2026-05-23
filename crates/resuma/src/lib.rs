//! # Resuma
//!
//! The first Rust web framework with **SSR + Resumability + Islands +
//! Server Actions + a friendly JS bridge** — all in one box.
//!
//! ```ignore
//! use resuma::prelude::*;
//!
//! #[component]
//! fn Counter() -> View {
//!     let count = use_signal(0);
//!     view! {
//!         <div>
//!             <h1>"Counter: " {count}</h1>
//!             <button onClick={ move |_| count.update(|c| *c += 1) }>"+"</button>
//!         </div>
//!     }
//! }
//!
//! #[server]
//! async fn greet(name: String) -> String { format!("hello {name}") }
//!
//! #[tokio::main]
//! async fn main() -> std::io::Result<()> {
//!     ResumaApp::new()
//!         .with_title("Counter")
//!         .page("/", || Counter::render(CounterProps::default()))
//!         .serve(ServeOptions::default())
//!         .await
//! }
//! ```
//!
//! ## How it differs from Leptos / Yew / Dioxus
//!
//! * **Resumable, not hydrated.** Components only run on the server. The
//!   client never re-executes the framework — it only resumes interactions.
//! * **Islands by default.** Each `#[island]` ships its own JS chunk and
//!   the rest of the page stays static.
//! * **Server actions.** `#[server] async fn` is callable from event
//!   handlers as `actions::name(args)`.
//! * **Friendly JS bridge.** The `view!` macro understands a small Rust
//!   subset and translates closures to JS via `resuma-rs2js`. For anything
//!   beyond that subset, the `js!{}` escape hatch ships raw JS verbatim.

pub use resuma_core::{
    Signal, ReadSignal, WriteSignal, use_signal,
    Effect, Computed, use_effect, use_computed,
    View, IntoView, Component,
    RenderContext, RenderMode, ResumaError, Result,
};

pub use resuma_macros::{component, server, island, view, js};

pub use resuma_server::{ResumaApp, ServeOptions, register_server_action};

pub use resuma_ssr::{render_to_string, render_view, PageOptions};

pub mod prelude {
    //! Glob-friendly re-exports.
    pub use super::{
        Signal, ReadSignal, WriteSignal, use_signal,
        Effect, Computed, use_effect, use_computed,
        View, IntoView, Component,
        ResumaApp, ServeOptions, PageOptions,
        component, server, island, view, js,
        render_to_string, render_view,
    };
}

#[doc(hidden)]
pub mod __private {
    //! Re-exports used by the macro-generated code.
    //! Stable across patch releases of the same minor version.
    pub use ctor;
    pub use serde_json;
    pub use resuma_core::{
        Signal, ReadSignal, WriteSignal,
        View, IntoView, Component, Child,
        view::{AttrValue, Element, Fragment, Island as IslandView},
        handler::{HandlerRef, HandlerCapture},
        signal::SignalId,
        context::{current_context, RenderContext, RenderMode},
        ResumaError, Result,
    };
    pub use resuma_server::register_server_action;

    /// Source code for an event handler — produced by `view!` macro and the
    /// `js!{}` escape hatch.
    #[derive(Debug, Clone)]
    pub enum HandlerSource {
        Inline(String),
        Chunk { chunk: String, symbol: String, source: String },
    }

    /// Metadata captured by an event handler closure. The `name` field is
    /// the user-visible Rust identifier captured by the closure (e.g.
    /// `count`); `id` is the stable signal id assigned by the SSR renderer.
    #[derive(Debug, Clone)]
    pub enum ResumeCapture {
        Signal { name: String, id: SignalId },
        Action(String),
    }

    pub use resuma_core::view::Element as ElementType;

    /// Bridge between the macro layer and the SSR renderer. Registers the
    /// handler chunk in the active `RenderContext` and returns an
    /// `AttrValue::Handler` ready to be embedded in the View.
    pub fn register_handler(
        event: &str,
        chunk: &str,
        symbol: &str,
        js_source: &str,
        captures: Vec<ResumeCapture>,
        actions: Vec<String>,
    ) -> AttrValue {
        if let Some(ctx) = current_context() {
            ctx.register_handler(chunk, symbol, js_source);
            for a in &actions { ctx.register_action(a); }
        }

        // Strip out actions; only signal captures travel via the
        // `HandlerRef::captures` field. The `inline` JS contains references
        // like `state.count` — we ship `count:<id>` pairs in the captures
        // attribute so the runtime can build a name-keyed `state` proxy.
        let signal_captures: Vec<HandlerCapture> = captures
            .into_iter()
            .filter_map(|c| match c {
                ResumeCapture::Signal { name, id } => Some(HandlerCapture { name, id }),
                _ => None,
            })
            .collect();

        AttrValue::Handler(HandlerRef {
            event: event.to_string(),
            chunk: chunk.to_string(),
            symbol: symbol.to_string(),
            captures: signal_captures,
            inline: Some(js_source.to_string()),
        })
    }

    /// Helper used by the `view!` macro to convert a handler tuple
    /// `(name, AttrValue)` into a builder method invocation.
    pub trait ElementBuilderExt {
        fn attr_runtime(self, kv: (String, AttrValue)) -> Self;
    }

    impl ElementBuilderExt for resuma_core::view::ElementBuilder {
        fn attr_runtime(self, (name, value): (String, AttrValue)) -> Self {
            self.attr(name, value)
        }
    }

    /// `view!` calls this for `<MyComponent prop=v />` invocations.
    pub fn render_component<C: Component>(props: C::Props) -> View {
        C::render(props)
    }

    /// Resolve a Rust value into an `AttrValue` so `class={my_string}` etc.
    /// Just Works.
    pub fn resolve_attr_value<T: Into<AttrValueAuto>>(value: T) -> AttrValue {
        value.into().into_attr_value()
    }

    pub struct AttrValueAuto(AttrValue);

    impl AttrValueAuto {
        fn into_attr_value(self) -> AttrValue { self.0 }
    }

    impl From<&str>   for AttrValueAuto { fn from(s: &str)   -> Self { Self(AttrValue::Static(s.to_string())) } }
    impl From<String> for AttrValueAuto { fn from(s: String) -> Self { Self(AttrValue::Static(s)) } }
    impl From<bool>   for AttrValueAuto { fn from(b: bool)   -> Self { Self(AttrValue::Bool(b)) } }
    impl From<i32>    for AttrValueAuto { fn from(n: i32)    -> Self { Self(AttrValue::Static(n.to_string())) } }
    impl From<i64>    for AttrValueAuto { fn from(n: i64)    -> Self { Self(AttrValue::Static(n.to_string())) } }
    impl From<u32>    for AttrValueAuto { fn from(n: u32)    -> Self { Self(AttrValue::Static(n.to_string())) } }
    impl From<u64>    for AttrValueAuto { fn from(n: u64)    -> Self { Self(AttrValue::Static(n.to_string())) } }
    impl From<f64>    for AttrValueAuto { fn from(n: f64)    -> Self { Self(AttrValue::Static(n.to_string())) } }

    impl<T: Clone + serde::Serialize + 'static> From<&Signal<T>> for AttrValueAuto {
        fn from(s: &Signal<T>) -> Self {
            Self(AttrValue::Dynamic { signal: s.id(), format: None })
        }
    }
    impl<T: Clone + serde::Serialize + 'static> From<Signal<T>> for AttrValueAuto {
        fn from(s: Signal<T>) -> Self {
            Self(AttrValue::Dynamic { signal: s.id(), format: None })
        }
    }

    /// Wrap a view inside an `<Island>` boundary.
    pub fn wrap_in_island(name: &str, instance: u32, view: View) -> View {
        if let Some(ctx) = current_context() {
            ctx.register_island(name);
        }
        View::Island(IslandView {
            chunk_id: name.to_string(),
            instance_id: format!("{}-{}", name, instance),
            signal_ids: Vec::new(),
            view: Box::new(view),
            props: serde_json::Value::Null,
        })
    }

    pub use resuma_core::view::ElementBuilder;
    pub use resuma_core::view as view_mod;

    pub fn fragment(children: Vec<Child>) -> View {
        View::fragment(children)
    }

    /// `View::element(...)` shortcut — pulled into __private so the macro
    /// can reach it via `__private::element`.
    pub fn element(tag: &str) -> ElementBuilder {
        View::element(tag)
    }
}
