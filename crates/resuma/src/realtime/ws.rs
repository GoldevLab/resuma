use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Forwards a channel of pre-serialized text messages to a WebSocket sink so
/// callers manage `writer.tx.send(json_string)` instead of hand-writing a
/// `tokio::spawn` loop around `sink.send(Message::Text(..))` for every app.
///
/// Pair with `socket.split()`; drive the stream half with your own read loop
/// (message parsing/routing stays app-specific — see [`classify_frame`]).
///
/// ```rust,ignore
/// let (sink, mut stream) = socket.split();
/// let writer = spawn_ws_writer(sink);
/// writer.tx.send(json.to_string()).ok();
/// // ...read loop over `stream`...
/// writer.abort();
/// ```
pub struct WsWriter {
    pub tx: mpsc::UnboundedSender<String>,
    handle: JoinHandle<()>,
}

impl WsWriter {
    /// Stop the writer task. Call once the read loop ends (connection closed);
    /// dropping [`Self::tx`] alone also ends the loop once the channel drains.
    pub fn abort(&self) {
        self.handle.abort();
    }
}

pub fn spawn_ws_writer(mut sink: SplitSink<WebSocket, Message>) -> WsWriter {
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let handle = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sink.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });
    WsWriter { tx, handle }
}

/// Classification of an inbound WS frame for a typical text-JSON protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InboundFrame {
    /// A text frame — the app should parse/route this.
    Text(String),
    /// Ping/pong/binary — not part of the app protocol, keep reading.
    Ignore,
    /// The client closed the connection — the read loop should stop.
    Close,
}

/// Classifies a raw [`Message`], generalizing the
/// `Text => parse, Ping|Pong => continue, Close => break` match every
/// hand-rolled WS read loop repeats.
pub fn classify_frame(msg: Message) -> InboundFrame {
    match msg {
        Message::Text(t) => InboundFrame::Text(t.to_string()),
        Message::Close(_) => InboundFrame::Close,
        Message::Ping(_) | Message::Pong(_) | Message::Binary(_) => InboundFrame::Ignore,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_frame_matches_expected_variants() {
        assert_eq!(
            classify_frame(Message::Text("hi".into())),
            InboundFrame::Text("hi".to_string())
        );
        assert_eq!(classify_frame(Message::Close(None)), InboundFrame::Close);
        assert_eq!(
            classify_frame(Message::Ping(Default::default())),
            InboundFrame::Ignore
        );
        assert_eq!(
            classify_frame(Message::Pong(Default::default())),
            InboundFrame::Ignore
        );
    }
}
