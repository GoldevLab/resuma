//! Resuma Flow — full-stack pages, loaders, submits, and middleware on top of [`ResumaApp`](crate::server::ResumaApp).
//!
//! Use [`FlowApp`] for multi-page sites with `src/pages/`, [`#[load]`](crate::load),
//! [`#[submit]`](crate::submit), and [`#[layout]`](crate::layout). File-based routing is
//! discovered via [`discover_pages`] or `FlowApp::auto_pages`.

pub mod action_hook;
pub mod app;
pub mod cache;
pub mod errors;
pub mod extensions;
pub mod extract;
pub mod form;
pub mod invalidate;
pub mod layout;
pub mod load;
pub mod match_route;
pub mod middleware;
pub mod nav;
pub mod pages;
pub mod public;
pub mod pwa;
pub mod redirect;
pub mod registry;
pub mod request;
pub mod routes;
pub mod runtime;
pub mod submit;

pub mod stream_load;

pub use app::{FlowApp, FlowServeOptions};
pub use cache::{loader_cache, merge_cache_control, register_loader_cache};
pub use errors::{error_page, not_found_page, registry_miss_page, FlowError};
pub use extensions::{global_extensions, set_global_extensions, FlowExtensions};
pub use extract::{FromFlowRequest, Path, Query};
pub use form::form;
pub use invalidate::{invalidate_href, invalidate_href_now, invalidate_link};
pub use layout::{apply_layouts, register_layout};
pub use load::{load_boundary, LoadValue, LoaderError};
pub use match_route::{match_route, RouteMatch};
pub use middleware::{register_middleware, run_middleware};
pub use nav::{
    build_query_href, current_location_href, loader_refresh_form, loader_refresh_input,
    query_nav_link, theme_into_pwa,
};
pub use pages::{discover_pages, DiscoveredPage, DiscoveredRoute, FlowPageRegistry};
pub use public::{collect_public_dir, default_public_dir, PublicAsset};
pub use pwa::{FlowPwaConfig, PwaShortcut};
pub use redirect::{
    extract_redirect, flash_message, redirect, redirect_response, redirect_with_flash, Redirect,
    FLASH_KEY,
};
pub use registry::{dispatch_load, dispatch_submit, register_loader, register_submit};
pub use request::FlowRequest;
pub use routes::{
    attach_flow_routes, attach_seo_kit_routes, FlowSeoConfig, SeoKitRouteOpts, SubmitResponse,
};
pub use runtime::{
    current_request, set_load_cache, try_use_load, try_use_load_value, use_load, with_request,
};
pub use stream_load::{register_stream_chunk, register_stream_loader};
pub use submit::{encode_submit_result, SubmitError, SubmitValue};
