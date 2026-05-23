//! Effects and computed values.
//!
//! Effects re-execute when any of the signals they depend on change. On the
//! server, effects are only used as part of `use_computed` so that derived
//! state is captured during SSR. The client runtime maintains its own,
//! mirrored effect graph.

use std::sync::Arc;

use parking_lot::RwLock;
use serde::Serialize;

use crate::context::current_context;
use crate::signal::{Signal, SignalId};

/// Opaque effect id. Stable within a single render pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectId(pub u32);

/// A user-supplied side effect bound to a closure.
pub struct Effect {
    pub id: EffectId,
    callback: Arc<RwLock<Box<dyn FnMut() + Send + Sync>>>,
}

impl Effect {
    pub fn run(&self) {
        if let Some(ctx) = current_context() {
            ctx.set_current_effect(Some(self.id));
        }
        (self.callback.write())();
        if let Some(ctx) = current_context() {
            ctx.set_current_effect(None);
        }
    }
}

/// Schedule a side effect. The closure runs once immediately and then again
/// whenever any tracked signal changes.
pub fn use_effect<F>(mut callback: F) -> Effect
where
    F: FnMut() + Send + Sync + 'static,
{
    let id = current_context()
        .map(|c| EffectId(c.next_effect_id()))
        .unwrap_or(EffectId(0));

    let cb: Arc<RwLock<Box<dyn FnMut() + Send + Sync>>> =
        Arc::new(RwLock::new(Box::new(move || callback())));

    if let Some(ctx) = current_context() {
        let cb_clone = cb.clone();
        ctx.register_effect(id, move || {
            (cb_clone.write())();
        });
    }

    let eff = Effect { id, callback: cb };
    eff.run();
    eff
}

/// Reactive derived value.
pub struct Computed<T: Clone + Serialize + Send + Sync + 'static> {
    signal: Signal<T>,
    #[allow(dead_code)]
    effect: Effect,
}

impl<T: Clone + Serialize + Send + Sync + 'static> Computed<T> {
    pub fn id(&self) -> SignalId { self.signal.id() }
    pub fn get(&self) -> T { self.signal.get() }
    pub fn peek(&self) -> T { self.signal.peek() }
}

pub fn use_computed<T, F>(mut compute: F) -> Computed<T>
where
    T: Clone + Serialize + Send + Sync + 'static,
    F: FnMut() -> T + Send + Sync + 'static,
{
    let initial = compute();
    let signal = Signal::new(initial);

    let signal_for_effect = signal.clone();
    let effect = use_effect(move || {
        let next = compute();
        signal_for_effect.set(next);
    });

    Computed { signal, effect }
}
