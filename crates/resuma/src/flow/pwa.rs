//! PWA routes — manifest, service worker, icons, offline fallback.
//!
//! Inspired by Qwik's `@qwikdev/pwa` (Workbox-style precache + manifest), but built into
//! the Resuma server so every [`FlowApp`] can ship installable PWAs without Vite plugins.
//!
//! **Default:** [`FlowApp::into_router`](super::app::FlowApp::into_router) enables PWA
//! automatically from page title/description and static routes. Opt out with
//! [`FlowApp::without_pwa`](super::app::FlowApp::without_pwa) or `RESUMA_PWA=0`.

use axum::http::header;
use axum::Router;
use serde::Serialize;

/// Shortcut entry in `manifest.webmanifest`.
#[derive(Debug, Clone, Serialize)]
pub struct PwaShortcut {
    pub name: String,
    pub short_name: String,
    pub url: String,
}

/// Progressive Web App configuration for [`crate::FlowApp::with_pwa`].
#[derive(Debug, Clone, Serialize)]
pub struct FlowPwaConfig {
    pub name: String,
    pub short_name: String,
    pub description: String,
    pub theme_color: String,
    pub background_color: String,
    pub start_url: String,
    pub scope: String,
    /// Bump to invalidate service-worker caches after deploys.
    pub cache_version: String,
    pub display: String,
    pub orientation: String,
    pub lang: String,
    /// Letter drawn on generated SVG icons (defaults to first char of `short_name`).
    pub icon_char: Option<String>,
    /// Extra static paths to precache (app routes are added automatically).
    pub precache_paths: Vec<String>,
    pub shortcuts: Vec<PwaShortcut>,
    pub offline_title: String,
    pub offline_message: String,
    /// PNG/SVG icons from `public/` (overrides generated SVG manifest entries when set).
    #[serde(skip)]
    pub manifest_icons: Vec<ManifestIconEntry>,
}

/// Manifest icon entry (static SVG defaults or files from `public/`).
#[derive(Debug, Clone, Serialize)]
pub struct ManifestIconEntry {
    pub src: String,
    pub sizes: String,
    #[serde(rename = "type")]
    pub mime: String,
    pub purpose: String,
}

impl Default for FlowPwaConfig {
    fn default() -> Self {
        Self {
            name: "Resuma App".into(),
            short_name: "Resuma".into(),
            description: String::new(),
            theme_color: "#6366f1".into(),
            background_color: "#0f0a1a".into(),
            start_url: "/".into(),
            scope: "/".into(),
            cache_version: "1".into(),
            display: "standalone".into(),
            orientation: "any".into(),
            lang: "es".into(),
            icon_char: None,
            precache_paths: Vec::new(),
            shortcuts: vec![PwaShortcut {
                name: "Inicio".into(),
                short_name: "Inicio".into(),
                url: "/".into(),
            }],
            offline_title: "Sin conexión".into(),
            offline_message: "No hay red. Vuelve a intentarlo cuando tengas conexión.".into(),
            manifest_icons: Vec::new(),
        }
    }
}

/// Detect install icons shipped in `public/` (see [`super::public::PWA_ICON_CANDIDATES`]).
pub fn manifest_icons_from_public(assets: &[super::public::PublicAsset]) -> Vec<ManifestIconEntry> {
    let mut icons = Vec::new();
    for (rel, url, sizes) in super::public::PWA_ICON_CANDIDATES {
        let Some(asset) = assets
            .iter()
            .find(|a| a.url_path == *url || a.url_path == format!("/{rel}"))
        else {
            continue;
        };
        let purpose = if url.contains("maskable") {
            "maskable"
        } else {
            "any"
        };
        icons.push(ManifestIconEntry {
            src: url.to_string(),
            sizes: sizes.to_string(),
            mime: asset.content_type.clone(),
            purpose: purpose.to_string(),
        });
    }
    icons
}

impl FlowPwaConfig {
    /// Build config from [`crate::ssr::PageOptions`] and discovered static routes.
    pub fn from_page_options(
        title: &str,
        description: &str,
        lang: &str,
        routes: &[String],
    ) -> Self {
        let short_name = short_name_from_title(title);
        let theme = std::env::var("RESUMA_PWA_THEME_COLOR")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "#6366f1".into());
        let background = std::env::var("RESUMA_PWA_BACKGROUND_COLOR")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "#0f0a1a".into());

        Self {
            name: title.to_string(),
            short_name,
            description: description.to_string(),
            theme_color: theme,
            background_color: background,
            lang: lang.to_string(),
            precache_paths: default_precache_paths(routes),
            ..Self::default()
        }
    }

    pub fn theme(
        mut self,
        theme_color: impl Into<String>,
        background_color: impl Into<String>,
    ) -> Self {
        self.theme_color = theme_color.into();
        self.background_color = background_color.into();
        self
    }

    pub fn shortcut(
        mut self,
        name: impl Into<String>,
        short_name: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        self.shortcuts.push(PwaShortcut {
            name: name.into(),
            short_name: short_name.into(),
            url: url.into(),
        });
        self
    }

    pub fn precache_path(mut self, path: impl Into<String>) -> Self {
        self.precache_paths.push(path.into());
        self
    }

    pub fn cache_version(mut self, version: impl Into<String>) -> Self {
        self.cache_version = version.into();
        self
    }

    pub fn icon_char(mut self, ch: impl Into<String>) -> Self {
        self.icon_char = Some(ch.into());
        self
    }

    pub fn to_pwa_options(&self) -> crate::ssr::PwaOptions {
        crate::ssr::PwaOptions {
            enabled: true,
            name: self.name.clone(),
            short_name: self.short_name.clone(),
            description: self.description.clone(),
            theme_color: self.theme_color.clone(),
            background_color: self.background_color.clone(),
        }
    }

    fn icon_letter(&self) -> char {
        self.icon_char
            .as_ref()
            .and_then(|s| s.chars().next())
            .or_else(|| self.short_name.chars().next())
            .unwrap_or('R')
    }

    fn precache_list(&self) -> Vec<String> {
        let mut paths = vec![
            self.start_url.clone(),
            "/offline.html".into(),
            "/manifest.webmanifest".into(),
            "/pwa-register.js".into(),
            "/favicon.svg".into(),
            "/icons/icon-192.svg".into(),
            "/icons/icon-512.svg".into(),
            "/icons/icon-maskable.svg".into(),
            "/icons/apple-touch-icon.svg".into(),
            "/_resuma/loader.js".into(),
            "/_resuma/core.js".into(),
        ];
        for p in &self.precache_paths {
            if !paths.contains(p) {
                paths.push(p.clone());
            }
        }
        paths
    }
}

pub fn pwa_enabled_by_default() -> bool {
    !matches!(
        std::env::var("RESUMA_PWA").as_deref(),
        Ok("0") | Ok("false") | Ok("FALSE") | Ok("off")
    )
}

fn short_name_from_title(title: &str) -> String {
    let base = title.split('·').next().unwrap_or(title).trim();
    let s: String = base.chars().take(14).collect();
    if s.is_empty() {
        "App".into()
    } else {
        s
    }
}

pub(crate) fn default_precache_paths(routes: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for r in routes {
        if r.contains(':') || r.contains('*') {
            continue;
        }
        if !out.contains(r) {
            out.push(r.clone());
        }
    }
    out
}

#[derive(Serialize)]
struct WebManifest<'a> {
    name: &'a str,
    short_name: &'a str,
    description: &'a str,
    start_url: &'a str,
    scope: &'a str,
    display: &'a str,
    orientation: &'a str,
    theme_color: &'a str,
    background_color: &'a str,
    lang: &'a str,
    dir: &'static str,
    categories: &'static [&'static str],
    icons: Vec<ManifestIconEntry>,
    shortcuts: Vec<PwaShortcut>,
}

fn default_svg_icons() -> Vec<ManifestIconEntry> {
    vec![
        ManifestIconEntry {
            src: "/icons/icon-192.svg".into(),
            sizes: "192x192".into(),
            mime: "image/svg+xml".into(),
            purpose: "any".into(),
        },
        ManifestIconEntry {
            src: "/icons/icon-512.svg".into(),
            sizes: "512x512".into(),
            mime: "image/svg+xml".into(),
            purpose: "any".into(),
        },
        ManifestIconEntry {
            src: "/icons/icon-maskable.svg".into(),
            sizes: "512x512".into(),
            mime: "image/svg+xml".into(),
            purpose: "maskable".into(),
        },
        ManifestIconEntry {
            src: "/icons/apple-touch-icon.svg".into(),
            sizes: "180x180".into(),
            mime: "image/svg+xml".into(),
            purpose: "any".into(),
        },
    ]
}

fn icon_svg(cfg: &FlowPwaConfig, size: u32, maskable: bool) -> String {
    let pad = if maskable { size / 5 } else { 0 };
    let inner = size.saturating_sub(pad * 2);
    let rx = if maskable { inner / 8 } else { inner / 6 };
    let font_size = inner / 2;
    let y = pad + inner * 2 / 3;
    let cx = pad + inner / 2;
    let letter = cfg.icon_letter();
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}" role="img" aria-label="{label}">
<rect x="{pad}" y="{pad}" width="{inner}" height="{inner}" rx="{rx}" fill="{fill}"/>
<text x="{cx}" y="{y}" text-anchor="middle" fill="#ffffff" font-family="Segoe UI, system-ui, sans-serif" font-size="{font_size}" font-weight="700">{letter}</text>
</svg>"##,
        label = escape_xml(&cfg.short_name),
        fill = escape_xml(&cfg.theme_color),
        letter = escape_xml(&letter.to_string()),
    )
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn manifest_json(cfg: &FlowPwaConfig) -> String {
    let manifest = WebManifest {
        name: &cfg.name,
        short_name: &cfg.short_name,
        description: &cfg.description,
        start_url: &cfg.start_url,
        scope: &cfg.scope,
        display: &cfg.display,
        orientation: &cfg.orientation,
        theme_color: &cfg.theme_color,
        background_color: &cfg.background_color,
        lang: &cfg.lang,
        dir: "ltr",
        categories: &["lifestyle", "utilities"],
        icons: if cfg.manifest_icons.is_empty() {
            default_svg_icons()
        } else {
            cfg.manifest_icons.clone()
        },
        shortcuts: cfg.shortcuts.clone(),
    };
    serde_json::to_string_pretty(&manifest).unwrap_or_else(|_| "{}".into())
}

fn service_worker(cfg: &FlowPwaConfig) -> String {
    // Serialize via serde_json so cache_version / precache paths cannot break
    // out of the JS string literals (JS injection into service-worker scope).
    let cache_name = format!("resuma-pwa-{}", cfg.cache_version);
    let cache_name_js =
        serde_json::to_string(&cache_name).unwrap_or_else(|_| "\"resuma-pwa\"".into());
    let precache: Vec<String> = cfg
        .precache_list()
        .into_iter()
        .map(|p| p.replace('\\', "/"))
        .collect();
    let precache_js = serde_json::to_string(&precache).unwrap_or_else(|_| "[]".into());

    format!(
        r#"const CACHE = {cache_name_js};
const PRECACHE = {precache_js};

self.addEventListener("install", (event) => {{
  event.waitUntil(
    caches.open(CACHE).then((cache) => cache.addAll(PRECACHE)).then(() => self.skipWaiting())
  );
}});

self.addEventListener("activate", (event) => {{
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k)))
    ).then(() => self.clients.claim())
  );
}});

self.addEventListener("fetch", (event) => {{
  const req = event.request;
  const url = new URL(req.url);

  if (req.method !== "GET" || url.origin !== self.location.origin) {{
    return;
  }}

  if (req.mode === "navigate") {{
    event.respondWith(
      fetch(req)
        .then((res) => {{
          const copy = res.clone();
          caches.open(CACHE).then((cache) => cache.put(req, copy));
          return res;
        }})
        .catch(() =>
          caches.match(req).then((cached) => cached || caches.match("/offline.html"))
        )
    );
    return;
  }}

  if (
    url.pathname.startsWith("/_resuma/") ||
    url.pathname.startsWith("/icons/") ||
    url.pathname.startsWith("/images/") ||
    url.pathname.endsWith(".svg") ||
    url.pathname.endsWith(".webmanifest") ||
    url.pathname.endsWith(".js") ||
    url.pathname.endsWith(".jpg") ||
    url.pathname.endsWith(".png")
  ) {{
    event.respondWith(
      caches.match(req).then((cached) => {{
        const network = fetch(req).then((res) => {{
          const copy = res.clone();
          caches.open(CACHE).then((cache) => cache.put(req, copy));
          return res;
        }});
        return cached || network;
      }})
    );
  }}
}});
"#,
        cache_name_js = cache_name_js,
        precache_js = precache_js,
    )
}

const PWA_REGISTER_JS: &str = r#""use strict";
if ("serviceWorker" in navigator) {
  window.addEventListener("load", () => {
    navigator.serviceWorker.register("/sw.js", { scope: "/" }).catch(() => {});
  });
}
"#;

fn offline_html(cfg: &FlowPwaConfig) -> String {
    format!(
        r##"<!doctype html>
<html lang="{lang}">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<meta name="theme-color" content="{theme}" />
<title>{title}</title>
<style>
  body {{ margin: 0; min-height: 100vh; display: grid; place-items: center; font-family: system-ui, sans-serif;
    background: {bg}; color: #f5f5f5; text-align: center; padding: 2rem; }}
  h1 {{ margin-bottom: .5rem; }}
  p {{ opacity: .85; max-width: 28rem; line-height: 1.55; }}
  a {{ color: {theme}; }}
</style>
</head>
<body>
  <main>
    <h1>{title}</h1>
    <p>{message}</p>
    <p><a href="{start}">Volver al inicio</a></p>
  </main>
</body>
</html>"##,
        lang = escape_xml(&cfg.lang),
        theme = escape_xml(&cfg.theme_color),
        bg = escape_xml(&cfg.background_color),
        title = escape_xml(&cfg.offline_title),
        message = escape_xml(&cfg.offline_message),
        start = escape_xml(&cfg.start_url),
    )
}

fn js_headers() -> [(header::HeaderName, &'static str); 2] {
    [
        (
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        ),
        (header::CACHE_CONTROL, "no-cache"),
    ]
}

pub fn attach_pwa_routes(router: Router, cfg: FlowPwaConfig) -> Router {
    let manifest_cfg = cfg.clone();
    let sw_cfg = cfg.clone();
    let offline_cfg = cfg.clone();
    let icon192 = cfg.clone();
    let icon512 = cfg.clone();
    let icon_mask = cfg.clone();
    let icon_apple = cfg.clone();

    router
        .route(
            "/manifest.webmanifest",
            axum::routing::get(move || {
                let body = manifest_json(&manifest_cfg);
                async move {
                    (
                        [(
                            header::CONTENT_TYPE,
                            "application/manifest+json; charset=utf-8",
                        )],
                        body,
                    )
                }
            }),
        )
        .route(
            "/sw.js",
            axum::routing::get(move || {
                let body = service_worker(&sw_cfg);
                async move { (js_headers(), body) }
            }),
        )
        .route(
            "/pwa-register.js",
            axum::routing::get(|| async move { (js_headers(), PWA_REGISTER_JS) }),
        )
        .route(
            "/offline.html",
            axum::routing::get(move || {
                let body = offline_html(&offline_cfg);
                async move { ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], body) }
            }),
        )
        .route(
            "/icons/icon-192.svg",
            axum::routing::get(move || {
                let body = icon_svg(&icon192, 192, false);
                async move {
                    (
                        [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
                        body,
                    )
                }
            }),
        )
        .route(
            "/icons/icon-512.svg",
            axum::routing::get(move || {
                let body = icon_svg(&icon512, 512, false);
                async move {
                    (
                        [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
                        body,
                    )
                }
            }),
        )
        .route(
            "/icons/icon-maskable.svg",
            axum::routing::get(move || {
                let body = icon_svg(&icon_mask, 512, true);
                async move {
                    (
                        [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
                        body,
                    )
                }
            }),
        )
        .route(
            "/icons/apple-touch-icon.svg",
            axum::routing::get(move || {
                let body = icon_svg(&icon_apple, 180, false);
                async move {
                    (
                        [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
                        body,
                    )
                }
            }),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_includes_app_routes() {
        let cfg = FlowPwaConfig::from_page_options(
            "Estudio Lama · Barbería",
            "Reservas online",
            "es",
            &["/".into(), "/servicios".into(), "/reservar".into()],
        );
        let json = manifest_json(&cfg);
        assert!(json.contains("Estudio Lama"));
        assert!(json.contains("standalone"));
    }

    #[test]
    fn service_worker_precaches_static_routes() {
        let cfg = FlowPwaConfig::from_page_options("App", "", "en", &["/about".into()]);
        let sw = service_worker(&cfg);
        assert!(sw.contains("\"/about\""));
        assert!(sw.contains("/_resuma/loader.js"));
    }
}
