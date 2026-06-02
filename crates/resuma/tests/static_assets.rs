//! Additional HTTP integration tests.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use resuma::prelude::*;
use tower::ServiceExt;

static DEMO_JS: &[u8] = b"export function demo() {}";

#[test]
fn core_runtime_reads_current_csrf_payload_without_cache() {
    let source = include_str!("../../../runtime/src/core.ts");
    let core_js = resuma::server::runtime_asset::CORE_JS;

    assert!(source.contains("payload.csrf_token"));
    assert!(!source.contains("cachedCsrf"));
    assert!(core_js.contains("csrf_token"));
    assert!(core_js.contains("x-resuma-csrf"));
    assert!(
        source.contains("navigate") && source.contains("buildUrl"),
        "core.ts must expose SPA helpers on __resuma"
    );
}

#[tokio::test]
async fn flow_serves_public_directory_files() {
    let dir = std::env::temp_dir().join(format!("resuma-public-http-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("img")).unwrap();
    std::fs::write(dir.join("img/logo.png"), b"\x89PNG").unwrap();

    let app = FlowApp::new()
        .with_public_dir(&dir)
        .into_router(FlowServeOptions {
            addr: "127.0.0.1:0".parse().unwrap(),
            security: SecurityConfig {
                production: false,
                ..SecurityConfig::default()
            },
        });

    let res = app
        .oneshot(Request::get("/img/logo.png").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("image/png")
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn flow_serves_static_and_client_assets() {
    let app = FlowApp::new()
        .static_asset("/static/app.js", DEMO_JS, "application/javascript")
        .client_asset("widget", DEMO_JS)
        .into_router(FlowServeOptions {
            addr: "127.0.0.1:0".parse().unwrap(),
            security: SecurityConfig {
                production: false,
                ..SecurityConfig::default()
            },
        });

    let static_js = app
        .clone()
        .oneshot(Request::get("/static/app.js").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(static_js.status(), StatusCode::OK);
    assert_eq!(
        static_js
            .headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok()),
        Some("public, max-age=31536000, immutable")
    );
    let body = axum::body::to_bytes(static_js.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(body.as_ref(), DEMO_JS);

    let client_js = app
        .oneshot(
            Request::get("/static/client/widget.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(client_js.status(), StatusCode::OK);
    assert!(client_js.headers().get("cache-control").is_some());
}

#[tokio::test]
async fn pre_registered_island_chunk_keeps_custom_resume() {
    use parking_lot::RwLock;
    use resuma::core::ResumePayload;
    use std::collections::HashMap;
    use std::sync::Arc;

    let handlers: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
    let islands: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
    islands.write().insert(
        "chart".into(),
        "export function resume(p,s,r){ r.textContent='ok'; }\n".into(),
    );

    let payload = ResumePayload {
        signals: vec![],
        handlers: [("chart".into(), [("h1".into(), "return 1".into())].into())].into(),
        islands: vec!["chart".into()],
        actions: vec![],
        contexts: Default::default(),
        visible_tasks: Default::default(),
        effects: vec![],
        lazy_chunks: vec![],
        csrf_token: None,
    };

    resuma::server::handler_assets::merge_payload_handlers(&handlers, &islands, &payload);

    let src = islands.read().get("chart").cloned().unwrap();
    assert!(src.contains("r.textContent='ok'"));
    assert!(!src.contains("export function resume(props, signals, root) {}"));
}

#[tokio::test]
async fn handler_chunks_merge_new_symbols_for_existing_chunk() {
    use parking_lot::RwLock;
    use resuma::core::ResumePayload;
    use std::collections::HashMap;
    use std::sync::Arc;

    let handlers: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
    let islands: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));

    let first = ResumePayload {
        signals: vec![],
        handlers: [("Panel".into(), [("h_one".into(), "return 1".into())].into())].into(),
        islands: vec![],
        actions: vec![],
        contexts: Default::default(),
        visible_tasks: Default::default(),
        effects: vec![],
        lazy_chunks: vec![],
        csrf_token: None,
    };
    let second = ResumePayload {
        signals: vec![],
        handlers: [(
            "Panel".into(),
            [(
                "h_two".into(),
                "async (_event, state, __resuma) => { return 2; }".into(),
            )]
            .into(),
        )]
        .into(),
        islands: vec![],
        actions: vec![],
        contexts: Default::default(),
        visible_tasks: Default::default(),
        effects: vec![],
        lazy_chunks: vec![],
        csrf_token: None,
    };

    resuma::server::handler_assets::merge_payload_handlers(&handlers, &islands, &first);
    resuma::server::handler_assets::merge_payload_handlers(&handlers, &islands, &second);

    let src = handlers.read().get("Panel").cloned().unwrap();
    assert!(src.contains("export function h_one("));
    assert!(src.contains("export const h_two = async"));
    assert!(!src.contains("export async ("));
}
