//! Reusable WebSocket "room" primitives for realtime multiplayer apps.
//!
//! Resuma's exec layer (`#[worker]`, SSE progress) is built for async batch jobs,
//! not tick-rate multiplayer — the dev-mode HMR socket in [`crate::server`] is the
//! only other WebSocket usage in the framework. Apps that need presence/rooms
//! (a game lobby, a shared cursor, a chat channel) previously hand-rolled peer
//! bookkeeping, rate limiting, and broadcast directly on `axum::extract::ws`.
//!
//! `realtime` extracts that boilerplate — **not** a message protocol. Apps still
//! define and parse their own JSON envelope; Resuma only owns "who is connected",
//! "send to everyone but me", "drop peers that went quiet", and "don't let one
//! peer spam this message type".
//!
//! ```rust,ignore
//! use resuma::realtime::{Room, RoomRegistry, classify_frame, spawn_ws_writer, InboundFrame};
//! use axum::extract::ws::{WebSocket, WebSocketUpgrade};
//! use futures_util::StreamExt;
//! use once_cell::sync::Lazy;
//! use std::time::Duration;
//!
//! #[derive(Clone, Default)]
//! struct Pose { x: f32, z: f32 }
//!
//! static ROOMS: Lazy<RoomRegistry<u32, Pose>> = Lazy::new(RoomRegistry::new);
//!
//! async fn handle_socket(socket: WebSocket, room_key: u32, my_id: String) {
//!     let (sink, mut stream) = socket.split();
//!     let writer = spawn_ws_writer(sink);
//!     ROOMS.with_room_or_insert(room_key, || Room::new(32), |room| {
//!         let _ = room.join(my_id.clone(), Pose::default(), writer.tx.clone());
//!     });
//!
//!     while let Some(Ok(msg)) = stream.next().await {
//!         match classify_frame(msg) {
//!             InboundFrame::Text(text) => {
//!                 ROOMS.with_room(&room_key, |room| {
//!                     if let Some(peer) = room.get_mut(&my_id) {
//!                         peer.touch();
//!                         if peer.allow_rate("pose", Duration::from_millis(30)) {
//!                             room.broadcast_except(&my_id, &text);
//!                         }
//!                     }
//!                 });
//!             }
//!             InboundFrame::Close => break,
//!             InboundFrame::Ignore => {}
//!         }
//!     }
//!
//!     writer.abort();
//!     ROOMS.with_room(&room_key, |room| { room.leave(&my_id); });
//!     ROOMS.remove_if_empty(&room_key);
//! }
//! ```

mod peer;
mod room;
mod ws;

pub use peer::Peer;
pub use room::{PeerId, Room, RoomFull, RoomRegistry};
pub use ws::{classify_frame, spawn_ws_writer, InboundFrame, WsWriter};
