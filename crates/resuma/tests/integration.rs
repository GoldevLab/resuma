//! HTTP integration smoke tests.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use resuma::prelude::*;
use tower::ServiceExt;

#[tokio::test]
async fn serves_page_and_runtime_assets() {
    let app = ResumaApp::new()
        .page("/", || view! { <main>"ok"</main> })
        .into_router();

    let page = app
        .clone()
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(page.status(), StatusCode::OK);
    let body = axum::body::to_bytes(page.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(
        body.windows(b"<main".len()).any(|w| w == b"<main")
            || body
                .windows(b"resuma-root".len())
                .any(|w| w == b"resuma-root")
    );

    let loader = app
        .oneshot(
            Request::get("/_resuma/loader.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(loader.status(), StatusCode::OK);
}

#[tokio::test]
async fn island_refresh_returns_cached_html() {
    use resuma::core::context::{with_context, RenderContext, RenderMode};
    use resuma::core::View;
    use resuma::server::island_cache;

    island_cache::clear_island_cache();

    let ctx = RenderContext::new(RenderMode::Ssr);
    let view = resuma::__private::wrap_in_island("demo", 1, View::Text("inner".into()), "eager");
    with_context(ctx, || {
        let _html = resuma::ssr::render_view(&view);
    });

    let app = ResumaApp::new().into_router();
    let res = app
        .oneshot(
            Request::get("/_resuma/island/demo-1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("resuma-island"));
    assert!(html.contains("inner"));
}
