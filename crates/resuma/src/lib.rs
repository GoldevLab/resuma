//! # Resuma
//!
//! The first Rust web framework with **SSR + Resumability + Islands +
//! Server Actions + a friendly JS bridge** — all in one crate.
//!
//! Internal layout: `core`, `ssr`, `server`, `router`, `flow`, and optional `cli`.
//! Users typically depend on this crate only; `resuma-macros` is a separate proc-macro crate.

pub mod core;
pub mod ssr;
pub mod server;
pub mod router;
pub mod flow;

#[cfg(feature = "cli")]
pub mod cli;

pub use resuma_macros::{component, server, island, view, js, load, submit, layout, middleware};

pub use crate::core::{
    Signal, ReadSignal, WriteSignal, use_signal,
    Effect, Computed, use_effect, use_computed,
    View, IntoView, Component, Child,
    RenderContext, RenderMode, ResumaError, Result,
    Store, use_store, NoSerialize, no_serialize,
    SlottedChild, push_slots, resolve_slot, SlotGuard, with_default_slot,
    ContextId, provide_context, use_context,
    use_task, use_visible_task, use_debounce, visible_task_js,
    nav_link, combine_js,
    portal, with_view_transition, stream_slot, stream_chunk,
    Theme, provide_theme, use_theme, theme_css_vars,
    FlowRequest,
};

pub use crate::server::{
    ResumaApp, ServeOptions, register_server_action, set_action_middleware,
    SecurityConfig, configure_security, CSRF_HEADER, CSRF_FIELD,
};

pub use crate::ssr::{render_to_string, render_view, PageOptions, render_to_stream};

pub use crate::flow::{
    FlowApp, FlowServeOptions, FlowPwaConfig, LoadValue, SubmitValue, LoaderError, SubmitError,
    register_loader, register_submit, register_layout, register_middleware,
    register_loader_cache, register_stream_loader, register_stream_chunk,
    use_load, try_use_load, try_use_load_value, with_request, current_request, form, encode_submit_result,
    discover_pages, DiscoveredPage, FlowPageRegistry, apply_layouts,
    FlowError, error_page, not_found_page,
};

/// CLI entry point (`cargo install resuma`).
#[cfg(feature = "cli")]
pub fn run() -> anyhow::Result<()> {
    crate::cli::run()
}

pub mod prelude {
    //! Glob-friendly re-exports.
    pub use super::{
        Signal, ReadSignal, WriteSignal, use_signal,
        Effect, Computed, use_effect, use_computed,
        View, IntoView, Component,
        ResumaApp, ServeOptions, PageOptions,
        SecurityConfig, configure_security, set_action_middleware,
        CSRF_HEADER, CSRF_FIELD,
        component, server, island, view, js, load, submit, layout, middleware,
        render_to_string, render_view,
        FlowApp, FlowRequest, FlowServeOptions, use_load, try_use_load, try_use_load_value, form,
        current_request,
        LoadValue,
        Store, use_store, provide_context, use_context,
        use_task, use_visible_task, use_debounce, push_slots, resolve_slot, SlottedChild,
        nav_link, combine_js, SubmitError, LoaderError,
        FlowError, error_page, not_found_page,
        portal, with_view_transition, stream_slot,
        Theme, provide_theme, use_theme, theme_css_vars,
        ResumaError, Result,
        FlowPageRegistry, Child,
    };
}

#[doc(hidden)]
pub mod __private {
    //! Re-exports used by the macro-generated code.
    pub use ctor;
    pub use serde_json;
    pub use crate::core::{
        Signal, ReadSignal, WriteSignal,
        View, IntoView, Component, Child,
        view::{AttrValue, Element, Fragment, Island as IslandView},
        handler::{HandlerRef, HandlerCapture},
        signal::SignalId,
        context::{current_context, RenderContext, RenderMode},
        ResumaError, Result,
        slot::{SlottedChild, push_slots, resolve_slot, with_default_slot},
    };
    pub use crate::flow::form as flow_form;
    pub use crate::core::{combine_js, nav_link};
    pub use crate::server::register_server_action;

    #[derive(Debug, Clone)]
    pub enum HandlerSource {
        Inline(String),
        Chunk { chunk: String, symbol: String, source: String },
    }

    #[derive(Debug, Clone)]
    pub enum ResumeCapture {
        Signal { name: String, id: SignalId },
        Action(String),
    }

    pub use crate::core::view::Element as ElementType;

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
            for a in &actions {
                ctx.register_action(a);
            }
        }

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

    pub trait ElementBuilderExt {
        fn attr_runtime(self, kv: (String, AttrValue)) -> Self;
    }

    impl ElementBuilderExt for crate::core::view::ElementBuilder {
        fn attr_runtime(self, (name, value): (String, AttrValue)) -> Self {
            self.attr(name, value)
        }
    }

    pub fn render_component<C: Component>(props: C::Props) -> View {
        C::render(props)
    }

    pub fn resolve_attr_value<T: Into<AttrValueAuto>>(value: T) -> AttrValue {
        value.into().into_attr_value()
    }

    pub struct AttrValueAuto(AttrValue);

    impl AttrValueAuto {
        fn into_attr_value(self) -> AttrValue {
            self.0
        }
    }

    impl From<&str> for AttrValueAuto {
        fn from(s: &str) -> Self {
            Self(AttrValue::Static(s.to_string()))
        }
    }
    impl From<String> for AttrValueAuto {
        fn from(s: String) -> Self {
            Self(AttrValue::Static(s))
        }
    }
    impl From<bool> for AttrValueAuto {
        fn from(b: bool) -> Self {
            Self(AttrValue::Static(b.to_string()))
        }
    }
    impl From<i32> for AttrValueAuto {
        fn from(n: i32) -> Self {
            Self(AttrValue::Static(n.to_string()))
        }
    }
    impl From<i64> for AttrValueAuto {
        fn from(n: i64) -> Self {
            Self(AttrValue::Static(n.to_string()))
        }
    }
    impl From<u32> for AttrValueAuto {
        fn from(n: u32) -> Self {
            Self(AttrValue::Static(n.to_string()))
        }
    }
    impl From<u64> for AttrValueAuto {
        fn from(n: u64) -> Self {
            Self(AttrValue::Static(n.to_string()))
        }
    }
    impl From<f64> for AttrValueAuto {
        fn from(n: f64) -> Self {
            Self(AttrValue::Static(n.to_string()))
        }
    }

    impl<T: Clone + serde::Serialize + 'static> From<&Signal<T>> for AttrValueAuto {
        fn from(s: &Signal<T>) -> Self {
            Self(AttrValue::Dynamic {
                signal: s.id(),
                format: None,
            })
        }
    }
    impl<T: Clone + serde::Serialize + 'static> From<Signal<T>> for AttrValueAuto {
        fn from(s: Signal<T>) -> Self {
            Self(AttrValue::Dynamic {
                signal: s.id(),
                format: None,
            })
        }
    }

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

    pub use crate::core::view::ElementBuilder;
    pub use crate::core::view as view_mod;

    pub fn fragment(children: Vec<Child>) -> View {
        View::fragment(children)
    }

    pub fn element(tag: &str) -> ElementBuilder {
        View::element(tag)
    }
}
