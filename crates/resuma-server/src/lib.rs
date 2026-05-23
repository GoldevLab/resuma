//! Resuma HTTP server.
//!
//! Built on top of `axum`. Apps interact with this crate by:
//!
//!   1. Building a [`ResumaApp`] with [`ResumaApp::new`].
//!   2. Mounting page routes via [`ResumaApp::page`] / [`ResumaApp::route`].
//!   3. Spawning the HTTP listener with [`ResumaApp::serve`].
//!
//! The server provides the following built-in routes:
//!
//!   * `GET  /_resuma/runtime.js`               — embedded client runtime.
//!   * `POST /_resuma/action/:name`             — invokes a `#[server]` action.
//!   * `GET  /_resuma/handler/:chunk.js`        — handler chunk lazy-loaded by the runtime.
//!   * `GET  /_resuma/island/:chunk.js`         — island chunk loader.
//!   * `GET  /_resuma/island/:instance`         — re-rendered island HTML (HMR).

pub mod actions;
pub mod app;
pub mod runtime_asset;
pub mod handlers;

pub use app::{ResumaApp, ServeOptions};
pub use actions::{register_server_action, ActionFn};
