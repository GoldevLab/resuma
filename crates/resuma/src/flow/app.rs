//! [`FlowApp`] — Resuma Flow application builder.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::middleware;

use crate::core::view::View;
use crate::server::ResumaApp;

use super::cache::{loader_cache, merge_cache_control};
use super::errors::{error_page, FlowError};
use super::layout::apply_layouts;
use super::match_route::match_route;
use super::middleware::run_middleware;
use super::pages::{discover_pages, FlowPageRegistry};
use super::request::FlowRequest;
use super::routes::attach_flow_routes;
use super::runtime::{first_load_error, stage_deferred_stream_plan, with_request_deferred};

type PageFn = Arc<dyn Fn(FlowRequest) -> View + Send + Sync>;

#[derive(Clone)]
struct PageEntry {
    handler: PageFn,
    layouts: Vec<String>,
}

/// Full-stack Resuma application with pages, layouts, loads, and submits.
pub struct FlowApp {
    inner: ResumaApp,
    pages: HashMap<String, PageEntry>,
    streaming: bool,
    not_found: Option<Arc<dyn Fn() -> View + Send + Sync>>,
    pwa: Option<super::pwa::FlowPwaConfig>,
}

#[derive(Debug, Clone)]
pub struct FlowServeOptions {
    pub addr: SocketAddr,
    pub security: crate::server::SecurityConfig,
}

impl Default for FlowServeOptions {
    fn default() -> Self {
        Self::from_env()
    }
}

impl FlowServeOptions {
    /// Read bind address from `RESUMA_ADDR` or `HOST` + `PORT` (Fly.io, Docker).
    pub fn from_env() -> Self {
        Self {
            addr: Self::addr_from_env(),
            security: crate::server::SecurityConfig::from_env(),
        }
    }

    fn addr_from_env() -> SocketAddr {
        if let Ok(raw) = std::env::var("RESUMA_ADDR") {
            if let Ok(addr) = raw.parse() {
                return addr;
            }
        }

        let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let port: u16 = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);
        format!("{host}:{port}")
            .parse()
            .unwrap_or_else(|_| SocketAddr::from(([127, 0, 0, 1], port)))
    }
}

impl FlowApp {
    pub fn new() -> Self {
        Self {
            inner: ResumaApp::new(),
            pages: HashMap::new(),
            streaming: false,
            not_found: None,
            pwa: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.inner = self.inner.with_title(title);
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.inner = self.inner.with_description(description);
        self
    }

    pub fn with_site_url(mut self, url: impl Into<String>) -> Self {
        self.inner = self.inner.with_site_url(url);
        self
    }

    pub fn with_og_image(mut self, image: impl Into<String>) -> Self {
        self.inner = self.inner.with_og_image(image);
        self
    }

    pub fn with_json_ld(mut self, json_ld: impl Into<String>) -> Self {
        self.inner = self.inner.with_json_ld(json_ld);
        self
    }

    /// Enable installable PWA (manifest, service worker, icons) for Android/iOS/desktop.
    pub fn with_pwa(mut self, config: super::pwa::FlowPwaConfig) -> Self {
        self.inner = self.inner.with_pwa(config.to_pwa_options());
        self.pwa = Some(config);
        self
    }

    pub fn with_head(mut self, head: impl Into<String>) -> Self {
        self.inner = self.inner.with_head(head);
        self
    }

    pub fn with_stylesheet(mut self, href: impl Into<String>) -> Self {
        self.inner = self.inner.with_stylesheet(href);
        self
    }

    /// Enable chunked streaming SSR (head sent before body completes).
    pub fn streaming(mut self, enabled: bool) -> Self {
        self.streaming = enabled;
        self
    }

    /// Custom 404 page renderer.
    pub fn not_found<F>(mut self, handler: F) -> Self
    where
        F: Fn() -> View + Send + Sync + 'static,
    {
        self.not_found = Some(Arc::new(handler));
        self
    }

    /// Register all pages under `pages_root` using a generated [`FlowPageRegistry`].
    pub fn auto_pages<R>(self, pages_root: impl AsRef<std::path::Path>, registry: R) -> Self
    where
        R: FlowPageRegistry + 'static,
    {
        self.pages_from_registry(pages_root, Arc::new(registry))
    }

    pub fn page<F>(mut self, pattern: &str, handler: F) -> Self
    where
        F: Fn(FlowRequest) -> View + Send + Sync + 'static,
    {
        self.pages.insert(
            pattern.to_string(),
            PageEntry {
                handler: Arc::new(handler),
                layouts: Vec::new(),
            },
        );
        self
    }

    pub fn page_with_layouts<F>(mut self, pattern: &str, layouts: Vec<String>, handler: F) -> Self
    where
        F: Fn(FlowRequest) -> View + Send + Sync + 'static,
    {
        self.pages.insert(
            pattern.to_string(),
            PageEntry {
                handler: Arc::new(handler),
                layouts,
            },
        );
        self
    }

    pub fn pages_from_registry(
        mut self,
        pages_root: impl AsRef<std::path::Path>,
        registry: Arc<dyn FlowPageRegistry>,
    ) -> Self {
        for meta in discover_pages(pages_root) {
            let module = meta.module.clone();
            let layouts = meta.layouts.clone();
            let reg = registry.clone();
            let handler: PageFn = Arc::new(move |req| {
                reg.render(&module, req)
                    .unwrap_or_else(|| View::text(format!("missing page module `{module}`")))
            });
            self.pages
                .insert(meta.pattern.clone(), PageEntry { handler, layouts });
        }
        self
    }

    pub async fn serve(self, opts: FlowServeOptions) -> std::io::Result<()> {
        let mut app = self.inner;
        if self.streaming {
            app = app.with_streaming(true);
        }

        let deferred_streaming = self.streaming;
        let not_found = self.not_found.clone();

        let static_pages: Vec<(String, PageEntry)> = self
            .pages
            .iter()
            .filter(|(pat, _)| !pat.contains(':') && !pat.contains('*'))
            .map(|(pat, f)| (pat.clone(), f.clone()))
            .collect();

        for (pattern, entry) in static_pages {
            let pat = pattern.clone();
            app = app.page(&pattern, move || {
                render_entry(&pat, entry.clone(), deferred_streaming)
            });
        }

        let site_url = std::env::var("SITE_URL").unwrap_or_default();
        let mut paths: Vec<String> = self.pages.keys().cloned().collect();
        paths.sort();

        let dynamic_pages: HashMap<String, PageEntry> = self
            .pages
            .into_iter()
            .filter(|(pat, _)| pat.contains(':') || pat.contains('*'))
            .collect();

        if !dynamic_pages.is_empty() {
            let ds = deferred_streaming;
            app = app.fallback(move |path| {
                dispatch_dynamic(&dynamic_pages, path, ds)
                    .or_else(|| not_found.as_ref().map(|f| f()))
            });
        } else if let Some(nf) = not_found {
            app = app.fallback(move |_path| Some(nf()));
        }

        let mut router = attach_flow_routes(
            app.into_router(),
            super::routes::FlowSeoConfig { site_url, paths },
        );

        if let Some(pwa) = self.pwa {
            router = super::pwa::attach_pwa_routes(router, pwa);
        }

        crate::server::configure_security(opts.security.clone());
        use axum::extract::DefaultBodyLimit;
        let router = router
            .layer(DefaultBodyLimit::max(opts.security.body_limit_bytes))
            .layer(middleware::from_fn(
                crate::server::security_headers_middleware,
            ));
        let listener = tokio::net::TcpListener::bind(opts.addr).await?;
        println!("resuma flow listening on http://{}", opts.addr);
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
    }
}

impl Default for FlowApp {
    fn default() -> Self {
        Self::new()
    }
}

fn render_entry(path: &str, entry: PageEntry, deferred_streaming: bool) -> View {
    let req = FlowRequest {
        path: path.to_string(),
        ..Default::default()
    };
    render_with_flow(req, entry, deferred_streaming)
}

fn dispatch_dynamic(
    pages: &HashMap<String, PageEntry>,
    path: &str,
    deferred_streaming: bool,
) -> Option<View> {
    for (pattern, entry) in pages {
        if let Some(m) = match_route(pattern, path) {
            let req = FlowRequest {
                path: path.to_string(),
                params: m.params,
                ..Default::default()
            };
            return Some(render_with_flow(req, entry.clone(), deferred_streaming));
        }
    }
    None
}

fn render_with_flow(mut req: FlowRequest, entry: PageEntry, deferred_streaming: bool) -> View {
    if let Ok(h) = tokio::runtime::Handle::try_current() {
        let updated = tokio::task::block_in_place(|| h.block_on(run_middleware(req.clone())));
        match updated {
            Ok(r) => req = r,
            Err(e) => return error_page(&FlowError::from_resuma(e)),
        }
    }

    let (view, final_req, deferred) =
        with_request_deferred(req.clone(), deferred_streaming, || {
            let page = (entry.handler)(req.clone());
            if let Some(err) = first_load_error() {
                return error_page(&FlowError::Loader(err));
            }
            apply_layouts(&req, page, &entry.layouts)
        });

    if deferred_streaming && !deferred.is_empty() {
        stage_deferred_stream_plan(deferred.clone(), final_req.clone());
        let mut hints = final_req.cache_control.clone();
        for name in &deferred {
            if let Some(c) = loader_cache(name) {
                hints.insert(name.clone(), c);
            }
        }
        if let Some(cache) = merge_cache_control(&hints) {
            crate::server::stage_response_cache_control(cache);
        }
    } else if let Some(cache) = merge_cache_control(&final_req.cache_control) {
        crate::server::stage_response_cache_control(cache);
    }

    view
}
