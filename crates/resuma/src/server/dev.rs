//! Dev-mode WebSocket bridge for granular HMR notifications.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use tokio::sync::{broadcast, Notify};

static DEV_TX: Lazy<broadcast::Sender<String>> = Lazy::new(|| {
    let (tx, _) = broadcast::channel(64);
    tx
});

static DEV_STOP: Lazy<Notify> = Lazy::new(Notify::new);
static DEV_STOPPING: AtomicBool = AtomicBool::new(false);

/// Whether dev tooling (WebSocket HMR) is enabled.
pub fn dev_mode_enabled() -> bool {
    matches!(
        std::env::var("RESUMA_DEV").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

/// Close every attached HMR socket so graceful shutdown does not wait on
/// browsers. `cargo watch` can then restart on the same port immediately.
pub fn notify_dev_shutdown() {
    DEV_STOPPING.store(true, Ordering::SeqCst);
    DEV_STOP.notify_waiters();
}

/// Inline script injected in dev mode so every page reloads after `cargo watch`
/// rebuilds. This is the **only** HMR WebSocket (core.js listens for
/// `resuma:dev` events — it must not open a second socket).
pub fn dev_reload_script(nonce: &str) -> String {
    if !dev_mode_enabled() {
        return String::new();
    }
    let nonce_attr = if nonce.is_empty() {
        String::new()
    } else {
        format!(r#" nonce="{}""#, nonce)
    };
    format!(
        r#"<script{}>
(function () {{
  window.__resumaDev = true;
  if (typeof WebSocket === "undefined") return;
  var proto = location.protocol === "https:" ? "wss" : "ws";
  var hadConnection = false;
  var delay = 400;
  var timer;
  function connect() {{
    var ws = new WebSocket(proto + "://" + location.host + "/_resuma/dev/ws");
    ws.addEventListener("open", function () {{
      delay = 400;
      if (hadConnection) location.reload();
      hadConnection = true;
    }});
    ws.addEventListener("message", function (ev) {{
      var data = String(ev.data);
      if (data === "reload") {{ location.reload(); return; }}
      try {{
        window.dispatchEvent(new CustomEvent("resuma:dev", {{ detail: data }}));
      }} catch (e) {{}}
    }});
    ws.addEventListener("close", function () {{
      if (timer) clearTimeout(timer);
      timer = setTimeout(connect, delay);
      delay = Math.min(delay * 1.5, 2000);
    }});
    ws.addEventListener("error", function () {{
      try {{ ws.close(); }} catch (e) {{}}
    }});
  }}
  connect();
}})();
</script>"#,
        nonce_attr
    )
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
    if DEV_STOPPING.load(Ordering::SeqCst) {
        return;
    }

    let (mut sender, mut receiver) = socket.split();
    let mut rx = DEV_TX.subscribe();

    loop {
        if DEV_STOPPING.load(Ordering::SeqCst) {
            let _ = sender.send(Message::Close(None)).await;
            break;
        }
        tokio::select! {
            biased;
            _ = DEV_STOP.notified() => {
                let _ = sender.send(Message::Close(None)).await;
                break;
            }
            msg = rx.recv() => {
                match msg {
                    Ok(text) => {
                        if sender.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            incoming = receiver.next() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }
}

/// Shared handle for tests.
#[doc(hidden)]
pub fn dev_broadcast_sender() -> Arc<broadcast::Sender<String>> {
    Arc::new(DEV_TX.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_dev<T>(on: bool, f: impl FnOnce() -> T) -> T {
        let prev = std::env::var_os("RESUMA_DEV");
        if on {
            std::env::set_var("RESUMA_DEV", "1");
        } else {
            std::env::remove_var("RESUMA_DEV");
        }
        let out = f();
        match prev {
            Some(v) => std::env::set_var("RESUMA_DEV", v),
            None => std::env::remove_var("RESUMA_DEV"),
        }
        out
    }

    #[test]
    fn reload_script_empty_without_dev() {
        with_dev(false, || {
            assert!(dev_reload_script("n").is_empty());
        });
    }

    #[test]
    fn reload_script_owns_socket_and_dispatches() {
        with_dev(true, || {
            let s = dev_reload_script("abc");
            assert!(s.contains(r#"nonce="abc""#));
            assert!(s.contains("window.__resumaDev = true"));
            assert!(s.contains("resuma:dev"));
            assert!(s.contains("Math.min"));
            assert!(s.contains("/_resuma/dev/ws"));
        });
    }
}
