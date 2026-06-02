//! [`FlowApp`] — Resuma Flow application builder.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::middleware;
use axum::routing::get;

use crate::core::view::View;
use crate::core::Component;
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

/// Full-stack application: pages, layouts, server loads, form submits, and middleware.
///
/// Wraps [`ResumaApp`]. Call [`serve`](Self::serve) with
/// [`FlowServeOptions::from_env`] on Fly.io, Docker, or local dev.
pub struct FlowApp {
    inner: ResumaApp,
    pages: HashMap<String, PageEntry>,
    streaming: bool,
    not_found: Option<Arc<dyn Fn() -> View + Send + Sync>>,
    pwa: Option<super::pwa::FlowPwaConfig>,
    /// When true, skip auto PWA even if `RESUMA_PWA` is enabled.
    pwa_disabled: bool,
    extensions: super::extensions::FlowExtensions,
    /// Extra GET routes (e.g. bundled JS/CSS for marketing pages).
    static_assets: Vec<(String, &'static [u8], &'static str)>,
    /// Optional `public/` directory (defaults to `{CARGO_MANIFEST_DIR}/public`).
    public_dir: Option<PathBuf>,
    /// When set, merges [`Theme`](crate::Theme) colors into auto PWA config.
    theme_for_pwa: Option<crate::Theme>,
}

/// Listen and security options for [`FlowApp::serve`].
///
/// [`Default`] delegates to [`Self::from_env`] (`RESUMA_ADDR` or `HOST` + `PORT`).
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
        crate::server::listen::listen_addr_from_env()
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
            pwa_disabled: false,
            extensions: super::extensions::FlowExtensions::default(),
            static_assets: Vec::new(),
            public_dir: None,
            theme_for_pwa: None,
        }
    }

    /// Serve files from `public/` at their URL paths (e.g. `public/images/a.jpg` → `/images/a.jpg`).
    ///
    /// Defaults to `{CARGO_MANIFEST_DIR}/public` when not called explicitly.
    pub fn with_public_dir(mut self, dir: impl AsRef<std::path::Path>) -> Self {
        self.public_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Apply [`Theme`] primary/background to the auto-generated PWA manifest.
    pub fn with_theme_pwa(mut self, theme: crate::Theme) -> Self {
        self.theme_for_pwa = Some(theme);
        self
    }

    /// Serve a fixed byte slice at `path` (must start with `/`).
    pub fn static_asset(
        mut self,
        path: impl Into<String>,
        body: &'static [u8],
        content_type: &'static str,
    ) -> Self {
        self.static_assets.push((path.into(), body, content_type));
        self
    }

    /// Register a TypeScript client component bundle at `/static/client/{id}.js`.
    pub fn client_asset(self, id: impl AsRef<str>, body: &'static [u8]) -> Self {
        let path = crate::client::client_script_url(id.as_ref());
        self.static_asset(path, body, "application/javascript; charset=utf-8")
    }

    /// Attach a JSON-serializable value to every request (`req.extension("key")` in loads/submits).
    pub fn with_extension(mut self, key: impl Into<String>, value: impl serde::Serialize) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.extensions.insert(key, v);
        }
        self
    }

    /// Merge a map of extensions into every request (e.g. `"db": "ready"` marker after pool init).
    pub fn with_extensions(mut self, extensions: super::extensions::FlowExtensions) -> Self {
        for (k, v) in extensions.0 {
            self.extensions.insert(k, v);
        }
        self
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

    /// Configure installable PWA (manifest, service worker, icons). Overrides auto defaults.
    pub fn with_pwa(mut self, config: super::pwa::FlowPwaConfig) -> Self {
        self.inner = self.inner.with_pwa(config.to_pwa_options());
        self.pwa = Some(config);
        self
    }

    /// Disable PWA for this app (`RESUMA_PWA=0` also disables globally).
    pub fn without_pwa(mut self) -> Self {
        self.pwa_disabled = true;
        self.pwa = None;
        self.inner.page_options_mut().pwa = None;
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

    /// Register a no-props component page without spelling
    /// `Component::render(ComponentProps::default())`.
    pub fn component<C>(self, pattern: &str, _component: C) -> Self
    where
        C: Component + 'static,
        C::Props: Default,
    {
        self.page(pattern, |_req| C::render(Default::default()))
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
        crate::server::configure_security(opts.security.clone());
        let router = self.into_router(opts.clone());
        let (listener, bound) = crate::server::listen::bind_listener(opts.addr).await?;
        tracing::info!(addr = %bound, "resuma flow listening");
        println!("resuma flow listening on http://{}", bound);
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(crate::server::shutdown_signal())
        .await
    }

    /// Build the axum router (pages, Flow routes, static assets, security layers).
    pub fn into_router(mut self, opts: FlowServeOptions) -> axum::Router {
        let mut paths: Vec<String> = self.pages.keys().cloned().collect();
        paths.sort();

        let public_dir = self
            .public_dir
            .clone()
            .unwrap_or_else(super::public::default_public_dir);
        let public_assets = super::public::collect_public_dir(&public_dir);

        if let Some(mut cfg) = self.resolve_pwa_config(&paths) {
            merge_route_precache(&mut cfg, &paths);
            for asset in &public_assets {
                if !cfg.precache_paths.contains(&asset.url_path) {
                    cfg.precache_paths.push(asset.url_path.clone());
                }
            }
            if let Some(theme) = &self.theme_for_pwa {
                super::nav::theme_into_pwa(theme, &mut cfg);
            }
            cfg.manifest_icons = super::pwa::manifest_icons_from_public(&public_assets);
            self.inner.page_options_mut().pwa = Some(cfg.to_pwa_options());
            self.pwa = Some(cfg);
        }

        let mut app = self.inner;
        if self.streaming {
            app = app.with_streaming(true);
        }

        let deferred_streaming = self.streaming;
        let not_found = self.not_found.clone();
        let global_extensions = self.extensions.clone();
        let pwa = self.pwa.clone();

        let static_pages: Vec<(String, PageEntry)> = self
            .pages
            .iter()
            .filter(|(pat, _)| !pat.contains(':') && !pat.contains('*'))
            .map(|(pat, f)| (pat.clone(), f.clone()))
            .collect();

        for (pattern, entry) in static_pages {
            let ext = global_extensions.clone();
            app = app.page_with_request(&pattern, move |req| {
                render_with_flow(req, entry.clone(), deferred_streaming, ext.clone())
            });
        }

        let site_url = std::env::var("SITE_URL").unwrap_or_default();

        let dynamic_pages: HashMap<String, PageEntry> = self
            .pages
            .into_iter()
            .filter(|(pat, _)| pat.contains(':') || pat.contains('*'))
            .collect();

        if !dynamic_pages.is_empty() {
            let ds = deferred_streaming;
            let ext = global_extensions.clone();
            app = app.fallback_with_request(move |path, req| {
                dispatch_dynamic(&dynamic_pages, path, req, ds, ext.clone())
                    .or_else(|| not_found.as_ref().map(|f| f()))
            });
        } else if let Some(nf) = not_found {
            app = app.fallback(move |_path| Some(nf()));
        }

        let mut router = attach_flow_routes(
            app.into_router(),
            super::routes::FlowSeoConfig { site_url, paths },
        );

        if let Some(pwa) = pwa {
            router = super::pwa::attach_pwa_routes(router, pwa);
        }

        for (path, body, content_type) in self.static_assets {
            router = router.route(
                &path,
                get(move || async move {
                    crate::server::static_assets::static_asset_response(content_type, body)
                }),
            );
        }

        for asset in public_assets {
            let path = asset.url_path.clone();
            let body = asset.body.clone();
            let ct = asset.content_type.clone();
            router = router.route(
                &path,
                get(move || {
                    let body = body.clone();
                    let ct = ct.clone();
                    async move { public_asset_response(&ct, &body) }
                }),
            );
        }

        use axum::extract::DefaultBodyLimit;
        router
            .layer(DefaultBodyLimit::max(opts.security.body_limit_bytes))
            .layer(middleware::from_fn(
                crate::server::security_headers_middleware,
            ))
            .layer(middleware::from_fn(crate::server::request_id_middleware))
    }
}

impl Default for FlowApp {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowApp {
    fn resolve_pwa_config(&self, routes: &[String]) -> Option<super::pwa::FlowPwaConfig> {
        if self.pwa_disabled || !super::pwa::pwa_enabled_by_default() {
            return None;
        }
        if let Some(cfg) = &self.pwa {
            return Some(cfg.clone());
        }
        let opts = self.inner.page_options();
        Some(super::pwa::FlowPwaConfig::from_page_options(
            &opts.title,
            &opts.description,
            &opts.lang,
            routes,
        ))
    }
}

fn public_asset_response(
    content_type: &str,
    body: &[u8],
) -> (
    [(axum::http::header::HeaderName, axum::http::HeaderValue); 2],
    Vec<u8>,
) {
    use axum::http::{header, HeaderValue};
    let ct = HeaderValue::from_str(content_type)
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
    let cache = if content_type.starts_with("image/") || content_type.contains("font/") {
        crate::server::static_assets::STATIC_IMMUTABLE_CACHE
    } else {
        "public, max-age=3600"
    };
    (
        [
            (header::CONTENT_TYPE, ct),
            (
                header::CACHE_CONTROL,
                axum::http::HeaderValue::from_static(cache),
            ),
        ],
        body.to_vec(),
    )
}

fn merge_route_precache(cfg: &mut super::pwa::FlowPwaConfig, routes: &[String]) {
    for path in super::pwa::default_precache_paths(routes) {
        if !cfg.precache_paths.contains(&path) {
            cfg.precache_paths.push(path);
        }
    }
}

fn dispatch_dynamic(
    pages: &HashMap<String, PageEntry>,
    path: &str,
    mut req: FlowRequest,
    deferred_streaming: bool,
    extensions: super::extensions::FlowExtensions,
) -> Option<View> {
    for (pattern, entry) in pages {
        if let Some(m) = match_route(pattern, path) {
            req.path = path.to_string();
            req.params = m.params;
            return Some(render_with_flow(
                req,
                entry.clone(),
                deferred_streaming,
                extensions,
            ));
        }
    }
    None
}

fn render_with_flow(
    mut req: FlowRequest,
    entry: PageEntry,
    deferred_streaming: bool,
    extensions: super::extensions::FlowExtensions,
) -> View {
    extensions.merge_into(&mut req);
    if let Ok(h) = tokio::runtime::Handle::try_current() {
        let updated = tokio::task::block_in_place(|| h.block_on(run_middleware(req.clone())));
        match updated {
            Ok(r) => req = r,
            Err(e) => return error_page(&FlowError::from_resuma(e)),
        }
    }

    let (view, final_req, deferred) =
        with_request_deferred(req.clone(), deferred_streaming, || {
            // Render may panic if a page uses the panicking `use_*_load()` accessor
            // on a failed loader. Catch it so the connection survives and the proper
            // error page is served (loaders record their error before panicking).
            let rendered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                (entry.handler)(req.clone())
            }));
            match rendered {
                Ok(page) => {
                    if let Some(err) = first_load_error() {
                        return error_page(&FlowError::Loader(err));
                    }
                    apply_layouts(&req, page, &entry.layouts)
                }
                Err(_) => match first_load_error() {
                    Some(err) => error_page(&FlowError::Loader(err)),
                    None => error_page(&FlowError::Render("page render failed".into())),
                },
            }
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
