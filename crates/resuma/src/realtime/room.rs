use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tokio::sync::mpsc;

use super::peer::Peer;

/// Session-scoped peer identifier (a connection id, not necessarily a stable user
/// id — apps are free to make it one, e.g. an authenticated account id).
pub type PeerId = String;

/// The room has reached [`Room::new`]'s `max_peers` and rejected a join.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoomFull;

impl std::fmt::Display for RoomFull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "room full")
    }
}

impl std::error::Error for RoomFull {}

/// A set of peers sharing presence/state — one game world, one lobby, one chat
/// channel. `S` is app-defined per-peer state (position, name, flags, …); Resuma
/// only stores and returns it.
///
/// Extracted from a real multiplayer game's hand-rolled `HashMap<String, PeerLive>`
/// + broadcast/sweep helpers so future apps do not re-derive the same ~150 lines.
pub struct Room<S> {
    peers: HashMap<PeerId, Peer<S>>,
    max_peers: usize,
}

impl<S> Room<S> {
    pub fn new(max_peers: usize) -> Self {
        Self {
            peers: HashMap::new(),
            max_peers,
        }
    }

    pub fn len(&self) -> usize {
        self.peers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    pub fn get(&self, id: &str) -> Option<&Peer<S>> {
        self.peers.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Peer<S>> {
        self.peers.get_mut(id)
    }

    /// Every peer, in arbitrary order.
    pub fn peers(&self) -> impl Iterator<Item = &Peer<S>> {
        self.peers.values()
    }

    /// Every peer except `except` (pass `""` to include everyone).
    pub fn peers_except<'a>(&'a self, except: &'a str) -> impl Iterator<Item = &'a Peer<S>> {
        self.peers.values().filter(move |p| p.id != except)
    }

    /// Insert a new peer. If `id` was already connected (e.g. a browser tab
    /// refresh reusing a client-generated id), the stale entry is replaced and
    /// returned so the caller can broadcast a "left" event for it before
    /// announcing the new join.
    ///
    /// Errors with [`RoomFull`] once `max_peers` is reached and `id` is new.
    pub fn join(
        &mut self,
        id: PeerId,
        state: S,
        tx: mpsc::UnboundedSender<String>,
    ) -> Result<Option<Peer<S>>, RoomFull> {
        let replaced = self.peers.remove(&id);
        if replaced.is_none() && self.peers.len() >= self.max_peers {
            return Err(RoomFull);
        }
        self.peers.insert(id.clone(), Peer::new(id, state, tx));
        Ok(replaced)
    }

    pub fn leave(&mut self, id: &str) -> Option<Peer<S>> {
        self.peers.remove(id)
    }

    /// Send `msg` to every peer except `except` (pass `""` to include everyone).
    pub fn broadcast_except(&self, except: &str, msg: &str) {
        for peer in self.peers_except(except) {
            peer.send(msg.to_string());
        }
    }

    pub fn broadcast(&self, msg: &str) {
        self.broadcast_except("", msg);
    }

    /// Remove peers whose [`Peer::last_seen`] is older than `timeout` and return
    /// their ids so callers can broadcast a "left" event / clean up app-side state.
    ///
    /// Browser tabs get heavily throttled in the background (timers can pause for
    /// tens of seconds while the socket stays open), so `timeout` should be
    /// generous — minutes, not seconds — or reconnecting tabs look like churn.
    pub fn sweep_stale(&mut self, timeout: Duration) -> Vec<PeerId> {
        let cutoff = Instant::now() - timeout;
        let stale: Vec<PeerId> = self
            .peers
            .iter()
            .filter(|(_, p)| p.last_seen < cutoff)
            .map(|(id, _)| id.clone())
            .collect();
        for id in &stale {
            self.peers.remove(id);
        }
        stale
    }
}

/// Thread-safe registry of [`Room`]s keyed by an app-defined room key `K` (world
/// seed, lobby id, channel name, …). Intended to live behind a
/// `once_cell::sync::Lazy` static, matching how apps already keep one room map
/// per process.
pub struct RoomRegistry<K, S> {
    rooms: Mutex<HashMap<K, Room<S>>>,
}

impl<K: Eq + Hash + Clone, S> RoomRegistry<K, S> {
    pub fn new() -> Self {
        Self {
            rooms: Mutex::new(HashMap::new()),
        }
    }

    /// Runs `f` with exclusive access to the room for `key`, creating it via
    /// `make` on first use.
    pub fn with_room_or_insert<R>(
        &self,
        key: K,
        make: impl FnOnce() -> Room<S>,
        f: impl FnOnce(&mut Room<S>) -> R,
    ) -> R {
        let mut rooms = self.rooms.lock();
        let room = rooms.entry(key).or_insert_with(make);
        f(room)
    }

    /// Runs `f` with exclusive access to an existing room, or returns `None` if
    /// `key` has no room yet (never auto-creates — use [`Self::with_room_or_insert`]
    /// on the join path).
    pub fn with_room<R>(&self, key: &K, f: impl FnOnce(&mut Room<S>) -> R) -> Option<R> {
        let mut rooms = self.rooms.lock();
        rooms.get_mut(key).map(f)
    }

    /// Drop the room for `key` if it currently has no peers. Call after
    /// [`Room::leave`] so abandoned worlds/lobbies do not accumulate forever.
    pub fn remove_if_empty(&self, key: &K) {
        let mut rooms = self.rooms.lock();
        if rooms.get(key).is_some_and(Room::is_empty) {
            rooms.remove(key);
        }
    }

    pub fn room_count(&self) -> usize {
        self.rooms.lock().len()
    }
}

impl<K: Eq + Hash + Clone, S> Default for RoomRegistry<K, S> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn channel() -> (
        mpsc::UnboundedSender<String>,
        mpsc::UnboundedReceiver<String>,
    ) {
        mpsc::unbounded_channel()
    }

    #[test]
    fn join_replaces_stale_same_id_connection() {
        let mut room: Room<i32> = Room::new(8);
        let (tx1, _rx1) = channel();
        let (tx2, _rx2) = channel();
        assert!(room.join("p1".into(), 1, tx1).unwrap().is_none());
        let replaced = room.join("p1".into(), 2, tx2).unwrap();
        assert!(replaced.is_some());
        assert_eq!(room.len(), 1);
        assert_eq!(room.get("p1").unwrap().state, 2);
    }

    #[test]
    fn join_rejects_when_full() {
        let mut room: Room<()> = Room::new(1);
        let (tx1, _rx1) = channel();
        let (tx2, _rx2) = channel();
        room.join("p1".into(), (), tx1).unwrap();
        assert!(matches!(room.join("p2".into(), (), tx2), Err(RoomFull)));
    }

    #[test]
    fn broadcast_except_skips_self() {
        let mut room: Room<()> = Room::new(8);
        let (tx1, mut rx1) = channel();
        let (tx2, mut rx2) = channel();
        room.join("p1".into(), (), tx1).unwrap();
        room.join("p2".into(), (), tx2).unwrap();
        room.broadcast_except("p1", "hello");
        assert!(rx1.try_recv().is_err());
        assert_eq!(rx2.try_recv().unwrap(), "hello");
    }

    #[test]
    fn sweep_stale_removes_and_reports_timed_out_peers() {
        let mut room: Room<()> = Room::new(8);
        let (tx1, _rx1) = channel();
        room.join("p1".into(), (), tx1).unwrap();
        room.get_mut("p1").unwrap().last_seen = Instant::now() - Duration::from_secs(200);
        let removed = room.sweep_stale(Duration::from_secs(120));
        assert_eq!(removed, vec!["p1".to_string()]);
        assert!(room.is_empty());
    }

    #[test]
    fn registry_creates_room_lazily_and_cleans_up_when_empty() {
        let registry: RoomRegistry<u32, ()> = RoomRegistry::new();
        let (tx, _rx) = channel();
        registry.with_room_or_insert(
            42,
            || Room::new(8),
            |room| {
                room.join("p1".into(), (), tx).unwrap();
            },
        );
        assert_eq!(registry.room_count(), 1);
        registry.with_room(&42, |room| {
            room.leave("p1");
        });
        registry.remove_if_empty(&42);
        assert_eq!(registry.room_count(), 0);
    }
}
