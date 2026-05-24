//! Resuma Core
//!
//! Core primitives shared by the framework:
//!  * `Signal<T>` / `ReadSignal<T>` / `WriteSignal<T>` — fine grained reactive state.
//!  * `Effect` / `Computed<T>` — automatic dependency tracking.
//!  * `View` — the resumable virtual node tree returned by components.
//!  * `Component` — trait implemented by every renderable unit.
//!  * `RenderContext` — collects state, handlers and islands during SSR so the
//!    runtime can resume execution on the client without re-running components.
//!
//! The big idea: components are *only* executed on the server. Their reactive
//! dependencies, event handler references and serialized state travel inside
//! the HTML payload. A tiny JS runtime then resumes execution, no hydration.

pub mod app_context;
pub mod component;
pub mod context;
pub mod effect;
pub mod error;
pub mod flow_request;
pub mod handler;
pub mod handler_combine;
pub mod nav;
pub mod portal;
pub mod serialize;
pub mod signal;
pub mod slot;
pub mod store;
pub mod stream;
pub mod task;
pub mod theme;
pub mod view;
pub mod view_transition;

pub use app_context::{provide_context, push_context_frame, use_context, ContextGuard, ContextId};
pub use component::{Component, IntoView};
pub use context::{
    current_context, page_needs_client, with_context, with_handler_chunk, RenderContext,
    RenderMode, ResumePayload,
};
pub use effect::{
    attach_client_effect, use_computed, use_computed_with_js, use_effect, Computed, Effect,
};
pub use error::{Result, ResumaError};
pub use flow_request::FlowRequest;
pub use handler::{HandlerCapture, HandlerRef, IslandRef, ServerActionRef};
pub use handler_combine::combine_js;
pub use nav::nav_link;
pub use portal::portal;
pub use signal::{use_signal, ReadSignal, Signal, SignalId, WriteSignal};
pub use slot::{push_slots, resolve_slot, with_default_slot, SlotGuard, SlottedChild};
pub use store::{no_serialize, use_store, NoSerialize, Store};
pub use stream::{stream_chunk, stream_slot};
pub use task::{
    register_debounce_effect, use_debounce, use_task, use_visible_task, visible_task_js,
    VisibleTaskId,
};
pub use theme::{provide_theme, theme_css_vars, use_theme, Theme};
pub use view::{Attr, AttrValue, Child, Element, Fragment, SlotView, View};
pub use view_transition::with_view_transition;
