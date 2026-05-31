//! Ops endpoints, request-id propagation, and loader-error handling.

use std::net::SocketAddr;
use std::pin::Pin;

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use resuma::prelude::*;
use tower::ServiceExt;

fn test_connect_info() -> ConnectInfo<SocketAddr> {
    ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345)))
}

#[tokio::test]
async fn health_and_ready_return_ok() {
    let app = ResumaApp::new()
        .page("/", || view! { <main>"ok"</main> })
        .into_router();

    for path in ["/health", "/ready"] {
        let res = app
            .clone()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK, "{path} should be 200");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn flow_router_echoes_request_id() {
    let app = FlowApp::new()
        .page("/", |_req| view! { <main>"home"</main> })
        .into_router(FlowServeOptions::default());

    // Client-provided id is echoed back unchanged.
    let res = app
        .clone()
        .oneshot(
            Request::get("/")
                .header("x-request-id", "test-correlation-123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        res.headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok()),
        Some("test-correlation-123")
    );

    // Absent id is generated.
    let res = app
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert!(res.headers().get("x-request-id").is_some());
}

fn failing_loader(
    _req: FlowRequest,
) -> Pin<Box<dyn std::future::Future<Output = resuma::Result<serde_json::Value>> + Send>> {
    Box::pin(async {
        Err(resuma::ResumaError::Other(
            "database unavailable".to_string(),
        ))
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn loader_failure_renders_error_page_without_crashing() {
    resuma::register_loader("ops_flow_failing_loader", failing_loader);

    let app = FlowApp::new()
        .page("/boom", |_req| {
            // Panicking accessor on a failed loader — must be caught and turned
            // into an error page rather than aborting the request.
            let _data: String = resuma::use_load("ops_flow_failing_loader");
            view! { <main>"never reached"</main> }
        })
        .into_router(FlowServeOptions::default());

    let res = app
        .oneshot(Request::get("/boom").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // The connection survived (no panic propagation) and an error page rendered.
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("Error 500") || html.contains("resuma-error"),
        "expected rendered error page, got: {html}"
    );
}

fn redirect_submit(
    _data: serde_json::Value,
    _req: FlowRequest,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = resuma::Result<serde_json::Value>> + Send>>
{
    Box::pin(async { Ok(serde_json::to_value(resuma::Redirect::to("/items?created=1")).unwrap()) })
}

const TEST_CSRF: &str = "abcdef0123456789abcdef";

#[data]
struct ActionDto {
    value: i32,
}

#[data]
struct ActionReply {
    message: String,
}

#[server]
async fn ops_flow_data_action(input: ActionDto) -> Result<ActionReply> {
    Ok(ActionReply {
        message: format!("value={}", input.value),
    })
}

#[data]
struct EmptySubmit {}

#[submit]
async fn ops_flow_invalid_submit(_data: EmptySubmit) -> std::result::Result<(), SubmitError> {
    Err(SubmitError::new("Please fix the form").field("title", "Title is required"))
}

#[tokio::test(flavor = "multi_thread")]
async fn data_macro_supplies_action_serialization_traits() {
    let app = FlowApp::new()
        .page("/", |_req| view! { <main>"home"</main> })
        .into_router(FlowServeOptions::default());

    let res = app
        .oneshot(
            Request::post("/_resuma/action/ops_flow_data_action")
                .header("content-type", "application/json")
                .header("host", "localhost")
                .header("cookie", format!("__resuma-csrf={TEST_CSRF}"))
                .header("x-resuma-csrf", TEST_CSRF)
                .extension(test_connect_info())
                .body(Body::from(r#"{"args":[{"value":7}]}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["value"]["message"], "value=7");
}

#[tokio::test(flavor = "multi_thread")]
async fn action_decode_error_names_argument_and_suggests_data_macro() {
    let app = FlowApp::new()
        .page("/", |_req| view! { <main>"home"</main> })
        .into_router(FlowServeOptions::default());

    let res = app
        .oneshot(
            Request::post("/_resuma/action/ops_flow_data_action")
                .header("content-type", "application/json")
                .header("host", "localhost")
                .header("cookie", format!("__resuma-csrf={TEST_CSRF}"))
                .header("x-resuma-csrf", TEST_CSRF)
                .extension(test_connect_info())
                .body(Body::from(r#"{"args":[{"value":"wrong"}]}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let error = json["error"].as_str().unwrap();
    assert!(error.contains("argument `input`"));
    assert!(error.contains("ops_flow_data_action"));
    assert!(error.contains("#[data]"));
}

#[tokio::test(flavor = "multi_thread")]
async fn submit_field_errors_return_422_json() {
    let app = FlowApp::new()
        .page("/", |_req| view! { <main>"home"</main> })
        .into_router(FlowServeOptions::default());

    let res = app
        .oneshot(
            Request::post("/_resuma/submit/ops_flow_invalid_submit")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("accept", "application/json")
                .header("host", "localhost")
                .header("cookie", format!("__resuma-csrf={TEST_CSRF}"))
                .header("x-resuma-csrf", TEST_CSRF)
                .extension(test_connect_info())
                .body(Body::from(""))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["ok"], serde_json::Value::Bool(false));
    assert_eq!(json["error"], "Please fix the form");
    assert_eq!(json["field_errors"]["title"], "Title is required");
}

#[test]
fn form_submit_accepts_qualified_handler_paths() {
    use resuma::core::context::{with_context, RenderContext, RenderMode};

    let ctx = RenderContext::new(RenderMode::Ssr);
    let view = view! {
        <Form submit={crate::ops_flow_invalid_submit}>
            <button type="submit">"Save"</button>
        </Form>
    };
    let html = with_context(ctx, || resuma::ssr::render_view(&view));
    assert!(html.contains(r#"data-r-submit="ops_flow_invalid_submit""#));
    assert!(html.contains(r#"action="/_resuma/submit/ops_flow_invalid_submit""#));
}

#[tokio::test(flavor = "multi_thread")]
async fn flow_form_renders_csrf_hidden_field_for_no_js_submit() {
    let app = FlowApp::new()
        .page("/", |_req| {
            view! {
                <main>
                    <Form submit={crate::ops_flow_invalid_submit}>
                        <input name="title" />
                        <button type="submit">"Save"</button>
                    </Form>
                </main>
            }
        })
        .into_router(FlowServeOptions::default());

    let res = app
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let set_cookie = res
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .expect("page should set CSRF cookie")
        .to_string();
    let token = set_cookie
        .strip_prefix("__resuma-csrf=")
        .and_then(|rest| rest.split(';').next())
        .expect("CSRF cookie should include token");

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);

    assert!(html.contains(r#"name="_csrf""#), "{html}");
    assert!(html.contains(&format!(r#"value="{token}""#)), "{html}");
}

#[tokio::test(flavor = "multi_thread")]
async fn unknown_submit_returns_404_json() {
    let app = FlowApp::new()
        .page("/", |_req| view! { <main>"home"</main> })
        .into_router(FlowServeOptions::default());

    let res = app
        .oneshot(
            Request::post("/_resuma/submit/does_not_exist")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("accept", "application/json")
                .header("host", "localhost")
                .header("cookie", format!("__resuma-csrf={TEST_CSRF}"))
                .header("x-resuma-csrf", TEST_CSRF)
                .extension(test_connect_info())
                .body(Body::from(""))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["ok"], serde_json::Value::Bool(false));
    assert!(json["error"].as_str().unwrap().contains("does_not_exist"));
}

#[test]
fn validation_errors_are_422() {
    assert_eq!(
        ResumaError::validation("bad input").status_code(),
        StatusCode::UNPROCESSABLE_ENTITY.as_u16()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn submit_redirect_303_without_js() {
    resuma::register_submit("ops_flow_redirect_submit", redirect_submit);

    let app = FlowApp::new()
        .page("/", |_req| view! { <main>"home"</main> })
        .into_router(FlowServeOptions::default());

    // No `Accept: application/json` → progressive-enhancement PRG path (303).
    let res = app
        .oneshot(
            Request::post("/_resuma/submit/ops_flow_redirect_submit")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("cookie", format!("__resuma-csrf={TEST_CSRF}"))
                .header("x-resuma-csrf", TEST_CSRF)
                .extension(test_connect_info())
                .body(Body::from(""))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        res.headers().get("location").and_then(|v| v.to_str().ok()),
        Some("/items?created=1")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn submit_redirect_json_hint() {
    resuma::register_submit("ops_flow_redirect_submit_json", redirect_submit);

    let app = FlowApp::new()
        .page("/", |_req| view! { <main>"home"</main> })
        .into_router(FlowServeOptions::default());

    let res = app
        .oneshot(
            Request::post("/_resuma/submit/ops_flow_redirect_submit_json")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("accept", "application/json")
                .header("cookie", format!("__resuma-csrf={TEST_CSRF}"))
                .header("x-resuma-csrf", TEST_CSRF)
                .extension(test_connect_info())
                .body(Body::from(""))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["ok"], serde_json::Value::Bool(true));
    assert_eq!(json["redirect"], "/items?created=1");
}

#[test]
fn render_view_snapshot_is_stable() {
    use resuma::core::context::{with_context, RenderContext, RenderMode};

    let view = view! {
        <section class="card">
            <h2>"Title"</h2>
            <p>"Body text"</p>
        </section>
    };

    let ctx = RenderContext::new(RenderMode::Ssr);
    let html = with_context(ctx, || resuma::ssr::render_view(&view));

    assert_eq!(
        html,
        r#"<section class="card"><h2>Title</h2><p>Body text</p></section>"#
    );
}

#[test]
fn view_macro_decodes_rust_string_literals() {
    use resuma::core::context::{with_context, RenderContext, RenderMode};

    let view = view! {
        <p title="quote: \"">
            "Line\nBreak \"quoted\""
        </p>
    };

    let ctx = RenderContext::new(RenderMode::Ssr);
    let html = with_context(ctx, || resuma::ssr::render_view(&view));

    assert_eq!(
        html,
        "<p title=\"quote: &quot;\">Line\nBreak \"quoted\"</p>"
    );
}
