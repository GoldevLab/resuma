//! Deferred streaming SSR — loaders run once; failures set HTTP status before headers.

use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use resuma::prelude::*;
use tower::ServiceExt;

static STREAM_LOADER_CALLS: AtomicUsize = AtomicUsize::new(0);

fn counting_stream_loader(
    _req: FlowRequest,
) -> Pin<Box<dyn std::future::Future<Output = resuma::Result<serde_json::Value>> + Send>> {
    Box::pin(async {
        STREAM_LOADER_CALLS.fetch_add(1, Ordering::SeqCst);
        Ok(serde_json::json!({"ok": true}))
    })
}

fn failing_stream_loader(
    _req: FlowRequest,
) -> Pin<Box<dyn std::future::Future<Output = resuma::Result<serde_json::Value>> + Send>> {
    Box::pin(async { Err(resuma::ResumaError::Other("stream loader failed".into())) })
}

#[tokio::test(flavor = "multi_thread")]
async fn deferred_stream_loader_runs_once() {
    const LOADER: &str = "deferred_stream_once_loader";
    STREAM_LOADER_CALLS.store(0, Ordering::SeqCst);
    resuma::register_loader(LOADER, counting_stream_loader);
    resuma::register_stream_loader(LOADER);

    let app = FlowApp::new()
        .streaming(true)
        .page("/stream-once", |_req| {
            match resuma::try_use_load_value::<serde_json::Value>(LOADER) {
                LoadValue::Pending => view! {
                    <main>{stream_slot(LOADER)}</main>
                },
                LoadValue::Ok(_) => view! { <main>"sync ok"</main> },
                LoadValue::Err(err) => error_page(&FlowError::Loader(err)),
            }
        })
        .into_router(FlowServeOptions::default());

    let res = app
        .oneshot(Request::get("/stream-once").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("resuma-stream-loaded") || html.contains("\"ok\": true"),
        "expected streamed loader chunk in response"
    );
    assert_eq!(
        STREAM_LOADER_CALLS.load(Ordering::SeqCst),
        1,
        "deferred stream loader must run exactly once per request"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn deferred_stream_loader_failure_returns_500() {
    const LOADER: &str = "deferred_stream_fail_loader";
    resuma::register_loader(LOADER, failing_stream_loader);
    resuma::register_stream_loader(LOADER);

    let app = FlowApp::new()
        .streaming(true)
        .page("/stream-fail", |_req| {
            match resuma::try_use_load_value::<serde_json::Value>(LOADER) {
                LoadValue::Pending => view! {
                    <main>{stream_slot(LOADER)}</main>
                },
                LoadValue::Ok(_) => view! { <main>"never"</main> },
                LoadValue::Err(err) => error_page(&FlowError::Loader(err)),
            }
        })
        .into_router(FlowServeOptions::default());

    let res = app
        .oneshot(Request::get("/stream-fail").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "loader failure must set HTTP 500 before stream headers"
    );
}
