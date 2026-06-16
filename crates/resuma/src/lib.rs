//! # Resuma
//!
//! **SSR + resumability for Rust** — components run on the server only; the browser
//! resumes serialized signals and lazy handler chunks instead of re-hydrating the tree.
//!
//! ## Quick start
//!
//! ```no_run
//! use resuma::prelude::*;
//!
//! #[component]
//! fn Counter() {
//!     let n = signal(0);
//!     view! {
//!         <button onClick={n.update(|v| *v += 1)}>{n}</button>
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> std::io::Result<()> {
//!     ResumaApp::new()
//!         .component("/", Counter)
//!         .serve(ServeOptions::default())
//!         .await
//! }
//! ```
//!
//! Install the CLI: `cargo install resuma`. Narrative guides live at
//! [resuma-docs.fly.dev](https://resuma-docs.fly.dev/docs).
//!
//! ## Resumability model (v0.4)
//!
//! * Every [`#[component]`](component) is a **resumable boundary** — handlers register
//!   under `/_resuma/handler/{Component}.js` and prefetch when the boundary enters the viewport.
//! * [`computed!`](computed), [`effect!`](effect), and [`debounce!`](debounce) translate Rust
//!   closures to client-replayable JS via rs2js (in `resuma-macros`).
//! * Plain [`use_computed`] / [`use_effect`] run on SSR only;
//!   use the macros when the browser must replay derived state or side effects.
//! * [`#[island]`](island) is **optional** — for heavy lazy bundles, `load = "visible"`, or dev HMR.
//!
//! ## Crate layout
//!
//! | Module | Role |
//! |--------|------|
//! | [`core`] | Signals, `View`, [`RenderContext`], [`ResumePayload`] |
//! | [`ssr`] | HTML rendering + embedded resumability payload |
//! | [`mod@server`] | axum HTTP, `ResumaApp`, `/_resuma/*` assets |
//! | [`flow`] | `FlowApp`, file-based pages, `#[load]`, `#[submit]` |
//! | [`router`] | Page discovery scanner |
//! | [`cli`] | `resuma new` / `dev` / `build` (feature `cli`) |
//!
//! Users depend on **`resuma`** only; [`resuma-macros`](https://docs.rs/resuma-macros) is a separate
//! proc-macro crate required by the build.
//!
//! ## Re-exports
//!
//! Most apps start with [`prelude`] (`use resuma::prelude::*`). Macros (`view!`, `#[component]`,
//! `#[server]`, `#[data]`, Flow attributes) and common types are re-exported at the crate root
//! for convenience.

pub mod client;
pub mod core;
pub mod flow;
pub mod router;
pub mod server;
pub mod ssr;

#[cfg(feature = "cli")]
pub mod cli;

pub use resuma_macros::{
    component, computed, data, debounce, effect, island, js, layout, load, middleware, server,
    submit, view, Store,
};

pub use crate::client::{
    client_component, client_script_url, ClientComponent, CLIENT_SCRIPT_PREFIX,
};

pub use crate::core::view::AttrValue;
pub use crate::core::{
    combine_js, error_boundary, for_signal, match_signal, nav_link, no_serialize, portal,
    provide_context, provide_theme, push_slots, resolve_slot, show, show_signal, signal, stream_chunk, stream_slot, theme_css_vars,
    use_computed, use_computed_with_js, use_context, use_debounce, use_effect, use_signal,
    use_store, use_task, use_theme, use_visible_task, visible_task_js, with_default_slot,
    with_view_transition, Child, Component, Computed, ContextId, Effect, FlowRequest, IntoView,
    NoSerialize, ReadSignal, RenderContext, RenderMode, Result, ResumaError, ResumePayload, Signal,
    SlotGuard, SlottedChild, Store, Theme, View, WriteSignal,
};

pub use crate::server::{
    build_content_security_policy, configure_security, register_server_action,
    set_action_middleware, CspConfig, ResumaApp, SecurityConfig, ServeOptions, CSRF_FIELD,
    CSRF_HEADER,
};

pub use crate::ssr::seo_kit::{AiCrawlerPolicy, MetaTag, SeoKit};
pub use crate::ssr::{render_to_stream, render_to_string, render_view, PageOptions};

pub use crate::flow::{
    apply_layouts, build_query_href, collect_public_dir, current_location_href, current_request,
    discover_pages, encode_submit_result, error_page, extract_redirect, flash_message, form,
    invalidate_href, invalidate_href_now, invalidate_link, load_boundary, loader_refresh_form,
    loader_refresh_input, not_found_page, query_nav_link, redirect, redirect_with_flash,
    register_layout, register_loader, register_loader_cache, register_middleware,
    register_stream_chunk, register_stream_loader, register_submit, theme_into_pwa, try_use_load,
    try_use_load_value, use_load, with_request, DiscoveredPage, FlowApp, FlowError, FlowExtensions,
    FlowPageRegistry, FlowPwaConfig, FlowServeOptions, FromFlowRequest, LoadValue, LoaderError,
    Path, PublicAsset, PwaShortcut, Query, Redirect, SubmitError, SubmitValue,
};

/// CLI entry point (`cargo install resuma`).
#[cfg(feature = "cli")]
pub fn run() -> anyhow::Result<()> {
    crate::cli::run()
}

pub mod prelude {
    //! Convenient re-exports for application code.
    //!
    //! ```rust,ignore
    //! use resuma::prelude::*;
    //! ```
    //!
    //! Includes:
    //!
    //! * **Macros** — [`view!`](crate::view), [`#[component]`](crate::component),
    //!   [`#[server]`](macro@crate::server), [`#[data]`](macro@crate::data), [`computed!`](crate::computed),
    //!   [`effect!`](crate::effect), [`debounce!`](crate::debounce), Flow (`#[load]`, `#[submit]`, …)
    //! * **Components** — [`View`], [`Signal`], [`Component`]
    //! * **Apps** — [`ResumaApp`], [`FlowApp`],
    //!   [`ServeOptions`], [`FlowServeOptions`]
    //! * **SSR** — [`render_to_string`], [`render_view`]
    //! * **Flow runtime** — [`FlowRequest`], [`current_request`],
    //!   [`use_load`], [`form`](crate::form())
    //! * **Client components** — [`ClientComponent`], [`client_component`]
    //!
    //! For low-level types ([`RenderContext`](crate::RenderContext), [`ResumePayload`](crate::ResumePayload)),
    //! import from [`crate::core`].
    pub use super::{
        build_query_href, client_component, client_script_url, combine_js, component, computed,
        configure_security, current_request, data, debounce, effect, error_boundary, error_page,
        extract_redirect, flash_message, form, for_signal, invalidate_href, invalidate_href_now,
        invalidate_link, island, js, layout, load, load_boundary, loader_refresh_form,
        loader_refresh_input, match_signal, middleware, nav_link, not_found_page, portal,
        provide_context, provide_theme, push_slots, query_nav_link, redirect, redirect_with_flash,
        render_to_string, render_view, resolve_slot, server, set_action_middleware, show, signal,
        stream_slot, submit, theme_css_vars, try_use_load, try_use_load_value, use_computed,
        use_computed_with_js, use_context, use_debounce, use_effect, use_load, use_signal,
        use_store, use_task, use_theme, use_visible_task, view, with_view_transition, AttrValue,
        Child, ClientComponent, Component, Computed, CspConfig, Effect, FlowApp, FlowError,
        FlowPageRegistry, FlowPwaConfig, FlowRequest, FlowServeOptions, FromFlowRequest, IntoView,
        LoadValue, LoaderError, PageOptions, Path, PublicAsset, PwaShortcut, Query, ReadSignal, Redirect, Result,
        ResumaApp, ResumaError, SecurityConfig, ServeOptions, Signal, SlottedChild, Store,
        SubmitError, Theme, View, WriteSignal, CLIENT_SCRIPT_PREFIX, CSRF_FIELD, CSRF_HEADER,
    };
}

#[doc(hidden)]
pub mod __private {
    //! Re-exports used by the macro-generated code.
    pub use crate::core::effect::{attach_client_effect, use_computed_with_js, use_effect};
    pub use crate::core::task::{register_debounce_effect, use_debounce};
    pub use crate::core::{combine_js, for_signal, match_signal, match_static, nav_link, show, show_signal};
    pub use crate::core::{
        context::{current_context, with_handler_chunk, RenderContext, RenderMode},
        handler::{HandlerCapture, HandlerRef},
        signal::SignalId,
        slot::{push_slots, resolve_slot, with_default_slot, SlottedChild},
        view::{AttrValue, Element, Fragment, Island as IslandView},
        Child, Component, IntoView, ReadSignal, Result, ResumaError, Signal, View, WriteSignal,
    };
    pub use crate::flow::form as flow_form;
    pub use crate::server::register_server_action;
    pub use ctor;
    pub use serde;
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
        _chunk: &str,
        symbol: &str,
        js_source: &str,
        captures: Vec<ResumeCapture>,
        actions: Vec<String>,
    ) -> AttrValue {
        let chunk = current_context()
            .map(|c| c.current_handler_chunk())
            .unwrap_or_else(|| "__page__".to_string());

        if let Some(ctx) = current_context() {
            ctx.register_handler(&chunk, symbol, js_source);
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

        let inline = if chunk == "__page__"
            && js_source.len() <= crate::core::context::INLINE_HANDLER_MAX_BYTES
        {
            Some(js_source.to_string())
        } else {
            None
        };

        AttrValue::Handler(HandlerRef {
            event: event.to_string(),
            chunk,
            symbol: symbol.to_string(),
            captures: signal_captures,
            inline,
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

    pub fn wrap_in_island(
        name: &str,
        instance: u32,
        build: impl FnOnce() -> View,
        load: &str,
    ) -> View {
        if let Some(ctx) = current_context() {
            ctx.register_island(name);
        }
        let load = match load {
            "visible" | "Visible" => view_mod::IslandLoad::Visible,
            _ => view_mod::IslandLoad::Eager,
        };
        let inner = crate::core::context::with_handler_chunk(name, build);
        View::Island(IslandView {
            chunk_id: name.to_string(),
            instance_id: format!("{}-{}", name, instance),
            signal_ids: Vec::new(),
            view: Box::new(inner),
            props: serde_json::Value::Null,
            load,
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
