//! `ResumaApp` — high-level builder used by example apps & the CLI dev server.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderValue, Request, StatusCode, Uri};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use parking_lot::RwLock;
use resuma_core::view::View;
use resuma_core::FlowRequest;
use resuma_ssr::PageOptions;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::actions::dispatch as dispatch_action;
use crate::deferred_stream::try_deferred_stream;
use crate::page_cache::take_response_cache_control;
use crate::compressed_asset::{self, core_asset, loader_asset, runtime_asset, serve_js};
use crate::runtime_asset::{CORE_JS, LOADER_JS, RUNTIME_JS};

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
}

impl Default for ServeOptions {
    fn default() -> Self {
        Self {
            addr: ([127, 0, 0, 1], 3000).into(),
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
        let router = self
            .into_router()
            .layer(middleware::from_fn(security_headers_middleware));
        let listener = tokio::net::TcpListener::bind(opts.addr).await?;
        info!(addr = %opts.addr, "resuma server listening");
        println!("resuma listening on http://{}", opts.addr);
        axum::serve(listener, router).await
    }

    pub fn into_router(self) -> Router {
        let state = Arc::new(AppState {
            pages: self.page_factories,
            handler_chunks: self.handler_chunks,
            island_chunks: self.island_chunks,
            page_options: self.page_options,
            streaming: self.streaming,
            fallback: self.fallback,
        });

        let mut router = Router::new();
        for path in state.pages.keys() {
            let p = path.clone();
            router = router.route(&p, get(serve_page));
        }

        router = router.fallback(get(serve_fallback));

        router
            .route("/_resuma/benchmark.json", get(serve_benchmark))
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
pub fn apply_security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    insert_header(headers, header::STRICT_TRANSPORT_SECURITY, "max-age=31536000; includeSubDomains");
    insert_header(headers, header::X_FRAME_OPTIONS, "DENY");
    insert_header(headers, header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    insert_header(headers, header::REFERRER_POLICY, "strict-origin-when-cross-origin");
    insert_header(headers, header::HeaderName::from_static("permissions-policy"), "camera=(), microphone=(), geolocation=()");
    insert_header(
        headers,
        header::CONTENT_SECURITY_POLICY,
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'",
    );
    response
}

pub async fn security_headers_middleware(req: Request<Body>, next: Next) -> Response {
    apply_security_headers(next.run(req).await)
}

fn insert_header(headers: &mut axum::http::HeaderMap, name: header::HeaderName, value: &str) {
    if let Ok(v) = HeaderValue::from_str(value) {
        headers.insert(name, v);
    }
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
}

fn render_page_response(state: &AppState, view: View, path: &str) -> Response {
    let opts = state.page_options.clone();
    let cache = take_response_cache_control();
    if state.streaming {
        use axum::body::Body;
        use futures_util::StreamExt;

        let stream = if let Some(deferred) = try_deferred_stream(view.clone(), &opts, path) {
            deferred
        } else {
            use resuma_ssr::render_to_stream;
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
        builder.body(Body::from_stream(stream)).unwrap()
    } else {
        let html = resuma_ssr::render_to_string_at_path(&opts, path, move || view);
        let mut res = Html(html).into_response();
        if let Some(cache) = cache {
            res.headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_str(&cache).unwrap_or_else(|_| HeaderValue::from_static("no-store")));
        }
        res
    }
}

async fn serve_page(
    uri: Uri,
    State(state): State<Arc<AppState>>,
) -> Response {
    let path = uri.path().to_string();
    let factory = match state.pages.get(&path) {
        Some(f) => f.clone(),
        None => return (StatusCode::NOT_FOUND, "not found").into_response(),
    };

    render_page_response(&state, factory(), &path)
}

async fn serve_fallback(
    uri: Uri,
    State(state): State<Arc<AppState>>,
) -> Response {
    let path = uri.path();
    if let Some(fb) = &state.fallback {
        if let Some(view) = fb(path) {
            return render_page_response(&state, view, path);
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
    Json(body): Json<ActionRequest>,
) -> Json<ActionResponse> {
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
        Ok(value) => Json(ActionResponse { ok: true, value: Some(value), error: None }),
        Err(err) => Json(ActionResponse { ok: false, value: None, error: Some(err.to_string()) }),
    }
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
