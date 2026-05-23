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
//!   * `GET  /_resuma/loader.js`                — tiny event bootstrap (~1–2 KB).
//!   * `GET  /_resuma/core.js`                  — lazy-loaded resumability core.
//!   * `GET  /_resuma/runtime.js`               — legacy monolithic runtime.
//!   * `POST /_resuma/action/:name`             — invokes a `#[server]` action.
//!   * `GET  /_resuma/handler/:chunk.js`        — handler chunk lazy-loaded by the runtime.
//!   * `GET  /_resuma/island/:chunk.js`         — island chunk loader.
//!   * `GET  /_resuma/island/:instance`         — re-rendered island HTML (HMR).

pub mod security;
pub mod actions;
pub mod compressed_asset;
pub mod app;
pub mod runtime_asset;
pub mod handlers;
pub mod page_cache;
pub mod request_path;
pub mod deferred_stream;

pub use app::{apply_security_headers, security_headers_middleware, ResumaApp, ServeOptions};
pub use security::{
    client_ip, client_ip_from_parts, configure as configure_security, csrf_token, guard_mutation,
    http_status, random_token, request_is_https, CspNonce, CSRF_COOKIE, CSRF_FIELD, CSRF_HEADER,
    SecurityConfig, SecurityHeaderOptions,
};
pub use page_cache::{page_csrf, stage_page_csrf, stage_response_cache_control, take_response_cache_control};
pub use request_path::{stage_response_path, take_response_path};
pub use deferred_stream::{set_deferred_stream_hook, try_deferred_stream};
pub use actions::{register_server_action, ActionFn, set_action_middleware};
