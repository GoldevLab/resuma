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

pub use signal::{Signal, ReadSignal, WriteSignal, use_signal};
pub use effect::{Effect, Computed, use_effect, use_computed};
pub use view::{View, Element, Attr, AttrValue, Child, Fragment};
pub use component::{Component, IntoView};
pub use context::{RenderContext, RenderMode, with_context, current_context};
pub use handler::{HandlerRef, HandlerCapture, ServerActionRef, IslandRef};
pub use error::{ResumaError, Result};
