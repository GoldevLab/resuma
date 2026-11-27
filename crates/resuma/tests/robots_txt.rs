//! `/robots.txt` and `/llms.txt`: generated defaults and app-supplied overrides.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use resuma::prelude::*;
use tower::ServiceExt;

static CUSTOM_ROBOTS: &[u8] = b"User-agent: *\nDisallow: /private\n";

fn serve_opts() -> FlowServeOptions {
    FlowServeOptions {
        addr: "127.0.0.1:0".parse().unwrap(),
        security: SecurityConfig {
            production: false,
            ..SecurityConfig::default()
        },
    }
}

fn kit() -> SeoKit {
    SeoKit::new("Demo", "https://example.com")
        .with_llms_summary("A demo site.")
        .with_disallow("/api/")
        .with_disallow("/admin")
}

async fn fetch(app: &axum::Router, path: &str) -> (StatusCode, Option<String>, String) {
    let res = app
        .clone()
        .oneshot(Request::get(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let cache = res
        .headers()
        .get("cache-control")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, cache, String::from_utf8(body.to_vec()).unwrap())
}

#[tokio::test]
async fn default_robots_lists_ai_crawlers_and_guards_paths_in_every_group() {
    let app = FlowApp::new().with_seo_kit(kit()).into_router(serve_opts());
    let (status, _, body) = fetch(&app, "/robots.txt").await;
    assert_eq!(status, StatusCode::OK);
    for agent in [
        "GPTBot",
        "OAI-SearchBot",
        "ClaudeBot",
        "Claude-SearchBot",
        "Claude-User",
    ] {
        assert!(
            body.contains(&format!("User-agent: {agent}\n")),
            "missing {agent}"
        );
    }
    for group in body.split("\n\n").filter(|g| g.starts_with("User-agent:")) {
        assert!(
            group.contains("Disallow: /admin"),
            "group open to /admin: {group}"
        );
    }
    assert!(body.contains("Sitemap: https://example.com/sitemap.xml"));
}

#[tokio::test]
async fn static_asset_replaces_the_seo_kit_robots_route() {
    let app = FlowApp::new()
        .with_seo_kit(kit())
        .static_asset("/robots.txt", CUSTOM_ROBOTS, "text/plain; charset=utf-8")
        .into_router(serve_opts());

    let (status, cache, body) = fetch(&app, "/robots.txt").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_bytes(), CUSTOM_ROBOTS);
    let cache = cache.expect("cache-control");
    assert!(
        !cache.contains("immutable"),
        "robots.txt must not be immutable: {cache}"
    );

    // The kit still owns `/llms.txt`.
    let (status, _, llms) = fetch(&app, "/llms.txt").await;
    assert_eq!(status, StatusCode::OK);
    assert!(llms.contains("A demo site."));
}

#[tokio::test]
async fn static_asset_replaces_the_seo_kit_llms_route() {
    let app = FlowApp::new()
        .with_seo_kit(kit())
        .static_asset("/llms.txt", b"# Custom llms\n", "text/plain; charset=utf-8")
        .into_router(serve_opts());

    let (_, cache, body) = fetch(&app, "/llms.txt").await;
    assert_eq!(body, "# Custom llms\n");
    assert!(!cache.expect("cache-control").contains("immutable"));
    let (_, _, robots) = fetch(&app, "/robots.txt").await;
    assert!(robots.contains("User-agent: GPTBot"));
}

#[tokio::test]
async fn static_asset_robots_works_without_a_seo_kit() {
    let app = FlowApp::new()
        .static_asset("/robots.txt", CUSTOM_ROBOTS, "text/plain; charset=utf-8")
        .into_router(serve_opts());

    let (status, _, body) = fetch(&app, "/robots.txt").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_bytes(), CUSTOM_ROBOTS);
}

#[tokio::test]
async fn without_a_kit_or_override_flow_still_serves_a_default_robots() {
    let app = FlowApp::new().into_router(serve_opts());
    let (status, _, body) = fetch(&app, "/robots.txt").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.starts_with("User-agent: *\nAllow: /\n"));
}

#[tokio::test]
async fn seo_kit_robots_override_is_served_verbatim() {
    let app = FlowApp::new()
        .with_seo_kit(kit().with_robots_txt("User-agent: *\nDisallow: /nope"))
        .into_router(serve_opts());
    let (_, _, body) = fetch(&app, "/robots.txt").await;
    assert_eq!(body, "User-agent: *\nDisallow: /nope\n");
}
