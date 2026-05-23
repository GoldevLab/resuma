//! `ResumaApp` — high-level builder used by example apps & the CLI dev server.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderValue, Request, StatusCode, Uri};
use axum::extract::DefaultBodyLimit;
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::extract::ConnectInfo;
use axum::{Json, Router};
use parking_lot::RwLock;
use crate::core::view::View;
use crate::core::{FlowRequest, ResumaError};
use crate::ssr::PageOptions;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::actions::dispatch as dispatch_action;
use super::deferred_stream::try_deferred_stream;
use super::page_cache::take_response_cache_control;
use super::compressed_asset::{self, core_asset, loader_asset, runtime_asset, serve_js};
use super::runtime_asset::{CORE_JS, LOADER_JS, RUNTIME_JS};
use super::security::{
    self, client_ip_from_parts, csrf_set_cookie, csrf_token, guard_mutation, http_status, random_token,
    request_is_https, CspNonce, SecurityConfig, SecurityHeaderOptions,
};

/// User-facing builder.
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

type PageFactory = dyn Fn() -> View + Send + Sync;
type FallbackFactory = dyn Fn(&str) -> Option<View> + Send + Sync;

#[derive(Debug, Clone)]
pub struct ServeOptions {
    pub addr: SocketAddr,
    pub security: SecurityConfig,
}

impl Default for ServeOptions {
    fn default() -> Self {
        Self {
            addr: ([127, 0, 0, 1], 3000).into(),
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

    /// Register a page route. The factory is invoked on every request — components
    /// only run on the server, guaranteeing a fresh `RenderContext` per request.
    pub fn page<F>(mut self, path: &str, factory: F) -> Self
    where
        F: Fn() -> View + Send + Sync + 'static,
    {
        self.page_factories
            .insert(path.to_string(), Arc::new(factory));
        self
    }

    /// Catch-all renderer for dynamic routes (Resuma Flow param patterns).
    pub fn fallback<F>(mut self, factory: F) -> Self
    where
        F: Fn(&str) -> Option<View> + Send + Sync + 'static,
    {
        self.fallback = Some(Arc::new(factory));
        self
    }

    /// Register a precompiled handler chunk to be served at
    /// `/_resuma/handler/<chunk>.js`.
    pub fn handler_chunk(self, chunk_id: &str, source: impl Into<String>) -> Self {
        self.handler_chunks.write().insert(chunk_id.to_string(), source.into());
        self
    }

    /// Register a precompiled island chunk to be served at
    /// `/_resuma/island/<chunk>.js`.
    pub fn island_chunk(self, chunk_id: &str, source: impl Into<String>) -> Self {
        self.island_chunks.write().insert(chunk_id.to_string(), source.into());
        self
    }

    pub async fn serve(self, opts: ServeOptions) -> std::io::Result<()> {
        security::configure(opts.security.clone());
        let router = self
            .into_router()
            .layer(DefaultBodyLimit::max(opts.security.body_limit_bytes))
            .layer(middleware::from_fn(security_headers_middleware));
        let listener = tokio::net::TcpListener::bind(opts.addr).await?;
        info!(addr = %opts.addr, "resuma server listening");
        println!("resuma listening on http://{}", opts.addr);
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
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

        router = router.fallback(get(serve_fallback));

        if !state.hide_benchmark {
            router = router.route("/_resuma/benchmark.json", get(serve_benchmark));
        }

        router
            .route("/_resuma/loader.js", get(serve_loader))
            .route("/_resuma/core.js", get(serve_core))
            .route("/_resuma/runtime.js", get(serve_runtime))
            .route("/_resuma/action/:name", post(serve_action))
            .route("/_resuma/handler/:chunk", get(serve_handler_chunk))
            .route("/_resuma/island/:chunk", get(serve_island_chunk))
            .with_state(state)
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
    fn default() -> Self { Self::new() }
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
    res.extensions_mut().insert(CspNonce(opts.csp_nonce.clone()));
    res
}

fn render_page_response(state: &AppState, view: View, path: &str, https: bool) -> Response {
    let opts = page_security_opts(&state.page_options);
    super::page_cache::stage_page_csrf(opts.csrf_token.clone());
    let cache = take_response_cache_control();
    if state.streaming {
        use axum::body::Body;
        use futures_util::StreamExt;

        let stream = if let Some(deferred) = try_deferred_stream(view.clone(), &opts, path) {
            deferred
        } else {
            use crate::ssr::render_to_stream;
            render_to_stream(&opts, path, move || view)
        };

        let stream = stream.map(|chunk| {
            chunk.map(axum::body::Bytes::from).map_err(std::io::Error::other)
        });
        let mut builder = Response::builder()
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::TRANSFER_ENCODING, "chunked");
        if let Some(ref cache) = cache {
            builder = builder.header(header::CACHE_CONTROL, cache.as_str());
        }
        let res = builder
            .header("x-robots-tag", "index, follow")
            .body(Body::from_stream(stream))
            .unwrap();
        attach_page_security(res, &opts, https)
    } else {
        let html = crate::ssr::render_to_string_at_path(&opts, path, move || view);
        let mut res = Html(html).into_response();
        if let Some(cache) = cache {
            res.headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_str(&cache).unwrap_or_else(|_| HeaderValue::from_static("no-store")));
        }
        res.headers_mut().insert(
            header::HeaderName::from_static("x-robots-tag"),
            HeaderValue::from_static("index, follow"),
        );
        attach_page_security(res, &opts, https)
    }
}

async fn serve_page(
    uri: Uri,
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> Response {
    let path = uri.path().to_string();
    let factory = match state.pages.get(&path) {
        Some(f) => f.clone(),
        None => return (StatusCode::NOT_FOUND, "not found").into_response(),
    };

    render_page_response(&state, factory(), &path, request_is_https(&req))
}

async fn serve_fallback(
    uri: Uri,
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> Response {
    let path = uri.path();
    if let Some(fb) = &state.fallback {
        if let Some(view) = fb(path) {
            return render_page_response(&state, view, path, request_is_https(&req));
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
        qwik_reference: vec![
            BundleSize {
                name: "qwikloader (docs + 2024 opts)".into(),
                raw: 1024,
                gzip: 2499,
                brotli: 1434,
            },
            BundleSize {
                name: "handler chunk (on demand)".into(),
                raw: 0,
                gzip: 0,
                brotli: 0,
            },
        ],
        notes: vec![
            "Resuma static pages ship zero JS — no loader, no payload.".into(),
            "Interactive pages load loader.js first; core.js loads on first interaction or when reactive bindings exist.".into(),
            "Qwik reference numbers are from qwik.dev/docs and Qwik PR #7519 (gzip ~2.44 KB qwikloader).".into(),
            "Compare the same metric: Network transfer size with Content-Encoding enabled.".into(),
        ],
    })
}

#[derive(Debug, Serialize)]
struct BenchmarkReport {
    resuma: Vec<BundleSize>,
    qwik_reference: Vec<BundleSize>,
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
            .filter_map(|(k, v)| v.to_str().ok().map(|s| (k.as_str().to_string(), s.to_string())))
            .collect(),
        std::collections::BTreeMap::from([(String::from("name"), name.clone())]),
        std::collections::BTreeMap::new(),
    );

    match dispatch_action(&name, body.args, flow_req).await {
        Ok(value) => (
            StatusCode::OK,
            Json(ActionResponse {
                ok: true,
                value: Some(value),
                error: None,
            }),
        )
            .into_response(),
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
            res.headers_mut()
                .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/javascript; charset=utf-8"));
            res
        }
        None => (StatusCode::NOT_FOUND, "handler chunk not found").into_response(),
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
            res.headers_mut()
                .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/javascript; charset=utf-8"));
            res
        }
        None => (StatusCode::NOT_FOUND, "island chunk not found").into_response(),
    }
}
