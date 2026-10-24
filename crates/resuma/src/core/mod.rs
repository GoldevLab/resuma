//! Resuma Core
//!
//! Core primitives shared by the framework:
//!
//! * [`Signal`] / [`ReadSignal`] / [`WriteSignal`] —
//!   fine-grained reactive state serialized into the resumability payload.
//! * [`Effect`] / [`Computed`] — SSR dependency tracking.
//!   For **client replay**, use the [`computed!`](crate::computed) and [`effect!`](crate::effect) macros.
//! * [`View`] — the virtual node tree returned by components.
//! * [`RenderContext`] — collects signals, handlers, and [`ResumePayload`]
//!   during SSR so the runtime can resume without re-running components.
//!
//! Every [`#[component]`](crate::component) wraps its output in a lazy handler boundary
//! (`<resuma-boundary>`). Handler JS is fetched from `/_resuma/handler/{Component}.js` unless
//! served from same-origin ESM chunks and loaded when an interaction needs them.

pub mod app_context;
pub mod component;
pub mod context;
pub mod effect;
pub mod error;
pub mod error_boundary;
pub mod flow_request;
pub mod for_list;
pub mod handler;
pub mod handler_combine;
pub mod match_branch;
pub mod nav;
pub mod overlay;
pub mod portal;
pub mod serialize;
pub mod show;
pub mod signal;
pub mod slot;
pub mod store;
pub mod stream;
pub mod task;
pub mod theme;
pub mod view;
pub mod view_transition;

pub use app_context::{
    ContextGuard, ContextId, provide_context, push_context_frame, try_use_context, use_context,
};
pub use component::{Component, IntoView};
pub use context::{
    RenderContext, RenderMode, ResumePayload, VisibleTaskSpec, current_context, page_needs_client,
    with_context, with_handler_chunk,
};
pub use effect::{
    Computed, Effect, attach_client_effect, use_computed, use_computed_with_js, use_effect,
};
pub use error::{Result, ResumaError};
pub use error_boundary::error_boundary;
pub use flow_request::FlowRequest;
pub use for_list::{ListSignal, for_signal, for_signal_virtual};
pub use handler::{HandlerCapture, HandlerRef, IslandRef, ServerActionRef};
pub use handler_combine::combine_js;
pub use match_branch::{MatchSignal, match_signal, match_static, match_value_string};
pub use nav::nav_link;
pub use overlay::{
    DEFAULT_POPUP_POSITIONS, GestureOpts, ModalOpts, PopupOpts, gesture_view, modal, modal_trigger,
    popup, popup_anchor,
};
pub use portal::portal;
pub use show::{show, show_signal};
pub use signal::{ReadSignal, Signal, SignalId, WriteSignal, signal, use_signal};
pub use slot::{SlotGuard, SlottedChild, push_slots, resolve_slot, with_default_slot};
pub use store::{NoSerialize, Store, no_serialize, use_store};
pub use stream::{stream_chunk, stream_slot};
pub use task::{
    VisibleTaskId, register_debounce_effect, use_debounce, use_task, use_visible_task,
    use_visible_task_with_captures, visible_task_js,
};
pub use theme::{
    HtmlTheme, OFFICIAL_PALETTE_CSS, OFFICIAL_THEME_DARK, OFFICIAL_THEME_IDS, Theme,
    official_palette_css, official_palette_css_for, provide_theme, theme_css_vars, theme_switch,
    use_theme,
};
pub use view::{Attr, AttrValue, Child, Element, Fragment, SlotView, View};
pub use view_transition::with_view_transition;
