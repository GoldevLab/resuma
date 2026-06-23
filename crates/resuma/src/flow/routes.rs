//! Submit HTTP handler + SEO routes (robots, sitemap, OG image).

use std::collections::{BTreeMap, HashMap};

use crate::core::view::View;
use crate::core::ResumaError;
use crate::ssr::seo_kit::SeoKit;
use crate::ssr::{render_to_string, PageOptions};
use axum::extract::{ConnectInfo, Form, Path};
use axum::http::{header, HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::Router;
use std::net::SocketAddr;

use super::middleware::run_middleware;
use super::redirect::{extract_redirect, redirect_response};
use super::registry::dispatch_submit;
use super::submit::SubmitError;
use crate::server::{guard_mutation, http_status, security::config, CSRF_FIELD};

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct SubmitResponse {
    pub ok: bool,
    pub value: Option<serde_json::Value>,
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub field_errors: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FlowSeoConfig {
    pub site_url: String,
    pub paths: Vec<String>,
    /// When set, `/robots.txt` and `/llms.txt` use [`SeoKit`] generators (GEO + AI crawlers).
    pub seo_kit: Option<SeoKit>,
}

/// Which auto-generated SEO text routes to mount (skip paths already registered as pages).
#[derive(Debug, Clone, Copy, Default)]
pub struct SeoKitRouteOpts {
    pub robots: bool,
    pub llms: bool,
}

impl SeoKitRouteOpts {
    pub fn all() -> Self {
        Self {
            robots: true,
            llms: true,
        }
    }
}

const FAVICON_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" role="img" aria-label="Resuma">
<rect width="32" height="32" rx="6" fill="#712cf9"/>
<text x="16" y="22" text-anchor="middle" fill="#fff" font-family="Segoe UI, system-ui, sans-serif" font-size="18" font-weight="700">R</text>
</svg>"##;

const OG_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630" viewBox="0 0 1200 630" role="img" aria-label="Resuma">
<defs>
<linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
<stop offset="0%" stop-color="#712cf9"/>
<stop offset="55%" stop-color="#5a1fd4"/>
<stop offset="100%" stop-color="#0550ae"/>
</linearGradient>
</defs>
<rect width="1200" height="630" fill="url(#bg)"/>
<text x="96" y="250" fill="#ffffff" font-family="Segoe UI, system-ui, sans-serif" font-size="96" font-weight="700">Resuma</text>
<text x="96" y="340" fill="#e9d5ff" font-family="Segoe UI, system-ui, sans-serif" font-size="42">Instantly-interactive Rust — without hydration</text>
<text x="96" y="430" fill="#ddd6fe" font-family="Segoe UI, system-ui, sans-serif" font-size="28">907 B initial · 5.08 KiB first click · 0 B static pages</text>
</svg>"##;

pub async fn handle_submit(
    Path(name): Path<String>,
    uri: Uri,
    headers: HeaderMap,
    connect: ConnectInfo<SocketAddr>,
    Form(mut form): Form<HashMap<String, String>>,
) -> Response {
    let wants_json = headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("application/json"))
        .unwrap_or(false);

    let cfg = config();
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost")
        .to_string();
    let form_csrf = form.remove(CSRF_FIELD);
    let ip = client_ip_from_connect(&headers, &connect);

    if let Err(err) = guard_mutation(
        &headers,
        &host,
        &ip,
        "submit",
        cfg.submits_per_minute,
        form_csrf.as_deref(),
    ) {
        return submit_error(err, wants_json);
    }

    let data = serde_json::to_value(&form).unwrap_or(serde_json::Value::Null);
    let mut req = super::request::from_http(
        "POST",
        uri.path(),
        &headers,
        BTreeMap::new(),
        BTreeMap::new(),
    );

    match run_middleware(req.clone()).await {
        Ok(r) => req = r,
        Err(e) => return submit_error(e, wants_json),
    }

    match dispatch_submit(&name, data, req).await {
        Ok(value) => {
            let redirect = extract_redirect(&value);
            if wants_json {
                axum::Json(SubmitResponse {
                    ok: true,
                    value: Some(value),
                    error: None,
                    field_errors: BTreeMap::new(),
                    redirect,
                })
                .into_response()
            } else if let Some(loc) = redirect {
                redirect_response(&loc)
            } else {
                let html = render_to_string(
                    &PageOptions {
                        title: "Submitted".into(),
                        ..Default::default()
                    },
                    || View::text("Submitted successfully."),
                );
                axum::response::Html(html).into_response()
            }
        }
        Err(err) => {
            let (message, field_errors) = parse_submit_error(&err);
            let status = if field_errors.is_empty() {
                http_status(&err)
            } else {
                StatusCode::UNPROCESSABLE_ENTITY
            };
            (
                status,
                axum::Json(SubmitResponse {
                    ok: false,
                    value: None,
                    error: Some(message),
                    field_errors,
                    redirect: None,
                }),
            )
                .into_response()
        }
    }
}

fn parse_submit_error(err: &crate::core::ResumaError) -> (String, BTreeMap<String, String>) {
    if let crate::core::ResumaError::Other(msg) = err {
        if let Ok(se) = serde_json::from_str::<SubmitError>(msg) {
            return (se.message, se.field_errors);
        }
    }
    let cfg = config();
    (err.client_message(cfg.production), BTreeMap::new())
}

fn submit_error(err: ResumaError, wants_json: bool) -> Response {
    let status = http_status(&err);
    let cfg = config();
    let message = err.client_message(cfg.production);
    if wants_json {
        (
            status,
            axum::Json(SubmitResponse {
                ok: false,
                value: None,
                error: Some(message),
                field_errors: BTreeMap::new(),
                redirect: None,
            }),
        )
            .into_response()
    } else {
        (status, message).into_response()
    }
}

fn client_ip_from_connect(headers: &HeaderMap, connect: &ConnectInfo<SocketAddr>) -> String {
    crate::server::client_ip_from_parts(headers, Some(connect.0))
}

fn robots_body(seo: &FlowSeoConfig) -> String {
    if let Some(kit) = &seo.seo_kit {
        return kit.robots_txt();
    }
    let mut body = String::from("User-agent: *\nAllow: /\n");
    let base = seo.site_url.trim_end_matches('/');
    if !base.is_empty() {
        body.push_str("\nSitemap: ");
        body.push_str(base);
        body.push_str("/sitemap.xml\n");
    }
    body
}

/// Mount `/robots.txt` and `/llms.txt` from a [`SeoKit`].
pub fn attach_seo_kit_routes<S>(
    mut router: Router<S>,
    kit: SeoKit,
    opts: SeoKitRouteOpts,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    if opts.robots {
        let robots_kit = kit.clone();
        router = router.route(
            "/robots.txt",
            axum::routing::get(move || {
                let body = robots_kit.robots_txt();
                async move { ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body) }
            }),
        );
    }
    if opts.llms {
        router = router.route(
            "/llms.txt",
            axum::routing::get(move || {
                let body = kit.llms_txt();
                async move { ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body) }
            }),
        );
    }
    router
}

fn sitemap_xml(seo: &FlowSeoConfig) -> String {
    let base = seo.site_url.trim_end_matches('/');
    let mut paths = seo.paths.clone();
    if !paths.iter().any(|p| p == "/") {
        paths.push("/".into());
    }
    paths.sort();
    paths.dedup();

    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push_str(r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#);
    for path in paths {
        let priority = if path == "/" { "1.0" } else { "0.8" };
        let changefreq = if path == "/" { "weekly" } else { "monthly" };
        xml.push_str("<url><loc>");
        xml.push_str(base);
        if path == "/" {
            xml.push('/');
        } else {
            xml.push_str(&path);
        }
        xml.push_str("</loc><changefreq>");
        xml.push_str(changefreq);
        xml.push_str("</changefreq><priority>");
        xml.push_str(priority);
        xml.push_str("</priority></url>");
    }
    xml.push_str("</urlset>");
    xml
}

async fn serve_og_image() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
        OG_SVG,
    )
}

async fn serve_favicon() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
        FAVICON_SVG,
    )
}

pub fn attach_flow_routes(router: Router, seo: FlowSeoConfig) -> Router {
    let router = if seo.seo_kit.is_some() {
        attach_seo_kit_routes(router, seo.seo_kit.clone().unwrap(), SeoKitRouteOpts::all())
    } else {
        let robots_seo = seo.clone();
        router.route(
            "/robots.txt",
            axum::routing::get(move || {
                let body = robots_body(&robots_seo);
                async move { ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body) }
            }),
        )
    };
    let sitemap_seo = seo;

    router
        .route(
            "/sitemap.xml",
            axum::routing::get(move || {
                let body = sitemap_xml(&sitemap_seo);
                async move {
                    (
                        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
                        body,
                    )
                }
            }),
        )
        .route("/og.svg", axum::routing::get(serve_og_image))
        .route("/favicon.svg", axum::routing::get(serve_favicon))
        .route("/favicon.ico", axum::routing::get(serve_favicon))
        .route("/_resuma/submit/{name}", axum::routing::post(handle_submit))
}
