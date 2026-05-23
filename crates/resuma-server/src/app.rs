//! `ResumaApp` — high-level builder used by example apps & the CLI dev server.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{header, HeaderValue, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use parking_lot::RwLock;
use resuma_core::view::View;
use resuma_ssr::{render_to_string, PageOptions};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::actions::dispatch as dispatch_action;
use crate::runtime_asset::RUNTIME_JS;

/// User-facing builder.
pub struct ResumaApp {
    page_factories: HashMap<String, Arc<PageFactory>>,
    handler_chunks: Arc<RwLock<HashMap<String, String>>>,
    island_chunks: Arc<RwLock<HashMap<String, String>>>,
    page_options: PageOptions,
}

type PageFactory = dyn Fn() -> View + Send + Sync;

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
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.page_options.title = title.into();
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

    /// Register a page route. The factory is invoked on every request — this
    /// matches Qwik's "components only run on the server" mental model and
    /// guarantees a fresh `RenderContext` per request.
    pub fn page<F>(mut self, path: &str, factory: F) -> Self
    where
        F: Fn() -> View + Send + Sync + 'static,
    {
        self.page_factories
            .insert(path.to_string(), Arc::new(factory));
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
        let router = self.into_router();
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
        });

        let mut router = Router::new();
        for path in state.pages.keys() {
            let p = path.clone();
            router = router.route(&p, get(serve_page));
        }

        router
            .route("/_resuma/runtime.js", get(serve_runtime))
            .route("/_resuma/action/:name", post(serve_action))
            .route("/_resuma/handler/:chunk", get(serve_handler_chunk))
            .route("/_resuma/island/:chunk", get(serve_island_chunk))
            .with_state(state)
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

    let opts = state.page_options.clone();
    let html = render_to_string(&opts, move || factory());
    Html(html).into_response()
}

async fn serve_runtime() -> Response {
    let mut res = Response::new(RUNTIME_JS.into());
    res.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/javascript; charset=utf-8"));
    res
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
    Path(name): Path<String>,
    Json(body): Json<ActionRequest>,
) -> Json<ActionResponse> {
    match dispatch_action(&name, body.args).await {
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
