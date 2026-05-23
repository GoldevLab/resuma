//! Fine-grained reactive primitives.
//!
//! Signals are the unit of reactivity. They have a stable id assigned by the
//! current `RenderContext` so that the SSR pass can serialize them and the
//! client runtime can match them up by id.

use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::context::current_context;

/// Globally unique id of a reactive primitive within a single render pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SignalId(pub u32);

impl std::fmt::Display for SignalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "s{}", self.0)
    }
}

/// Inner state shared by every clone of a `Signal<T>`.
struct SignalInner<T> {
    id: SignalId,
    value: RwLock<T>,
    /// Subscribers tracked during effect execution. On the server we only
    /// care about *which* effects depend on this signal — actual notification
    /// is performed by [`super::effect::Effect::trigger`].
    subscribers: RwLock<Vec<u32>>,
}

/// A reactive cell whose changes notify subscribers. Cheap to clone (Arc).
pub struct Signal<T> {
    inner: Arc<SignalInner<T>>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Signal<T>
where
    T: Clone + Serialize + 'static,
{
    /// Create a new signal. Allocates a fresh `SignalId` from the active
    /// `RenderContext` (or a fallback global counter when called outside of
    /// SSR — useful in unit tests).
    pub fn new(initial: T) -> Self {
        let id = current_context()
            .map(|ctx| ctx.next_signal_id())
            .unwrap_or_else(fallback_id);

        let signal = Self {
            inner: Arc::new(SignalInner {
                id,
                value: RwLock::new(initial),
                subscribers: RwLock::new(Vec::new()),
            }),
        };

        if let Some(ctx) = current_context() {
            ctx.register_signal(id, signal.serialize_value());
        }

        signal
    }

    pub fn id(&self) -> SignalId {
        self.inner.id
    }

    /// Read the current value (without dependency tracking).
    pub fn peek(&self) -> T {
        self.inner.value.read().clone()
    }

    /// Read the current value and register the active effect (if any) as a
    /// dependency.
    pub fn get(&self) -> T {
        self.track();
        self.peek()
    }

    /// Replace the current value and notify subscribers.
    pub fn set(&self, value: T) {
        *self.inner.value.write() = value;
        self.notify();
    }

    /// Functional update — read, modify, write — atomically.
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut guard = self.inner.value.write();
        f(&mut guard);
        drop(guard);
        self.notify();
    }

    fn track(&self) {
        if let Some(ctx) = current_context() {
            if let Some(eid) = ctx.current_effect_id() {
                let mut subs = self.inner.subscribers.write();
                if !subs.contains(&eid) {
                    subs.push(eid);
                }
            }
        }
    }

    fn notify(&self) {
        if let Some(ctx) = current_context() {
            let subs = self.inner.subscribers.read().clone();
            for eid in subs {
                ctx.run_effect(eid);
            }
            ctx.update_signal(self.inner.id, self.serialize_value());
        }
    }

    fn serialize_value(&self) -> Value {
        serde_json::to_value(&*self.inner.value.read()).unwrap_or(Value::Null)
    }

    /// Split into a read-only and a write-only handle.
    pub fn split(self) -> (ReadSignal<T>, WriteSignal<T>) {
        (
            ReadSignal {
                signal: self.clone(),
            },
            WriteSignal { signal: self },
        )
    }
}

/// Read half of a signal returned by [`Signal::split`].
#[derive(Clone)]
pub struct ReadSignal<T> {
    signal: Signal<T>,
}

impl<T: Clone + Serialize + 'static> ReadSignal<T> {
    pub fn id(&self) -> SignalId {
        self.signal.id()
    }
    pub fn get(&self) -> T {
        self.signal.get()
    }
    pub fn peek(&self) -> T {
        self.signal.peek()
    }
}

/// Write half of a signal returned by [`Signal::split`].
#[derive(Clone)]
pub struct WriteSignal<T> {
    signal: Signal<T>,
}

impl<T: Clone + Serialize + 'static> WriteSignal<T> {
    pub fn id(&self) -> SignalId {
        self.signal.id()
    }
    pub fn set(&self, value: T) {
        self.signal.set(value)
    }
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        self.signal.update(f)
    }
}

/// Idiomatic constructor: `let count = use_signal(0);`.
pub fn use_signal<T: Clone + Serialize + 'static>(initial: T) -> Signal<T> {
    Signal::new(initial)
}

fn fallback_id() -> SignalId {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(1_000_000);
    SignalId(COUNTER.fetch_add(1, Ordering::Relaxed))
}
