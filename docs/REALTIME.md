# Resuma realtime — WebSocket rooms

`resuma::realtime` provides reusable WebSocket **room / peer** primitives for
multiplayer apps (lobbies, shared cursors, chat channels, game presence).

It is **not** a message protocol and **not** a replacement for `#[worker]` / SSE
(those are for async batch jobs). Apps still define their own JSON envelope;
Resuma owns who is connected, broadcast-except-self, stale peer sweeps, and
per-message-kind rate limiting.

## Types

| Type | Role |
|------|------|
| `Room<S>` | Join / leave / `broadcast_except` / `sweep_stale` for peer state `S` |
| `RoomRegistry<K, S>` | Thread-safe map of rooms keyed by lobby / world id |
| `Peer` | Outbound channel + `touch()` + `allow_rate(kind, min_interval)` |
| `spawn_ws_writer` | Spawn a task that drains a channel into an Axum WS sink |
| `classify_frame` | Map `Message` → `InboundFrame::{Text, Close, Ignore}` |

## Minimal pattern

```rust
use resuma::realtime::{Room, RoomRegistry, classify_frame, spawn_ws_writer, InboundFrame};
use axum::extract::ws::WebSocket;
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use std::time::Duration;

#[derive(Clone, Default)]
struct Pose { x: f32, z: f32 }

static ROOMS: Lazy<RoomRegistry<u32, Pose>> = Lazy::new(RoomRegistry::new);

async fn handle_socket(socket: WebSocket, room_key: u32, my_id: String) {
    let (sink, mut stream) = socket.split();
    let writer = spawn_ws_writer(sink);
    ROOMS.with_room_or_insert(room_key, || Room::new(32), |room| {
        let _ = room.join(my_id.clone(), Pose::default(), writer.tx.clone());
    });

    while let Some(Ok(msg)) = stream.next().await {
        match classify_frame(msg) {
            InboundFrame::Text(text) => {
                ROOMS.with_room(&room_key, |room| {
                    if let Some(peer) = room.get_mut(&my_id) {
                        peer.touch();
                        if peer.allow_rate("pose", Duration::from_millis(30)) {
                            room.broadcast_except(&my_id, &text);
                        }
                    }
                });
            }
            InboundFrame::Close => break,
            InboundFrame::Ignore => {}
        }
    }

    writer.abort();
    ROOMS.with_room(&room_key, |room| { room.leave(&my_id); });
    ROOMS.remove_if_empty(&room_key);
}
```

Mount the upgrade handler on your own Axum route (or via `FlowApp` / `ResumaApp`
custom routes). Resuma does not auto-mount a `/_resuma/ws` endpoint — the app
owns auth, room keys, and the wire format.

## When to use what

| Need | Use |
|------|-----|
| Long-running bake / AI job with progress UI | `#[worker]` + SSE / `FlowEngine` |
| Tick-rate presence / multiplayer sync | `resuma::realtime` |
| Form / RPC mutations | `#[submit]` / `#[server]` |

## See also

- Module docs: [`resuma::realtime`](https://docs.rs/resuma/latest/resuma/realtime/)
- Security defaults for HTTP (CSRF does **not** apply to raw WebSockets — authenticate
  the upgrade yourself): [SECURITY.md](./SECURITY.md)
