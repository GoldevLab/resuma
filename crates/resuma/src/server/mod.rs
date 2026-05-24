//! Resuma HTTP server.
//!
//! Built on **axum**. Typical flow:
//!
//!   1. [`ResumaApp::new`]
//!   2. [`ResumaApp::page`] / [`ResumaApp::page_with_request`]
//!   3. [`ResumaApp::serve`] with [`ServeOptions::from_env`]
//!
//! ## Built-in routes
//!
//! | Route | Purpose |
//! |-------|---------|
//! | `GET /_resuma/loader.js` | Tiny bootstrap (~884 B gzip) |
//! | `GET /_resuma/core.js` | Lazy-loaded resumability core |
//! | `GET /_resuma/runtime.js` | Legacy monolithic runtime |
//! | `GET /_resuma/handler/:chunk.js` | Lazy handler chunk (`#[component]` boundaries) |
//! | `GET /_resuma/island-chunk/:chunk.js` | Optional `#[island]` chunk |
//! | `GET /_resuma/island/:instance` | Cached island HTML (dev HMR refresh) |
//! | `GET /_resuma/dev/ws` | Dev WebSocket when `RESUMA_DEV=1` |
//! | `POST /_resuma/action/:name` | [`#[server]`](crate::server) RPC |

pub mod actions;
pub mod app;
pub mod compressed_asset;
pub mod deferred_stream;
pub mod dev;
pub mod handler_assets;
pub mod handlers;
pub mod island_cache;
pub mod listen;
pub mod page_cache;
pub mod request_path;
pub mod runtime_asset;
pub mod security;
pub mod static_assets;

pub use actions::{register_server_action, set_action_middleware, ActionFn};
pub use app::{apply_security_headers, security_headers_middleware, ResumaApp, ServeOptions};
pub use deferred_stream::{set_deferred_stream_hook, try_deferred_stream};
pub use listen::listen_addr_from_env;
pub use page_cache::{
    page_csrf, stage_page_csrf, stage_response_cache_control, take_response_cache_control,
};
pub use request_path::{stage_response_path, take_response_path};
pub use security::{
    client_ip, client_ip_from_parts, configure as configure_security, csrf_token, guard_mutation,
    http_status, random_token, request_is_https, CspNonce, SecurityConfig, SecurityHeaderOptions,
    CSRF_COOKIE, CSRF_FIELD, CSRF_HEADER,
};
