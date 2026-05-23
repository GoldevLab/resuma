//! # Resuma
//!
//! The first Rust web framework with **SSR + Resumability + Islands +
//! Server Actions + a friendly JS bridge** — all in one crate.
//!
//! Internal layout: `core`, `ssr`, `server`, `router`, `flow`, and optional `cli`.
//! Users typically depend on this crate only; `resuma-macros` is a separate proc-macro crate.

pub mod core;
pub mod flow;
pub mod router;
pub mod server;
pub mod ssr;

#[cfg(feature = "cli")]
pub mod cli;

pub use resuma_macros::{component, island, js, layout, load, middleware, server, submit, view};

pub use crate::core::{
    combine_js, nav_link, no_serialize, portal, provide_context, provide_theme, push_slots,
    resolve_slot, stream_chunk, stream_slot, theme_css_vars, use_computed, use_context,
    use_debounce, use_effect, use_signal, use_store, use_task, use_theme, use_visible_task,
    visible_task_js, with_default_slot, with_view_transition, Child, Component, Computed,
    ContextId, Effect, FlowRequest, IntoView, NoSerialize, ReadSignal, RenderContext, RenderMode,
    Result, ResumaError, Signal, SlotGuard, SlottedChild, Store, Theme, View, WriteSignal,
};

pub use crate::server::{
    configure_security, register_server_action, set_action_middleware, ResumaApp, SecurityConfig,
    ServeOptions, CSRF_FIELD, CSRF_HEADER,
};

pub use crate::ssr::{render_to_stream, render_to_string, render_view, PageOptions};

pub use crate::flow::{
    apply_layouts, current_request, discover_pages, encode_submit_result, error_page, form,
    not_found_page, register_layout, register_loader, register_loader_cache, register_middleware,
    register_stream_chunk, register_stream_loader, register_submit, try_use_load,
    try_use_load_value, use_load, with_request, DiscoveredPage, FlowApp, FlowError,
    FlowPageRegistry, FlowPwaConfig, FlowServeOptions, LoadValue, LoaderError, SubmitError,
    SubmitValue,
};

/// CLI entry point (`cargo install resuma`).
#[cfg(feature = "cli")]
pub fn run() -> anyhow::Result<()> {
    crate::cli::run()
}

pub mod prelude {
    //! Glob-friendly re-exports.
    pub use super::{
        combine_js, component, configure_security, current_request, error_page, form, island, js,
        layout, load, middleware, nav_link, not_found_page, portal, provide_context, provide_theme,
        push_slots, render_to_string, render_view, resolve_slot, server, set_action_middleware,
        stream_slot, submit, theme_css_vars, try_use_load, try_use_load_value, use_computed,
        use_context, use_debounce, use_effect, use_load, use_signal, use_store, use_task,
        use_theme, use_visible_task, view, with_view_transition, Child, Component, Computed,
        Effect, FlowApp, FlowError, FlowPageRegistry, FlowRequest, FlowServeOptions, IntoView,
        LoadValue, LoaderError, PageOptions, ReadSignal, Result, ResumaApp, ResumaError,
        SecurityConfig, ServeOptions, Signal, SlottedChild, Store, SubmitError, Theme, View,
        WriteSignal, CSRF_FIELD, CSRF_HEADER,
    };
}

#[doc(hidden)]
pub mod __private {
    //! Re-exports used by the macro-generated code.
    pub use crate::core::{combine_js, nav_link};
    pub use crate::core::{
        context::{current_context, RenderContext, RenderMode},
        handler::{HandlerCapture, HandlerRef},
        signal::SignalId,
        slot::{push_slots, resolve_slot, with_default_slot, SlottedChild},
        view::{AttrValue, Element, Fragment, Island as IslandView},
        Child, Component, IntoView, ReadSignal, Result, ResumaError, Signal, View, WriteSignal,
    };
    pub use crate::flow::form as flow_form;
    pub use crate::server::register_server_action;
    pub use ctor;
    pub use serde_json;

    #[derive(Debug, Clone)]
    pub enum HandlerSource {
        Inline(String),
        Chunk {
            chunk: String,
            symbol: String,
            source: String,
        },
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

    pub use crate::core::view as view_mod;
    pub use crate::core::view::ElementBuilder;

    pub fn fragment(children: Vec<Child>) -> View {
        View::fragment(children)
    }

    pub fn element(tag: &str) -> ElementBuilder {
        View::element(tag)
    }
}
