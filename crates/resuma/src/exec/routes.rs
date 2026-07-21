//! HTTP routes for the Resuma execution layer.

use std::convert::Infallible;
use std::net::SocketAddr;
use std::pin::Pin;
use std::time::Duration;

use async_stream::stream;
use axum::body::Body;
use axum::extract::{ConnectInfo, Multipart, Path, Query};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use futures_util::Stream;
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::broadcast::error::RecvError;

use crate::core::ResumaError;
use crate::exec::engine::FlowEngine;
use crate::exec::events::SharedEventBus;
use crate::exec::metrics;
use crate::exec::queue::{self, EnqueueResponse};
use crate::exec::scheduler::{self, CreateScheduleBody, ScheduleListResponse};
use crate::exec::security::{
    self, guard_admin, guard_admin_read, guard_graph_control, guard_graph_read, guard_metrics,
    validate_graph_id, validate_input, validate_resource_name, validate_schedule_id,
};
use crate::exec::status;
use crate::exec::types::{GraphId, GraphSnapshot, StartWorkerResponse, WorkerEvent};
use crate::exec::webhooks::{self, RegisterWebhookBody, WebhookListResponse};

#[derive(Debug, Deserialize, Default)]
pub struct StartWorkerBody {
    #[serde(default)]
    pub input: Value,
}

#[derive(Debug, Deserialize)]
pub struct EnqueueBody {
    pub worker: String,
    #[serde(default)]
    pub input: Value,
}

#[derive(Debug, Deserialize, Default)]
struct GraphTokenQuery {
    #[serde(default)]
    token: Option<String>,
}

fn request_host(headers: &HeaderMap) -> String {
    headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost")
        .to_string()
}

pub fn attach_exec_routes<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router
        .route("/_resuma/metrics", get(prometheus_metrics_handler))
        .route("/_resuma/status", get(exec_status_handler))
        .route(
            "/_resuma/webhooks",
            get(list_webhooks_handler).post(register_webhook_handler),
        )
        .route("/_resuma/webhooks/{id}", delete(delete_webhook_handler))
        .route(
            "/_resuma/scheduler",
            get(list_schedules_handler).post(create_schedule_handler),
        )
        .route("/_resuma/scheduler/tick", post(scheduler_tick_handler))
        .route("/_resuma/scheduler/{id}", delete(delete_schedule_handler))
        .route("/_resuma/worker/{name}", post(start_worker))
        .route("/_resuma/queue/{name}", post(enqueue_worker))
        .route("/_resuma/queue/{name}/stats", get(queue_stats_handler))
        .route("/_resuma/graph/{id}", get(get_graph))
        .route("/_resuma/graph/{id}/status", get(get_graph_status))
        .route("/_resuma/graph/{id}/events", get(graph_events_sse))
        .route("/_resuma/graph/{id}/replay", get(replay_graph))
        .route("/_resuma/graph/{id}/pause", post(pause_graph))
        .route("/_resuma/graph/{id}/resume", post(resume_graph))
        .route("/_resuma/graph/{id}/cancel", post(cancel_graph))
        .route("/_resuma/artifact/{id}", get(get_artifact))
        .route("/_resuma/upload", post(upload_multipart))
        .route("/_resuma/upload/{name}", post(upload_named))
        .route("/_resuma/uploads/{id}", get(get_upload))
}

fn client_ip(headers: &HeaderMap, addr: SocketAddr) -> String {
    security::client_ip(headers, Some(addr))
}

async fn prometheus_metrics_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ExecHttpError> {
    let ip = client_ip(&headers, addr);
    guard_metrics(&headers, &ip)?;
    let body = metrics::prometheus_text();
    Ok((
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        body,
    ))
}

async fn list_webhooks_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<WebhookListResponse>, ExecHttpError> {
    let ip = client_ip(&headers, addr);
    guard_admin_read(&headers, &ip)?;
    Ok(Json(webhooks::list()))
}

async fn register_webhook_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<RegisterWebhookBody>,
) -> Result<Json<webhooks::WebhookTarget>, ExecHttpError> {
    let host = request_host(&headers);
    let ip = client_ip(&headers, addr);
    guard_admin(&headers, &host, &ip, None)?;
    Ok(Json(webhooks::register_resolved(body).await?))
}

async fn delete_webhook_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, ExecHttpError> {
    let host = request_host(&headers);
    let ip = client_ip(&headers, addr);
    guard_admin(&headers, &host, &ip, None)?;
    validate_schedule_id(&id)?;
    if webhooks::remove(&id)? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

async fn exec_status_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<status::ExecStatus>, ExecHttpError> {
    let ip = client_ip(&headers, addr);
    guard_admin_read(&headers, &ip)?;
    Ok(Json(status::snapshot()))
}

async fn list_schedules_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<ScheduleListResponse>, ExecHttpError> {
    let ip = client_ip(&headers, addr);
    guard_admin_read(&headers, &ip)?;
    Ok(Json(scheduler::list_response()?))
}

async fn create_schedule_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<CreateScheduleBody>,
) -> Result<Json<scheduler::ScheduleJob>, ExecHttpError> {
    let host = request_host(&headers);
    let ip = client_ip(&headers, addr);
    guard_admin(&headers, &host, &ip, None)?;
    validate_input(&body.input)?;
    Ok(Json(scheduler::create(body)?))
}

async fn delete_schedule_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, ExecHttpError> {
    let host = request_host(&headers);
    let ip = client_ip(&headers, addr);
    guard_admin(&headers, &host, &ip, None)?;
    validate_schedule_id(&id)?;
    if scheduler::remove(&id)? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

async fn scheduler_tick_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ExecHttpError> {
    let host = request_host(&headers);
    let ip = client_ip(&headers, addr);
    guard_admin(&headers, &host, &ip, None)?;
    let fired = scheduler::tick().await?;
    Ok(Json(serde_json::json!({ "ok": true, "fired": fired })))
}

async fn start_worker(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(body): Json<StartWorkerBody>,
) -> Result<Json<StartWorkerResponse>, ExecHttpError> {
    let host = request_host(&headers);
    let ip = client_ip(&headers, addr);
    guard_admin(&headers, &host, &ip, None)?;
    validate_resource_name(&name)?;
    validate_input(&body.input)?;
    Ok(Json(FlowEngine::start(&name, body.input).await?))
}

async fn enqueue_worker(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(queue_name): Path<String>,
    Json(body): Json<EnqueueBody>,
) -> Result<Json<EnqueueResponse>, ExecHttpError> {
    let host = request_host(&headers);
    let ip = client_ip(&headers, addr);
    guard_admin(&headers, &host, &ip, None)?;
    validate_resource_name(&queue_name)?;
    validate_resource_name(&body.worker)?;
    validate_input(&body.input)?;
    Ok(Json(
        queue::enqueue(&queue_name, &body.worker, body.input).await?,
    ))
}

async fn queue_stats_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(queue_name): Path<String>,
) -> Result<Json<super::queue_disk::QueueStats>, ExecHttpError> {
    let ip = client_ip(&headers, addr);
    guard_admin_read(&headers, &ip)?;
    validate_resource_name(&queue_name)?;
    Ok(Json(queue::queue_stats(&queue_name)))
}

async fn get_graph(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<GraphTokenQuery>,
) -> Result<Json<GraphSnapshot>, ExecHttpError> {
    let host = request_host(&headers);
    let ip = client_ip(&headers, addr);
    validate_graph_id(&id)?;
    let gid = GraphId(id);
    guard_graph_read(&headers, &host, &ip, &gid, query.token.as_deref())?;
    FlowEngine::snapshot(&gid)
        .map(Json)
        .ok_or(ExecHttpError(ResumaError::UnknownGraph(gid.0)))
}

/// Lightweight poll: status + progress without shipping the full node graph.
#[derive(Debug, serde::Serialize)]
struct GraphStatusBody {
    id: String,
    status: crate::exec::types::GraphStatus,
    progress: u8,
    worker: String,
}

async fn get_graph_status(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<GraphTokenQuery>,
) -> Result<Json<GraphStatusBody>, ExecHttpError> {
    let host = request_host(&headers);
    let ip = client_ip(&headers, addr);
    validate_graph_id(&id)?;
    let gid = GraphId(id.clone());
    guard_graph_read(&headers, &host, &ip, &gid, query.token.as_deref())?;
    let snap = FlowEngine::snapshot(&gid).ok_or(ExecHttpError(ResumaError::UnknownGraph(id)))?;
    Ok(Json(GraphStatusBody {
        id: snap.id.0,
        status: snap.status,
        progress: snap.progress,
        worker: snap.worker,
    }))
}

async fn get_artifact(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<GraphTokenQuery>,
) -> Result<Response, ExecHttpError> {
    let host = request_host(&headers);
    let ip = client_ip(&headers, addr);
    match crate::exec::artifacts::get(&id) {
        Some((bytes, ctype, bound)) => {
            if let Some(gid) = bound {
                let gid = GraphId(gid);
                guard_graph_read(&headers, &host, &ip, &gid, query.token.as_deref())?;
            }
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, ctype)
                .header(header::CACHE_CONTROL, "private, max-age=300")
                .body(Body::from(bytes.as_ref().clone()))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()))
        }
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

async fn get_upload(Path(id): Path<String>) -> Response {
    match crate::exec::uploads::take(&id) {
        Some((bytes, ctype)) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, ctype)
            .header(header::CACHE_CONTROL, "private, max-age=300")
            .body(Body::from(bytes))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Multipart upload (`file` field). Distinct from trusted `public/`.
/// Auth: exec API key, or open when `RESUMA_EXEC_PUBLIC=1` (dev).
async fn upload_multipart(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<crate::exec::uploads::UploadReceipt>, ExecHttpError> {
    let ip = client_ip(&headers, addr);
    let cfg = security::config();
    if cfg.requires_api_key() {
        security::guard_admin(&headers, &request_host(&headers), &ip, None)?;
    }

    let (bytes, content_type, _filename) = read_multipart_file(&mut multipart).await?;
    let receipt = crate::exec::uploads::store(bytes, &content_type)?;
    Ok(Json(receipt))
}

/// Named `#[upload]` handler — multipart field `file`.
async fn upload_named(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(name): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<Value>, ExecHttpError> {
    let ip = client_ip(&headers, addr);
    validate_resource_name(&name)?;
    let cfg = security::config();
    if cfg.requires_api_key() {
        security::guard_admin(&headers, &request_host(&headers), &ip, None)?;
    }

    let (meta, run) = crate::exec::uploads::lookup_upload(&name)
        .ok_or_else(|| ResumaError::Validation(format!("unknown upload handler `{name}`")))?;

    let (bytes, content_type, filename) = read_multipart_file(&mut multipart).await?;
    let file = crate::exec::uploads::UploadedFile {
        bytes,
        content_type,
        filename,
    };
    crate::exec::uploads::validate_uploaded(&file, &meta)?;
    let value = run(file).await?;
    Ok(Json(value))
}

async fn read_multipart_file(
    multipart: &mut Multipart,
) -> Result<(Vec<u8>, String, Option<String>), ResumaError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut content_type = String::from("application/octet-stream");
    let mut filename = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ResumaError::Validation(format!("multipart: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name != "file" && name != "upload" && name != "heightmap" {
            continue;
        }
        filename = field.file_name().map(|s| s.to_string());
        if let Some(ct) = field.content_type() {
            content_type = ct.to_string();
        }
        let data = field
            .bytes()
            .await
            .map_err(|e| ResumaError::Validation(format!("multipart read: {e}")))?;
        file_bytes = Some(data.to_vec());
        break;
    }
    let bytes = file_bytes
        .ok_or_else(|| ResumaError::Validation("multipart must include field `file`".into()))?;
    Ok((bytes, content_type, filename))
}

async fn replay_graph(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<GraphTokenQuery>,
) -> Result<Json<Vec<WorkerEvent>>, ExecHttpError> {
    let host = request_host(&headers);
    let ip = client_ip(&headers, addr);
    validate_graph_id(&id)?;
    let gid = GraphId(id);
    guard_graph_read(&headers, &host, &ip, &gid, query.token.as_deref())?;
    FlowEngine::replay(&gid)
        .map(Json)
        .ok_or(ExecHttpError(ResumaError::UnknownGraph(gid.0)))
}

async fn pause_graph(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<GraphTokenQuery>,
) -> Result<StatusCode, ExecHttpError> {
    let host = request_host(&headers);
    let ip = client_ip(&headers, addr);
    validate_graph_id(&id)?;
    let gid = GraphId(id);
    guard_graph_control(&headers, &host, &ip, &gid, query.token.as_deref(), None)?;
    FlowEngine::pause(&gid)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn cancel_graph(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<GraphTokenQuery>,
) -> Result<StatusCode, ExecHttpError> {
    let host = request_host(&headers);
    let ip = client_ip(&headers, addr);
    validate_graph_id(&id)?;
    let gid = GraphId(id);
    guard_graph_control(&headers, &host, &ip, &gid, query.token.as_deref(), None)?;
    FlowEngine::cancel(&gid)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn resume_graph(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<GraphTokenQuery>,
) -> Result<Json<StartWorkerResponse>, ExecHttpError> {
    let host = request_host(&headers);
    let ip = client_ip(&headers, addr);
    validate_graph_id(&id)?;
    let gid = GraphId(id);
    guard_graph_control(&headers, &host, &ip, &gid, query.token.as_deref(), None)?;
    Ok(Json(FlowEngine::resume(&gid).await?))
}

type GraphEventStream = Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>;

fn graph_sse_stream(bus: SharedEventBus) -> GraphEventStream {
    let mut rx = bus.subscribe();
    Box::pin(stream! {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    if let Ok(data) = serde_json::to_string(&event) {
                        yield Ok(Event::default().data(data));
                    }
                }
                Err(RecvError::Lagged(n)) => {
                    // Client must re-fetch `/replay` — do not silently drop gaps.
                    let data = serde_json::json!({ "type": "resync", "skipped": n });
                    yield Ok(
                        Event::default()
                            .event("resync")
                            .data(data.to_string()),
                    );
                }
                Err(RecvError::Closed) => break,
            }
        }
    })
}

fn graph_sse_replay(gid: &GraphId) -> GraphEventStream {
    let history = super::durable::load_events(gid).unwrap_or_default();
    Box::pin(stream! {
        for event in history {
            if let Ok(data) = serde_json::to_string(&event) {
                yield Ok(Event::default().data(data));
            }
        }
    })
}

async fn graph_events_sse(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<GraphTokenQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ExecHttpError> {
    let host = request_host(&headers);
    let ip = client_ip(&headers, addr);
    validate_graph_id(&id)?;
    let gid = GraphId(id.clone());
    guard_graph_read(&headers, &host, &ip, &gid, query.token.as_deref())?;

    let stream: GraphEventStream = if let Some(bus) = FlowEngine::bus(&gid) {
        graph_sse_stream(bus)
    } else {
        graph_sse_replay(&gid)
    };

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    ))
}

#[derive(Debug)]
struct ExecHttpError(ResumaError);

impl IntoResponse for ExecHttpError {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.0.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = serde_json::json!({ "ok": false, "error": self.0.client_message(true) });
        (
            [(header::CONTENT_TYPE, "application/json")],
            (status, Json(body)),
        )
            .into_response()
    }
}

impl From<ResumaError> for ExecHttpError {
    fn from(e: ResumaError) -> Self {
        Self(e)
    }
}
