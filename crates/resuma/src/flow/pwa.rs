//! PWA routes — manifest, service worker, icons, offline fallback.

use axum::http::header;
use axum::Router;
use serde::Serialize;

/// Progressive Web App configuration for [`FlowApp::with_pwa`].
#[derive(Debug, Clone, Serialize)]
pub struct FlowPwaConfig {
    pub name: String,
    pub short_name: String,
    pub description: String,
    pub theme_color: String,
    pub background_color: String,
    pub start_url: String,
    pub scope: String,
    pub cache_version: String,
    pub display: String,
    pub orientation: String,
}

impl Default for FlowPwaConfig {
    fn default() -> Self {
        Self {
            name: "Resuma App".into(),
            short_name: "Resuma".into(),
            description: String::new(),
            theme_color: "#712cf9".into(),
            background_color: "#0f0a1a".into(),
            start_url: "/".into(),
            scope: "/".into(),
            cache_version: "1".into(),
            display: "standalone".into(),
            orientation: "any".into(),
        }
    }
}

impl FlowPwaConfig {
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
}

#[derive(Serialize)]
struct ManifestIcon {
    src: &'static str,
    sizes: &'static str,
    #[serde(rename = "type")]
    mime: &'static str,
    purpose: &'static str,
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
    icons: Vec<ManifestIcon>,
    shortcuts: Vec<ManifestShortcut<'a>>,
}

#[derive(Serialize)]
struct ManifestShortcut<'a> {
    name: &'a str,
    short_name: &'a str,
    url: &'a str,
}

fn icon_svg(size: u32, maskable: bool) -> String {
    let pad = if maskable { size / 5 } else { 0 };
    let inner = size.saturating_sub(pad * 2);
    let rx = if maskable { inner / 8 } else { inner / 6 };
    let font_size = inner / 2;
    let y = pad + inner * 2 / 3;
    let cx = pad + inner / 2;
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}" role="img" aria-label="Resuma">
<rect x="{pad}" y="{pad}" width="{inner}" height="{inner}" rx="{rx}" fill="{purple}"/>
<text x="{cx}" y="{y}" text-anchor="middle" fill="{white}" font-family="Segoe UI, system-ui, sans-serif" font-size="{font_size}" font-weight="700">R</text>
</svg>"##,
        size = size,
        pad = pad,
        inner = inner,
        rx = rx,
        cx = cx,
        y = y,
        font_size = font_size,
        purple = "#712cf9",
        white = "#ffffff",
    )
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
        lang: "en",
        dir: "ltr",
        categories: &["developer", "productivity"],
        icons: vec![
            ManifestIcon {
                src: "/icons/icon-192.svg",
                sizes: "192x192",
                mime: "image/svg+xml",
                purpose: "any",
            },
            ManifestIcon {
                src: "/icons/icon-512.svg",
                sizes: "512x512",
                mime: "image/svg+xml",
                purpose: "any",
            },
            ManifestIcon {
                src: "/icons/icon-maskable.svg",
                sizes: "512x512",
                mime: "image/svg+xml",
                purpose: "maskable",
            },
            ManifestIcon {
                src: "/icons/apple-touch-icon.svg",
                sizes: "180x180",
                mime: "image/svg+xml",
                purpose: "any",
            },
        ],
        shortcuts: vec![
            ManifestShortcut {
                name: "Documentation",
                short_name: "Docs",
                url: "/docs",
            },
            ManifestShortcut {
                name: "Getting Started",
                short_name: "Start",
                url: "/docs/getting_started",
            },
        ],
    };
    serde_json::to_string_pretty(&manifest).unwrap_or_else(|_| "{}".into())
}

fn service_worker(cfg: &FlowPwaConfig) -> String {
    let cache_name = format!("resuma-pwa-{}", cfg.cache_version);
    format!(
        r#"const CACHE = "{cache_name}";
const PRECACHE = [
  "/",
  "/docs",
  "/docs/getting_started",
  "/offline.html",
  "/manifest.webmanifest",
  "/pwa-register.js",
  "/favicon.svg",
  "/icons/icon-192.svg",
  "/icons/icon-512.svg",
  "/icons/icon-maskable.svg",
  "/icons/apple-touch-icon.svg",
  "/_resuma/loader.js",
  "/_resuma/core.js",
];

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
    url.pathname.endsWith(".svg") ||
    url.pathname.endsWith(".webmanifest") ||
    url.pathname.endsWith(".js")
  ) {{
    event.respondWith(
      caches.match(req).then((cached) => {{
        const network = fetch(req).then((res) => {{
          const copy = res.clone();
          caches.open(CACHE).then((cache) => cache.put(req, copy));
          return res;
        }});
        return cached || network;
      }}))
    );
  }}
}});
"#,
        cache_name = cache_name,
    )
}

const PWA_REGISTER_JS: &str = r#""use strict";
if ("serviceWorker" in navigator) {
  window.addEventListener("load", () => {
    navigator.serviceWorker.register("/sw.js", { scope: "/" }).catch(() => {});
  });
}
"#;

const OFFLINE_HTML: &str = r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<meta name="theme-color" content="#712cf9" />
<title>Offline — Resuma</title>
<style>
  body { margin: 0; min-height: 100vh; display: grid; place-items: center; font-family: system-ui, sans-serif;
    background: #0f0a1a; color: #e9d5ff; text-align: center; padding: 2rem; }
  h1 { color: #fff; margin-bottom: .5rem; }
  p { color: #c4b5fd; max-width: 28rem; }
  a { color: #a78bfa; }
</style>
</head>
<body>
  <main>
    <h1>You are offline</h1>
    <p>Resuma docs need a network connection for the latest pages. Cached content may still be available.</p>
    <p><a href="/">Return home</a></p>
  </main>
</body>
</html>"##;

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
            axum::routing::get(|| async move {
                (
                    [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                    OFFLINE_HTML,
                )
            }),
        )
        .route(
            "/icons/icon-192.svg",
            axum::routing::get(|| async move {
                (
                    [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
                    icon_svg(192, false),
                )
            }),
        )
        .route(
            "/icons/icon-512.svg",
            axum::routing::get(|| async move {
                (
                    [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
                    icon_svg(512, false),
                )
            }),
        )
        .route(
            "/icons/icon-maskable.svg",
            axum::routing::get(|| async move {
                (
                    [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
                    icon_svg(512, true),
                )
            }),
        )
        .route(
            "/icons/apple-touch-icon.svg",
            axum::routing::get(|| async move {
                (
                    [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
                    icon_svg(180, false),
                )
            }),
        )
}
