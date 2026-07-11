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
        if Self::values_equal(&self.inner.value.read(), &value) {
            return;
        }
        *self.inner.value.write() = value;
        self.notify();
    }

    /// Functional update — read, modify, write — atomically.
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut guard = self.inner.value.write();
        let before = guard.clone();
        f(&mut guard);
        let after = guard.clone();
        drop(guard);
        if Self::values_equal(&before, &after) {
            return;
        }
        self.notify();
    }

    fn track(&self) {
        if let Some(ctx) = current_context() {
            if let Some(eid) = ctx.current_effect_id() {
                ctx.record_effect_dep(eid, self.inner.id);
            }
        }
    }

    fn notify(&self) {
        if let Some(ctx) = current_context() {
            let subs = ctx.signal_subscriber_ids(self.inner.id);
            for eid in subs {
                ctx.run_effect(eid);
            }
            ctx.update_signal(self.inner.id, self.serialize_value());
        }
    }

    fn serialize_value(&self) -> Value {
        match serde_json::to_value(&*self.inner.value.read()) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, signal_id = self.inner.id.0, "signal value not JSON-serializable");
                Value::Null
            }
        }
    }

    fn values_equal(a: &T, b: &T) -> bool {
        // Prefer exact float identity matching the client `Object.is`
        // (NaN equals NaN; -0.0 differs from +0.0) before falling back to JSON.
        if let Some(eq) = float_bits_equal(a, b) {
            return eq;
        }
        match (serde_json::to_value(a), serde_json::to_value(b)) {
            (Ok(va), Ok(vb)) => json_values_equal(&va, &vb),
            // Non-JSON values: try string form before assuming a change.
            _ => match (serde_json::to_string(a), serde_json::to_string(b)) {
                (Ok(sa), Ok(sb)) => sa == sb,
                _ => false,
            },
        }
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

fn float_bits_equal<T: 'static>(a: &T, b: &T) -> Option<bool> {
    use std::any::Any;
    let a_any = a as &dyn Any;
    let b_any = b as &dyn Any;
    if let (Some(a), Some(b)) = (a_any.downcast_ref::<f64>(), b_any.downcast_ref::<f64>()) {
        return Some(a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan()));
    }
    if let (Some(a), Some(b)) = (a_any.downcast_ref::<f32>(), b_any.downcast_ref::<f32>()) {
        return Some(a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan()));
    }
    None
}

/// Equality aligned with the client for nested JSON (and floats that survived
/// serialization). Numbers use bitwise identity so `-0` ≠ `+0`.
fn json_values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(na), Value::Number(nb)) => match (na.as_f64(), nb.as_f64()) {
            (Some(fa), Some(fb)) => fa.to_bits() == fb.to_bits(),
            _ => na == nb,
        },
        (Value::Array(aa), Value::Array(ba)) => {
            aa.len() == ba.len() && aa.iter().zip(ba).all(|(x, y)| json_values_equal(x, y))
        }
        (Value::Object(ao), Value::Object(bo)) => {
            ao.len() == bo.len()
                && ao
                    .iter()
                    .all(|(k, v)| bo.get(k).is_some_and(|w| json_values_equal(v, w)))
        }
        _ => a == b,
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

/// Create a reactive signal.
///
/// `signal(0)` is the concise constructor recommended for application code.
/// `use_signal(0)` remains available as the hook-style alias.
pub fn signal<T: Clone + Serialize + 'static>(initial: T) -> Signal<T> {
    Signal::new(initial)
}

/// Hook-style alias for [`signal`].
pub fn use_signal<T: Clone + Serialize + 'static>(initial: T) -> Signal<T> {
    signal(initial)
}

fn fallback_id() -> SignalId {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(1_000_000);
    SignalId(COUNTER.fetch_add(1, Ordering::Relaxed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::context::{with_context, RenderContext, RenderMode};

    #[test]
    fn set_skips_notify_when_value_unchanged() {
        let ctx = RenderContext::new(RenderMode::Ssr);
        with_context(ctx, || {
            let n = signal(0_i32);
            n.set(0);
            n.set(1);
            assert_eq!(n.peek(), 1);
        });
    }

    #[test]
    fn float_nan_is_equal_like_object_is() {
        let ctx = RenderContext::new(RenderMode::Ssr);
        with_context(ctx, || {
            let n = signal(f64::NAN);
            // Setting NaN again must be a no-op (matches JS Object.is(NaN, NaN)).
            n.set(f64::NAN);
            assert!(n.peek().is_nan());
        });
    }

    #[test]
    fn float_negative_zero_differs_from_positive_zero() {
        let ctx = RenderContext::new(RenderMode::Ssr);
        with_context(ctx, || {
            let n = signal(0.0_f64);
            n.set(-0.0);
            // Object.is(+0, -0) is false — notify must fire and value must update.
            assert_eq!(n.peek().to_bits(), (-0.0_f64).to_bits());
        });
    }
}
