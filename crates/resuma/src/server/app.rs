//! `ResumaApp` — high-level builder used by example apps & the CLI dev server.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::Arc;

use crate::core::context::{with_context, RenderContext, RenderMode};
use crate::core::view::View;
use crate::core::Component;
use crate::core::{FlowRequest, ResumaError};
use crate::flow::extract_redirect;
use crate::ssr::PageOptions;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::extract::DefaultBodyLimit;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderValue, Request, StatusCode, Uri};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::actions::dispatch as dispatch_action;
use super::compressed_asset::{self, core_asset, loader_asset, runtime_asset, serve_js};
use super::deferred_stream::try_deferred_stream;
use super::page_cache::take_response_cache_control;
use super::runtime_asset::{CORE_JS, LOADER_JS, RUNTIME_JS};
use super::security::{
    self, client_ip_from_parts, csrf_set_cookie, csrf_token, guard_mutation, http_status,
    random_token, request_is_https, CspNonce, SecurityConfig, SecurityHeaderOptions,
};

/// HTTP application builder for single-page and manual-route apps.
///
/// Register routes with [`page`](Self::page) or [`page_with_request`](Self::page_with_request)
/// (the latter receives [`FlowRequest`] — query, headers, method). Mount the axum router with
/// [`serve`](Self::serve).
///
/// Built-in endpoints include `/_resuma/loader.js`, `/_resuma/handler/:chunk.js`, and
/// `POST /_resuma/action/:name` for [`#[server]`](macro@crate::server) actions.
pub struct ResumaApp {
    page_factories: HashMap<String, Arc<PageFactory>>,
    handler_chunks: Arc<RwLock<HashMap<String, String>>>,
    island_chunks: Arc<RwLock<HashMap<String, String>>>,
    page_options: PageOptions,
    /// When true, HTML is sent as chunked stream (head → body → tail).
    streaming: bool,
    /// Optional catch-all page renderer (used by Resuma Flow for param routes).
    fallback: Option<Arc<FallbackFactory>>,
}

type PageFactory = dyn Fn(FlowRequest) -> View + Send + Sync;
type FallbackFactory = dyn Fn(&str, FlowRequest) -> Option<View> + Send + Sync;

/// Listen options for [`ResumaApp::serve`].
///
/// [`Default`] and [`Self::from_env`] read `RESUMA_ADDR` or `HOST` + `PORT`
/// (defaults to `127.0.0.1:3000`). Security settings come from [`SecurityConfig::from_env`].
#[derive(Debug, Clone)]
pub struct ServeOptions {
    pub addr: SocketAddr,
    pub security: SecurityConfig,
}

impl Default for ServeOptions {
    fn default() -> Self {
        Self::from_env()
    }
}

impl ServeOptions {
    /// Read bind address from `RESUMA_ADDR` or `HOST` + `PORT` (same as [`crate::FlowServeOptions`]).
    pub fn from_env() -> Self {
        Self {
            addr: super::listen::listen_addr_from_env(),
            security: SecurityConfig::from_env(),
        }
    }
}

impl ResumaApp {
    pub fn new() -> Self {
        Self {
            page_factories: HashMap::new(),
            handler_chunks: Arc::new(RwLock::new(HashMap::new())),
            island_chunks: Arc::new(RwLock::new(HashMap::new())),
            page_options: PageOptions {
                lang: "en".into(),
                title: "Resuma App".into(),
                ..Default::default()
            },
            streaming: false,
            fallback: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.page_options.title = title.into();
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.page_options.description = description.into();
        self
    }

    pub fn with_site_url(mut self, url: impl Into<String>) -> Self {
        self.page_options.site_url = url.into();
        self
    }

    pub fn with_og_image(mut self, image: impl Into<String>) -> Self {
        self.page_options.og_image = image.into();
        self
    }

    pub fn with_json_ld(mut self, json_ld: impl Into<String>) -> Self {
        self.page_options.json_ld = json_ld.into();
        self
    }

    pub fn with_pwa(mut self, pwa: crate::ssr::PwaOptions) -> Self {
        self.page_options.pwa = Some(pwa);
        self
    }

    pub(crate) fn page_options(&self) -> &PageOptions {
        &self.page_options
    }

    pub(crate) fn page_options_mut(&mut self) -> &mut PageOptions {
        &mut self.page_options
    }

    pub fn with_stylesheet(mut self, href: impl Into<String>) -> Self {
        self.page_options.stylesheet = Some(href.into());
        self
    }

    /// Append raw markup to the document `<head>`. Useful for embedding
    /// inline `<style>` blocks during development.
    pub fn with_head(mut self, head: impl Into<String>) -> Self {
        self.page_options.head = head.into();
        self
    }

    /// Enable chunked streaming SSR (lower TTFB — head sent before body).
    pub fn with_streaming(mut self, enabled: bool) -> Self {
        self.streaming = enabled;
        self
    }

    /// Register a page route with per-request HTTP context (query, headers, method).
    ///
    /// Prefer this over [`page`](Self::page) when the handler reads `FlowRequest` fields.
    pub fn page_with_request<F>(mut self, path: &str, factory: F) -> Self
    where
        F: Fn(FlowRequest) -> View + Send + Sync + 'static,
    {
        self.page_factories
            .insert(path.to_string(), Arc::new(factory));
        self
    }

    /// Register a page route without HTTP context (legacy / simple apps).
    pub fn page<F>(self, path: &str, factory: F) -> Self
    where
        F: Fn() -> View + Send + Sync + 'static,
    {
        self.page_with_request(path, move |_req| factory())
    }

    /// Register a no-props component route without spelling
    /// `Component::render(ComponentProps::default())`.
    ///
    /// ```rust,ignore
    /// ResumaApp::new().component("/", App)
    /// ```
    pub fn component<C>(self, path: &str, _component: C) -> Self
    where
        C: Component + 'static,
        C::Props: Default,
    {
        self.page(path, || C::render(Default::default()))
    }

    /// Catch-all renderer for dynamic routes (Resuma Flow param patterns).
    pub fn fallback_with_request<F>(mut self, factory: F) -> Self
    where
        F: Fn(&str, FlowRequest) -> Option<View> + Send + Sync + 'static,
    {
        self.fallback = Some(Arc::new(factory));
        self
    }

    /// Catch-all without HTTP context.
    pub fn fallback<F>(self, factory: F) -> Self
    where
        F: Fn(&str) -> Option<View> + Send + Sync + 'static,
    {
        self.fallback_with_request(move |path, _req| factory(path))
    }

    /// Register a precompiled handler chunk to be served at
    /// `/_resuma/handler/<chunk>.js`.
    pub fn handler_chunk(self, chunk_id: &str, source: impl Into<String>) -> Self {
        self.handler_chunks
            .write()
            .insert(chunk_id.to_string(), source.into());
        self
    }

    /// Register a precompiled island chunk to be served at
    /// `/_resuma/island-chunk/<chunk>.js`.
    pub fn island_chunk(self, chunk_id: &str, source: impl Into<String>) -> Self {
        self.island_chunks
            .write()
            .insert(chunk_id.to_string(), source.into());
        self
    }

    pub async fn serve(self, opts: ServeOptions) -> std::io::Result<()> {
        security::configure(opts.security.clone());
        let router = self
            .into_router()
            .layer(DefaultBodyLimit::max(opts.security.body_limit_bytes))
            .layer(middleware::from_fn(security_headers_middleware))
            .layer(middleware::from_fn(super::ops::request_id_middleware));
        let (listener, bound) = super::listen::bind_listener(opts.addr).await?;
        info!(addr = %bound, "resuma server listening");
        println!("resuma listening on http://{}", bound);
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(super::ops::shutdown_signal())
        .await
    }

    pub fn into_router(self) -> Router {
        let security_cfg = security::config();
        let state = Arc::new(AppState {
            pages: self.page_factories,
            handler_chunks: self.handler_chunks,
            island_chunks: self.island_chunks,
            page_options: self.page_options,
            streaming: self.streaming,
            fallback: self.fallback,
            hide_benchmark: security_cfg.hide_benchmark,
        });

        let mut router = Router::new();
        for path in state.pages.keys() {
            let p = path.clone();
            router = router.route(&p, get(serve_page));
        }

        // Liveness / readiness probes (skipped if the app defines its own).
        if !state.pages.contains_key(super::ops::HEALTH_PATH) {
            router = router.route(super::ops::HEALTH_PATH, get(super::ops::health));
        }
        if !state.pages.contains_key(super::ops::READY_PATH) {
            router = router.route(super::ops::READY_PATH, get(super::ops::ready));
        }

        router = router.fallback(get(serve_fallback));

        if !state.hide_benchmark {
            router = router.route("/_resuma/benchmark.json", get(serve_benchmark));
        }

        let mut router = router
            .route("/_resuma/loader.js", get(serve_loader))
            .route("/_resuma/core.js", get(serve_core))
            .route("/_resuma/runtime.js", get(serve_runtime))
            .route("/_resuma/action/:name", post(serve_action))
            .route("/_resuma/handler/:chunk", get(serve_handler_chunk))
            .route("/_resuma/island-chunk/:chunk", get(serve_island_chunk))
            .route("/_resuma/island/:instance", get(serve_island_refresh));

        if super::dev::dev_mode_enabled() {
            router = router.route("/_resuma/dev/ws", get(super::dev::dev_ws_handler));
        }

        router.with_state(state)
    }
}

/// Apply standard security headers to every HTTP response.
pub fn apply_security_headers(response: Response, opts: &SecurityHeaderOptions) -> Response {
    security::apply_security_headers(response, opts)
}

pub async fn security_headers_middleware(req: Request<Body>, next: Next) -> Response {
    let https = request_is_https(&req);
    let res = next.run(req).await;
    let nonce = res.extensions().get::<CspNonce>().map(|n| n.0.clone());
    apply_security_headers(
        res,
        &SecurityHeaderOptions {
            csp_nonce: nonce,
            https,
        },
    )
}

impl Default for ResumaApp {
    fn default() -> Self {
        Self::new()
    }
}

struct AppState {
    pages: HashMap<String, Arc<PageFactory>>,
    handler_chunks: Arc<RwLock<HashMap<String, String>>>,
    island_chunks: Arc<RwLock<HashMap<String, String>>>,
    page_options: PageOptions,
    streaming: bool,
    fallback: Option<Arc<FallbackFactory>>,
    hide_benchmark: bool,
}

fn page_security_opts(base: &PageOptions) -> PageOptions {
    let mut opts = base.clone();
    opts.csp_nonce = random_token();
    opts.csrf_token = csrf_token();
    opts
}

fn attach_page_security(mut res: Response, opts: &PageOptions, https: bool) -> Response {
    if !opts.csrf_token.is_empty() {
        res.headers_mut()
            .insert(header::SET_COOKIE, csrf_set_cookie(&opts.csrf_token, https));
    }
    res.extensions_mut()
        .insert(CspNonce(opts.csp_nonce.clone()));
    res
}

fn render_page_response(
    state: &AppState,
    view: View,
    ctx: Rc<RenderContext>,
    opts: PageOptions,
    path: &str,
    https: bool,
) -> Response {
    let cache = take_response_cache_control();
    if state.streaming {
        use axum::body::Body;
        use futures_util::StreamExt;

        let body = crate::ssr::render_view(&view);
        let payload = ctx.snapshot_full();
        super::handler_assets::merge_payload_handlers(
            &state.handler_chunks,
            &state.island_chunks,
            &payload,
        );

        let stream = if let Some(deferred) = try_deferred_stream(view.clone(), &opts, path) {
            deferred
        } else {
            crate::ssr::build_page_stream(opts.clone(), path, body.clone(), payload, vec![body])
        };

        let stream = stream.map(|chunk| {
            chunk
                .map(axum::body::Bytes::from)
                .map_err(std::io::Error::other)
        });
        let mut builder = Response::builder()
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::TRANSFER_ENCODING, "chunked");
        if let Some(ref cache) = cache {
            builder = builder.header(header::CACHE_CONTROL, cache.as_str());
        }
        let res = match builder
            .header("x-robots-tag", "index, follow")
            .body(Body::from_stream(stream))
        {
            Ok(res) => res,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        attach_page_security(res, &opts, https)
    } else {
        let payload = ctx.snapshot_full();
        let html = crate::ssr::render_prebuilt_document(&opts, path, &view, &payload);
        super::handler_assets::merge_payload_handlers(
            &state.handler_chunks,
            &state.island_chunks,
            &payload,
        );
        let mut res = Html(html).into_response();
        if let Some(cache) = cache {
            res.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_str(&cache)
                    .unwrap_or_else(|_| HeaderValue::from_static("no-store")),
            );
        }
        res.headers_mut().insert(
            header::HeaderName::from_static("x-robots-tag"),
            HeaderValue::from_static("index, follow"),
        );
        attach_page_security(res, &opts, https)
    }
}

async fn serve_page(uri: Uri, State(state): State<Arc<AppState>>, req: Request<Body>) -> Response {
    let path = uri.path().to_string();
    let factory = match state.pages.get(&path) {
        Some(f) => f.clone(),
        None => return (StatusCode::NOT_FOUND, "not found").into_response(),
    };

    let flow_req = crate::flow::request::from_http_request(&req, &path, Default::default());
    let opts = page_security_opts(&state.page_options);
    super::page_cache::stage_page_csrf(opts.csrf_token.clone());
    let ctx = RenderContext::new(RenderMode::Ssr);
    let view = with_context(ctx.clone(), || factory(flow_req));
    render_page_response(&state, view, ctx, opts, &path, request_is_https(&req))
}

async fn serve_fallback(
    uri: Uri,
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> Response {
    let path = uri.path();
    let flow_req = crate::flow::request::from_http_request(&req, path, Default::default());
    if let Some(fb) = &state.fallback {
        let opts = page_security_opts(&state.page_options);
        super::page_cache::stage_page_csrf(opts.csrf_token.clone());
        let ctx = RenderContext::new(RenderMode::Ssr);
        if let Some(view) = with_context(ctx.clone(), || fb(path, flow_req)) {
            return render_page_response(&state, view, ctx, opts, path, request_is_https(&req));
        }
    }
    (StatusCode::NOT_FOUND, "not found").into_response()
}

async fn serve_benchmark() -> Json<BenchmarkReport> {
    Json(BenchmarkReport {
        resuma: compressed_asset::asset_sizes()
            .into_iter()
            .map(|(name, raw, gzip, brotli)| BundleSize {
                name: name.to_string(),
                raw,
                gzip,
                brotli,
            })
            .collect(),
        notes: vec![
            "Resuma static pages ship zero JS — no loader, no payload.".into(),
            "Interactive pages load loader.js first; core.js loads on first interaction or when reactive bindings exist.".into(),
            "Compare the same metric: Network transfer size with Content-Encoding enabled.".into(),
        ],
    })
}

#[derive(Debug, Serialize)]
struct BenchmarkReport {
    resuma: Vec<BundleSize>,
    notes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct BundleSize {
    name: String,
    raw: usize,
    gzip: usize,
    brotli: usize,
}

async fn serve_loader(headers: HeaderMap) -> Response {
    serve_js(&headers, loader_asset(), LOADER_JS)
}

async fn serve_core(headers: HeaderMap) -> Response {
    serve_js(&headers, core_asset(), CORE_JS)
}

async fn serve_runtime(headers: HeaderMap) -> Response {
    serve_js(&headers, runtime_asset(), RUNTIME_JS)
}

#[derive(Debug, Deserialize)]
struct ActionRequest {
    args: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct ActionResponse {
    ok: bool,
    value: Option<serde_json::Value>,
    error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    redirect: Option<String>,
}

async fn serve_action(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
    headers: HeaderMap,
    connect: ConnectInfo<SocketAddr>,
    Json(body): Json<ActionRequest>,
) -> Response {
    let cfg = security::config();
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost")
        .to_string();
    let ip = client_ip_from_parts(&headers, Some(connect.0));

    if let Err(err) = guard_mutation(&headers, &host, &ip, "action", cfg.actions_per_minute, None) {
        return action_error(err);
    }

    let flow_req = FlowRequest::from_parts(
        "POST",
        format!("/_resuma/action/{name}"),
        headers
            .iter()
            .filter_map(|(k, v)| {
                v.to_str()
                    .ok()
                    .map(|s| (k.as_str().to_string(), s.to_string()))
            })
            .collect(),
        std::collections::BTreeMap::from([(String::from("name"), name.clone())]),
        std::collections::BTreeMap::new(),
    );

    match dispatch_action(&name, body.args, flow_req).await {
        Ok(value) => {
            let redirect = extract_redirect(&value);
            (
                StatusCode::OK,
                Json(ActionResponse {
                    ok: true,
                    value: Some(value),
                    error: None,
                    redirect,
                }),
            )
                .into_response()
        }
        Err(err) => action_error(err),
    }
}

fn action_error(err: ResumaError) -> Response {
    let cfg = security::config();
    let status = http_status(&err);
    (
        status,
        Json(ActionResponse {
            ok: false,
            value: None,
            error: Some(err.client_message(cfg.production)),
            redirect: None,
        }),
    )
        .into_response()
}

async fn serve_handler_chunk(
    Path(chunk): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let key = chunk.trim_end_matches(".js").to_string();
    match state.handler_chunks.read().get(&key).cloned() {
        Some(src) => {
            let mut res = Response::new(src.into());
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/javascript; charset=utf-8"),
            );
            res
        }
        None => (StatusCode::NOT_FOUND, "handler chunk not found").into_response(),
    }
}

async fn serve_island_refresh(Path(instance): Path<String>) -> Response {
    match super::island_cache::island_refresh_html(&instance) {
        Some(html) => Html(html).into_response(),
        None => (StatusCode::NOT_FOUND, "island instance not found").into_response(),
    }
}

async fn serve_island_chunk(
    Path(chunk): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let key = chunk.trim_end_matches(".js").to_string();
    match state.island_chunks.read().get(&key).cloned() {
        Some(src) => {
            let mut res = Response::new(src.into());
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/javascript; charset=utf-8"),
            );
            res
        }
        None => (StatusCode::NOT_FOUND, "island chunk not found").into_response(),
    }
}
