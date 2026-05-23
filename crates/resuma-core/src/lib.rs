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

pub mod signal;
pub mod effect;
pub mod view;
pub mod component;
pub mod context;
pub mod handler;
pub mod serialize;
pub mod error;
pub mod store;
pub mod slot;
pub mod app_context;
pub mod task;
pub mod nav;
pub mod handler_combine;
pub mod portal;
pub mod view_transition;
pub mod theme;
pub mod stream;
pub mod flow_request;

pub use signal::{Signal, ReadSignal, WriteSignal, use_signal, SignalId};
pub use effect::{Effect, Computed, use_effect, use_computed};
pub use view::{View, Element, Attr, AttrValue, Child, Fragment, SlotView};
pub use component::{Component, IntoView};
pub use context::{RenderContext, RenderMode, ResumePayload, page_needs_client, with_context, current_context};
pub use handler::{HandlerRef, HandlerCapture, ServerActionRef, IslandRef};
pub use error::{ResumaError, Result};
pub use store::{Store, use_store, NoSerialize, no_serialize};
pub use slot::{SlottedChild, push_slots, resolve_slot, SlotGuard, with_default_slot};
pub use app_context::{ContextId, provide_context, use_context, push_context_frame, ContextGuard};
pub use task::{use_task, use_visible_task, use_debounce, VisibleTaskId, visible_task_js};
pub use nav::nav_link;
pub use handler_combine::combine_js;
pub use portal::portal;
pub use view_transition::with_view_transition;
pub use theme::{Theme, provide_theme, use_theme, theme_css_vars};
pub use stream::{stream_slot, stream_chunk};
pub use flow_request::FlowRequest;
