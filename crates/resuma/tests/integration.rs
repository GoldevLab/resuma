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
async fn serves_component_route_without_render_path_syntax() {
    #[component]
    fn Home() {
        view! { <main>"component route"</main> }
    }

    let app = ResumaApp::new().component("/", Home).into_router();

    let page = app
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(page.status(), StatusCode::OK);
    let body = axum::body::to_bytes(page.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("component route"));
}

#[tokio::test(flavor = "multi_thread")]
async fn flow_serves_component_page_without_render_path_syntax() {
    #[component]
    fn Home() {
        view! { <main>"flow component route"</main> }
    }

    let app = FlowApp::new()
        .component("/", Home)
        .into_router(FlowServeOptions::default());

    let page = app
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(page.status(), StatusCode::OK);
    let body = axum::body::to_bytes(page.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("flow component route"));
}

#[tokio::test(flavor = "multi_thread")]
async fn flow_nav_only_pages_ship_client_loader() {
    #[component]
    fn Home() {
        view! {
            <main>
                <NavLink href="/about" activeClass="active" class="nav-link">"About"</NavLink>
            </main>
        }
    }

    let app = FlowApp::new()
        .component("/", Home)
        .into_router(FlowServeOptions::default());

    let page = app
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(page.status(), StatusCode::OK);
    let body = axum::body::to_bytes(page.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("data-r-nav"));
    assert!(
        html.contains(r#"id="resuma-state" nonce=""#),
        "state payload script needs a CSP nonce"
    );
    assert!(
        html.contains(r#"src="/_resuma/loader.js" nonce=""#),
        "loader script must carry the same CSP nonce as strict-dynamic requires"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn flow_component_state_and_handlers_are_registered_during_render() {
    #[component]
    fn Counter() {
        let count = signal(0_i32);
        view! {
            <button onClick={count.update(|c| *c += 1)}>
                "Count: " {count}
            </button>
        }
    }

    let app = FlowApp::new()
        .component("/", Counter)
        .into_router(FlowServeOptions::default());

    let page = app
        .clone()
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(page.status(), StatusCode::OK);
    let body = axum::body::to_bytes(page.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains(r#""signals":[{"id":1,"value":0}]"#));
    assert!(html.contains(r#"data-r-on:click="Counter#"#));

    let handler = app
        .oneshot(
            Request::get("/_resuma/handler/Counter.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(handler.status(), StatusCode::OK);
    let body = axum::body::to_bytes(handler.into_body(), usize::MAX)
        .await
        .unwrap();
    let module = String::from_utf8_lossy(&body);
    assert!(module.contains("export const"));
    assert!(module.contains("state.count"));
    assert!(module.contains("async (_event, state, __resuma)"));
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
