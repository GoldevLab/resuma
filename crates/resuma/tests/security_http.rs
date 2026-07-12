//! HTTP security integration tests — CSRF, origin, rate limits.

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{header, HeaderValue, Request, StatusCode};
use resuma::prelude::*;
use resuma::server::{
    configure_security, guard_mutation, reset_rate_limits_for_tests, validate_csrf,
    validate_origin, SecurityConfig,
};
use std::net::SocketAddr;
use tower::ServiceExt;

fn test_connect_info() -> ConnectInfo<SocketAddr> {
    ConnectInfo("127.0.0.1:3000".parse().unwrap())
}

#[test]
fn security_guards_sequential() {
    configure_security(SecurityConfig {
        csrf: true,
        origin_check: true,
        ..SecurityConfig::from_env()
    });

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        header::ORIGIN,
        HeaderValue::from_static("http://127.0.0.1:3000"),
    );
    assert!(matches!(
        validate_csrf(&headers, None),
        Err(ResumaError::InvalidCsrf)
    ));

    configure_security(SecurityConfig {
        csrf: false,
        origin_check: true,
        ..SecurityConfig::from_env()
    });
    let mut bad_origin = axum::http::HeaderMap::new();
    bad_origin.insert(header::ORIGIN, HeaderValue::from_static("http://evil.test"));
    assert!(matches!(
        validate_origin(&bad_origin, "127.0.0.1:3000"),
        Err(ResumaError::Forbidden(_))
    ));

    configure_security(SecurityConfig {
        csrf: false,
        origin_check: false,
        actions_per_minute: 2,
        ..SecurityConfig::from_env()
    });
    reset_rate_limits_for_tests();
    let headers = axum::http::HeaderMap::new();
    let unique = resuma::exec::id::next_id();
    let ip = format!("10.0.0.{}", (unique % 200) + 1);
    let bucket = format!("test_rate_bucket_{unique}");
    assert!(guard_mutation(&headers, "localhost", &ip, &bucket, 2, None).is_ok());
    assert!(guard_mutation(&headers, "localhost", &ip, &bucket, 2, None).is_ok());
    assert!(matches!(
        guard_mutation(&headers, "localhost", &ip, &bucket, 2, None),
        Err(ResumaError::RateLimited)
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn action_post_without_csrf_returns_403() {
    configure_security(SecurityConfig {
        csrf: true,
        origin_check: false,
        ..SecurityConfig::from_env()
    });

    #[server]
    async fn ping() -> String {
        "pong".into()
    }

    let _ = ping;

    let app = ResumaApp::new().into_router();

    let res = app
        .oneshot(
            Request::post("/_resuma/action/ping")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::HOST, "127.0.0.1:3000")
                .header(header::ORIGIN, "http://127.0.0.1:3000")
                .extension(test_connect_info())
                .body(Body::from(r#"{"args":[]}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn interactive_page_includes_csrf_token_in_state() {
    configure_security(SecurityConfig::from_env());

    #[component]
    fn Counter() {
        let n = signal(0_i32);
        view! {
            <button onClick={n.update(|v| *v += 1)}>{n}</button>
        }
    }

    let app = ResumaApp::new().component("/", Counter).into_router();

    let res = app
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains(r#""csrf_token""#));
}
