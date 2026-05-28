//! Dev-mode WebSocket bridge for granular HMR notifications.

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use tokio::sync::broadcast;

static DEV_TX: Lazy<broadcast::Sender<String>> = Lazy::new(|| {
    let (tx, _) = broadcast::channel(64);
    tx
});

/// Whether dev tooling (WebSocket HMR) is enabled.
pub fn dev_mode_enabled() -> bool {
    matches!(
        std::env::var("RESUMA_DEV").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

/// Inline script injected in dev mode so every page reloads after `cargo watch` rebuilds.
pub fn dev_reload_script() -> String {
    if !dev_mode_enabled() {
        return String::new();
    }
    r#"<script>
(function () {
  window.__resumaDev = true;
  if (typeof WebSocket === "undefined") return;
  var proto = location.protocol === "https:" ? "wss" : "ws";
  var hadConnection = false;
  function connect() {
    var ws = new WebSocket(proto + "://" + location.host + "/_resuma/dev/ws");
    ws.addEventListener("open", function () {
      if (hadConnection) location.reload();
      hadConnection = true;
    });
    ws.addEventListener("message", function (ev) {
      if (String(ev.data) === "reload") location.reload();
    });
    ws.addEventListener("close", function () {
      setTimeout(connect, 500);
    });
    ws.addEventListener("error", function () {
      ws.close();
    });
  }
  connect();
})();
</script>"#
        .into()
}

/// Broadcast a dev event to connected browsers (`reload`, `island:instance-id`, …).
pub fn broadcast_dev_event(event: impl Into<String>) {
    if dev_mode_enabled() {
        let _ = DEV_TX.send(event.into());
    }
}

pub async fn dev_ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = DEV_TX.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if matches!(msg, Message::Close(_)) {
                break;
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };
}

/// Shared handle for tests.
#[doc(hidden)]
pub fn dev_broadcast_sender() -> Arc<broadcast::Sender<String>> {
    Arc::new(DEV_TX.clone())
}
