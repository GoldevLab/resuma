//! HTTP integration tests for Resuma OS graph routes, tokens, and SSE replay.

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use axum::body::{to_bytes, Body};
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use resuma::exec::{
    configure_exec_security, durable, id, scheduler, ExecSecurityConfig, FlowEngine, GraphId,
    Resources, WorkerContext, WorkerMeta, WorkerRegistry,
};
use resuma::prelude::*;
use resuma::server::{configure_security, SecurityConfig};
use serde_json::{json, Value};
use tower::ServiceExt;

const API_KEY: &str = "test-exec-api-key-32chars-min!!!!";

fn test_connect_info() -> ConnectInfo<SocketAddr> {
    ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345)))
}

fn exec_http_lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn temp_durable(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("resuma-exec-http-{name}-{}", id::next_id()));
    let _ = std::fs::remove_dir_all(&p);
    durable::configure(&p);
    scheduler::configure(p.join("scheduler"));
    p
}

fn enable_exec_routes() {
    std::env::set_var("RESUMA_EXEC_ENABLED", "1");
}

fn configure_test_exec_security() {
    configure_security(SecurityConfig {
        csrf: false,
        origin_check: false,
        ..SecurityConfig::from_env()
    });
    configure_exec_security(ExecSecurityConfig {
        api_key: Some(API_KEY.into()),
        public: false,
        ..ExecSecurityConfig::from_env()
    });
}

fn echo_worker(input: Value, ctx: WorkerContext) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>> {
    Box::pin(async move {
        ctx.log("worker started");
        ctx.progress(25);
        ctx.log("checkpoint");
        ctx.progress(100);
        Ok(input)
    })
}

fn register_echo_worker(name: &str) {
    WorkerRegistry::new()
        .register(
            name,
            WorkerMeta {
                intent: "http integration echo".into(),
                resources: Resources::auto(),
            },
            echo_worker,
        )
        .install();
}

fn slow_worker(_input: Value, ctx: WorkerContext) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>> {
    Box::pin(async move {
        for _ in 0..50 {
            ctx.check_cancelled()?;
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
        Ok(json!({ "done": true }))
    })
}

fn register_slow_worker(name: &str) {
    WorkerRegistry::new()
        .register(
            name,
            WorkerMeta {
                intent: "slow http integration worker".into(),
                resources: Resources::auto(),
            },
            slow_worker,
        )
        .install();
}

async fn post_graph_control(
    app: &axum::Router,
    graph_id: &str,
    action: &str,
    token: Option<&str>,
) -> StatusCode {
    let mut builder = Request::builder()
        .method("POST")
        .uri(format!("/_resuma/graph/{graph_id}/{action}"))
        .extension(test_connect_info());
    if let Some(t) = token {
        builder = builder.header("x-resuma-graph-token", t);
    }
    app.clone()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap()
        .status()
}

async fn fetch_graph_snapshot(app: &axum::Router, graph_id: &str, token: &str) -> Value {
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/_resuma/graph/{graph_id}?token={token}"))
                .extension(test_connect_info())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    body_json(res).await
}

async fn body_json(res: axum::response::Response) -> Value {
    let bytes = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    serde_json::from_slice(&bytes).unwrap_or_else(|_| {
        panic!(
            "expected JSON body, got: {}",
            String::from_utf8_lossy(&bytes)
        )
    })
}

async fn start_worker_via_http(app: &axum::Router, worker: &str) -> (String, String) {
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/_resuma/worker/{worker}"))
                .header("authorization", format!("Bearer {API_KEY}"))
                .header("content-type", "application/json")
                .extension(test_connect_info())
                .body(Body::from(r#"{"input":{"x":1}}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK, "start worker");
    let body = body_json(res).await;
    let graph_id = body["graph_id"]
        .as_str()
        .expect("graph_id")
        .to_string();
    let token = body["access_token"]
        .as_str()
        .expect("access_token")
        .to_string();
    assert!(!graph_id.is_empty());
    assert!(!token.is_empty());
    (graph_id, token)
}

async fn get_graph_status(app: &axum::Router, graph_id: &str, token: Option<&str>) -> StatusCode {
    let uri = match token {
        Some(t) => format!("/_resuma/graph/{graph_id}?token={t}"),
        None => format!("/_resuma/graph/{graph_id}"),
    };
    let mut builder = Request::builder().uri(uri).extension(test_connect_info());
    if let Some(t) = token {
        builder = builder.header("x-resuma-graph-token", t);
    }
    let res = app
        .clone()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap();
    res.status()
}

async fn wait_for_terminal_graph(app: &axum::Router, graph_id: &str, token: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/_resuma/graph/{graph_id}?token={token}"))
                    .extension(test_connect_info())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = body_json(res).await;
        let status = body["status"].as_str().unwrap_or("");
        if matches!(status, "done" | "failed" | "paused") {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timeout waiting for terminal graph, last status: {status}"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

#[tokio::test]
async fn exec_status_requires_api_key_when_configured() {
    let _guard = exec_http_lock();
    enable_exec_routes();
    configure_test_exec_security();

    let app = ResumaApp::new().into_router();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/_resuma/status")
                .extension(test_connect_info())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn exec_status_accepts_bearer_api_key() {
    let _guard = exec_http_lock();
    enable_exec_routes();
    configure_test_exec_security();

    let app = ResumaApp::new().into_router();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/_resuma/status")
                .header("authorization", format!("Bearer {API_KEY}"))
                .extension(test_connect_info())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn exec_worker_rejects_without_api_key_in_production_mode() {
    let _guard = exec_http_lock();
    enable_exec_routes();
    configure_test_exec_security();

    let app = ResumaApp::new().into_router();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_resuma/worker/unknown")
                .header("content-type", "application/json")
                .extension(test_connect_info())
                .body(Body::from(r#"{"input":{}}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn graph_snapshot_requires_token() {
    let _guard = exec_http_lock();
    let _root = temp_durable("graph-auth");
    enable_exec_routes();
    configure_test_exec_security();
    let worker = format!("http_auth_{}", id::next_id());
    register_echo_worker(&worker);

    let app = ResumaApp::new().into_router();
    let (graph_id, token) = start_worker_via_http(&app, &worker).await;

    assert_eq!(
        get_graph_status(&app, &graph_id, None).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        get_graph_status(&app, &graph_id, Some("wrong-token-value")).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        get_graph_status(&app, &graph_id, Some(&token)).await,
        StatusCode::OK
    );
}

#[tokio::test]
async fn graph_replay_returns_worker_events() {
    let _guard = exec_http_lock();
    let _root = temp_durable("graph-replay");
    enable_exec_routes();
    configure_test_exec_security();
    let worker = format!("http_replay_{}", id::next_id());
    register_echo_worker(&worker);

    let app = ResumaApp::new().into_router();
    let (graph_id, token) = start_worker_via_http(&app, &worker).await;
    wait_for_terminal_graph(&app, &graph_id, &token).await;

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/_resuma/graph/{graph_id}/replay?token={token}"))
                .extension(test_connect_info())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let events = body_json(res).await;
    let arr = events.as_array().expect("event array");
    assert!(!arr.is_empty(), "replay should return worker events");
}

#[tokio::test]
async fn graph_sse_replays_events_after_worker_completes() {
    let _guard = exec_http_lock();
    let _root = temp_durable("graph-sse");
    enable_exec_routes();
    configure_test_exec_security();
    let worker = format!("http_sse_{}", id::next_id());
    register_echo_worker(&worker);

    let app = ResumaApp::new().into_router();
    let (graph_id, token) = start_worker_via_http(&app, &worker).await;
    wait_for_terminal_graph(&app, &graph_id, &token).await;

    // In-memory bus is gone after completion — SSE must fall back to durable replay.
    assert!(FlowEngine::bus(&GraphId(graph_id.clone())).is_none());

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/_resuma/graph/{graph_id}/events?token={token}"))
                .extension(test_connect_info())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(
        text.contains("data:"),
        "SSE body should contain events, got: {text}"
    );
    assert!(
        text.contains("checkpoint") || text.contains("log"),
        "SSE should include worker log events, got: {text}"
    );
}

#[tokio::test]
async fn graph_snapshot_has_nodes_while_running() {
    let _guard = exec_http_lock();
    let _root = temp_durable("graph-nodes");
    enable_exec_routes();
    configure_test_exec_security();
    let worker = format!("http_nodes_{}", id::next_id());
    register_echo_worker(&worker);

    let app = ResumaApp::new().into_router();
    let (graph_id, token) = start_worker_via_http(&app, &worker).await;

    let mut saw_nodes = false;
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/_resuma/graph/{graph_id}?token={token}"))
                    .extension(test_connect_info())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = body_json(res).await;
        if body["nodes"].as_array().is_some_and(|n| !n.is_empty()) {
            saw_nodes = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(15)).await;
    }
    assert!(saw_nodes, "graph snapshot should expose nodes");
}

#[tokio::test]
async fn graph_snapshot_accepts_token_header() {
    let _guard = exec_http_lock();
    let _root = temp_durable("graph-header");
    enable_exec_routes();
    configure_test_exec_security();
    let worker = format!("http_hdr_{}", id::next_id());
    register_echo_worker(&worker);

    let app = ResumaApp::new().into_router();
    let (graph_id, token) = start_worker_via_http(&app, &worker).await;

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/_resuma/graph/{graph_id}"))
                .header("x-resuma-graph-token", &token)
                .extension(test_connect_info())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn graph_control_requires_graph_token_header() {
    let _guard = exec_http_lock();
    let _root = temp_durable("graph-control-auth");
    enable_exec_routes();
    configure_test_exec_security();
    let worker = format!("http_ctrl_{}", id::next_id());
    register_slow_worker(&worker);

    let app = ResumaApp::new().into_router();
    let (graph_id, token) = start_worker_via_http(&app, &worker).await;
    tokio::time::sleep(Duration::from_millis(80)).await;

    assert_eq!(
        post_graph_control(&app, &graph_id, "pause", None).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        post_graph_control(&app, &graph_id, "pause", Some("wrong-token-value")).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        post_graph_control(&app, &graph_id, "pause", Some(&token)).await,
        StatusCode::NO_CONTENT
    );

    let snap = fetch_graph_snapshot(&app, &graph_id, &token).await;
    assert_eq!(snap["status"].as_str(), Some("paused"));
}

#[tokio::test]
async fn graph_cancel_via_http_marks_failed() {
    let _guard = exec_http_lock();
    let _root = temp_durable("graph-cancel");
    enable_exec_routes();
    configure_test_exec_security();
    let worker = format!("http_cancel_{}", id::next_id());
    register_slow_worker(&worker);

    let app = ResumaApp::new().into_router();
    let (graph_id, token) = start_worker_via_http(&app, &worker).await;
    tokio::time::sleep(Duration::from_millis(80)).await;

    assert_eq!(
        post_graph_control(&app, &graph_id, "cancel", Some(&token)).await,
        StatusCode::NO_CONTENT
    );
    tokio::time::sleep(Duration::from_millis(150)).await;

    let snap = fetch_graph_snapshot(&app, &graph_id, &token).await;
    assert_eq!(snap["status"].as_str(), Some("failed"));
}

#[tokio::test]
async fn graph_replay_has_no_duplicate_log_events() {
    let _guard = exec_http_lock();
    let _root = temp_durable("graph-dedupe");
    enable_exec_routes();
    configure_test_exec_security();
    let worker = format!("http_dedupe_{}", id::next_id());
    register_echo_worker(&worker);

    let app = ResumaApp::new().into_router();
    let (graph_id, token) = start_worker_via_http(&app, &worker).await;
    wait_for_terminal_graph(&app, &graph_id, &token).await;

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/_resuma/graph/{graph_id}/replay?token={token}"))
                .extension(test_connect_info())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let events = body_json(res).await;
    let arr = events.as_array().expect("event array");
    let checkpoint_logs = arr
        .iter()
        .filter(|ev| {
            ev.get("type").and_then(|t| t.as_str()) == Some("log")
                && ev
                    .get("message")
                    .and_then(|m| m.as_str())
                    .is_some_and(|m| m.contains("checkpoint"))
        })
        .count();
    assert_eq!(
        checkpoint_logs, 1,
        "replay should store each log once, got {checkpoint_logs} checkpoint events"
    );
}

#[tokio::test]
async fn queue_enqueue_and_stats_via_http() {
    let _guard = exec_http_lock();
    let _root = temp_durable("queue-http");
    enable_exec_routes();
    configure_test_exec_security();
    let worker = format!("http_queue_{}", id::next_id());
    register_echo_worker(&worker);

    let app = ResumaApp::new().into_router();
    let queue = format!("q_{}", id::next_id());

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/_resuma/queue/{queue}"))
                .header("authorization", format!("Bearer {API_KEY}"))
                .header("content-type", "application/json")
                .extension(test_connect_info())
                .body(Body::from(format!(
                    r#"{{"worker":"{worker}","input":{{"x":1}}}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK, "enqueue");
    let body = body_json(res).await;
    assert!(body["message_id"].as_str().is_some_and(|s| !s.is_empty()));

    let stats_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/_resuma/queue/{queue}/stats"))
                .header("authorization", format!("Bearer {API_KEY}"))
                .extension(test_connect_info())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(stats_res.status(), StatusCode::OK);
    let stats = body_json(stats_res).await;
    assert!(
        stats["pending"].as_u64().unwrap_or(0) + stats["processing"].as_u64().unwrap_or(0)
            + stats["done"].as_u64().unwrap_or(0)
            >= 1,
        "queue stats should reflect enqueued job: {stats}"
    );
}

#[tokio::test]
async fn metrics_requires_api_key_when_configured() {
    let _guard = exec_http_lock();
    enable_exec_routes();
    configure_test_exec_security();

    let app = ResumaApp::new().into_router();
    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/_resuma/metrics")
                .extension(test_connect_info())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);

    let ok = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/_resuma/metrics")
                .header("authorization", format!("Bearer {API_KEY}"))
                .extension(test_connect_info())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    let body = to_bytes(ok.into_body(), 64 * 1024).await.unwrap();
    let text = String::from_utf8_lossy(&body);
    assert!(
        text.contains("resuma_exec_graphs_total"),
        "metrics should export Prometheus text, got: {text}"
    );
}

#[tokio::test]
async fn graph_token_reads_not_ip_rate_limited() {
    let _guard = exec_http_lock();
    let _root = temp_durable("graph-rate");
    enable_exec_routes();
    configure_test_exec_security();
    let worker = format!("http_rate_{}", id::next_id());
    register_echo_worker(&worker);

    let app = ResumaApp::new().into_router();
    let (graph_id, token) = start_worker_via_http(&app, &worker).await;

    for _ in 0..200 {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/_resuma/graph/{graph_id}?token={token}"))
                    .extension(test_connect_info())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_ne!(
            res.status(),
            StatusCode::TOO_MANY_REQUESTS,
            "graph token reads should not hit IP rate limit"
        );
        if res.status() != StatusCode::OK {
            break;
        }
    }
}

#[tokio::test]
async fn scheduler_create_and_list_via_http() {
    let _guard = exec_http_lock();
    let _root = temp_durable("scheduler-http");
    enable_exec_routes();
    configure_test_exec_security();
    let worker = format!("http_sched_{}", id::next_id());
    register_echo_worker(&worker);

    let app = ResumaApp::new().into_router();
    let schedule_name = format!("nightly_{}", id::next_id());

    let create_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_resuma/scheduler")
                .header("authorization", format!("Bearer {API_KEY}"))
                .header("content-type", "application/json")
                .extension(test_connect_info())
                .body(Body::from(format!(
                    r#"{{"name":"{schedule_name}","cron":"0 3 * * *","worker":"{worker}","input":{{}}}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_res.status(), StatusCode::OK, "create schedule");
    let created = body_json(create_res).await;
    assert_eq!(created["name"].as_str(), Some(schedule_name.as_str()));
    assert!(created["id"].as_str().is_some_and(|s| !s.is_empty()));

    let list_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/_resuma/scheduler")
                .header("authorization", format!("Bearer {API_KEY}"))
                .extension(test_connect_info())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_res.status(), StatusCode::OK);
    let list = body_json(list_res).await;
    let jobs = list["jobs"].as_array().expect("jobs array");
    assert!(
        jobs.iter().any(|j| j["name"].as_str() == Some(schedule_name.as_str())),
        "listed schedules should include created job"
    );
}
