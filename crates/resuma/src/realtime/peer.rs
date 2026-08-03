use std::collections::HashMap;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;

use super::room::PeerId;

/// One connected peer inside a [`Room`](super::Room): outbound channel, liveness,
/// and app-defined per-peer state `S` (position, name, flags, …).
///
/// Resuma does not interpret `S` — it is stored and handed back so apps can build
/// roster/snapshot messages without maintaining a second peer map of their own.
pub struct Peer<S> {
    pub id: PeerId,
    pub state: S,
    pub last_seen: Instant,
    tx: mpsc::UnboundedSender<String>,
    rate: HashMap<&'static str, Instant>,
}

impl<S> Peer<S> {
    pub(super) fn new(id: PeerId, state: S, tx: mpsc::UnboundedSender<String>) -> Self {
        Self {
            id,
            state,
            last_seen: Instant::now(),
            tx,
            rate: HashMap::new(),
        }
    }

    /// Send a pre-serialized text message to this peer only. Silently drops if the
    /// peer's writer task already exited (client disconnected mid-send) — callers
    /// should not treat this as fatal, the read loop will observe the close soon.
    pub fn send(&self, msg: impl Into<String>) {
        let _ = self.tx.send(msg.into());
    }

    /// Refresh liveness. Call on every inbound message, not just heartbeats, so a
    /// chatty-but-silent-on-`ping` client is not swept as stale.
    pub fn touch(&mut self) {
        self.last_seen = Instant::now();
    }

    /// Throttle a noisy message `kind` (e.g. `"pose"`, `"place"`, `"remove"`) to at
    /// most once per `min_interval`. Returns `true` the first time and again after
    /// each interval elapses, `false` otherwise.
    ///
    /// Generalizes the `last_pose` / `last_place` / `last_remove` / `last_sync`
    /// fields a hand-rolled multiplayer relay needs one-per-message-type — here it
    /// is a single map keyed by a `&'static str` tag apps choose per call site.
    pub fn allow_rate(&mut self, kind: &'static str, min_interval: Duration) -> bool {
        let now = Instant::now();
        match self.rate.get(kind) {
            Some(last) if now.duration_since(*last) < min_interval => false,
            _ => {
                self.rate.insert(kind, now);
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peer_with_rx(id: &str) -> (Peer<()>, mpsc::UnboundedReceiver<String>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Peer::new(id.to_string(), (), tx), rx)
    }

    #[test]
    fn allow_rate_gates_first_call_only_within_window() {
        let (mut p, _rx) = peer_with_rx("a");
        assert!(p.allow_rate("pose", Duration::from_secs(60)));
        assert!(!p.allow_rate("pose", Duration::from_secs(60)));
        // Different kind is independent.
        assert!(p.allow_rate("place", Duration::from_secs(60)));
    }

    #[test]
    fn send_does_not_panic_after_receiver_dropped() {
        let (p, rx) = peer_with_rx("a");
        drop(rx);
        p.send("hello"); // unbounded_channel send after drop just errors silently
    }
}
