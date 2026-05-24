//! Effects and computed values.
//!
//! Effects re-execute when any of the signals they depend on change.
//! SSR always runs effects once. When a client JS body is registered (via
//! [`use_computed_with_js`] or the `computed!` / `debounce!` macros), the
//! runtime replays them in the browser when dependencies change.

use std::collections::BTreeMap;
use std::sync::Arc;

use parking_lot::RwLock;
use serde::Serialize;

use super::context::{current_context, ClientEffectSpec};
use super::signal::{Signal, SignalId};

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
/// whenever any tracked signal changes (SSR only unless a client body is registered).
pub fn use_effect<F>(callback: F) -> Effect
where
    F: FnMut() + Send + Sync + 'static,
{
    let id = current_context()
        .map(|c| EffectId(c.next_effect_id()))
        .unwrap_or(EffectId(0));

    let cb: Arc<RwLock<Box<dyn FnMut() + Send + Sync>>> = Arc::new(RwLock::new(Box::new(callback)));

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

/// Attach a client JS body to an effect that already ran during SSR.
pub fn attach_client_effect(
    effect: &Effect,
    kind: &str,
    body: impl Into<String>,
    captures: BTreeMap<String, SignalId>,
    target: Option<SignalId>,
    debounce_ms: Option<u64>,
) {
    if let Some(ctx) = current_context() {
        let deps = ctx.take_effect_deps(effect.id.0);
        ctx.register_client_effect(ClientEffectSpec {
            id: effect.id.0,
            deps,
            captures,
            kind: kind.to_string(),
            body: body.into(),
            target,
            debounce_ms,
        });
    }
}

/// Register a client-replayable side effect with a JS body (from `debounce!` or manual use).
pub fn register_client_effect(
    kind: &str,
    body: impl Into<String>,
    captures: BTreeMap<String, SignalId>,
    target: Option<SignalId>,
    debounce_ms: Option<u64>,
) -> EffectId {
    let id = current_context()
        .map(|c| EffectId(c.next_effect_id()))
        .unwrap_or(EffectId(0));

    if let Some(ctx) = current_context() {
        let deps = ctx.take_effect_deps(id.0);
        ctx.register_client_effect(ClientEffectSpec {
            id: id.0,
            deps,
            captures,
            kind: kind.to_string(),
            body: body.into(),
            target,
            debounce_ms,
        });
    }
    id
}

/// Reactive derived value.
pub struct Computed<T: Clone + Serialize + Send + Sync + 'static> {
    signal: Signal<T>,
    #[allow(dead_code)]
    effect: Effect,
}

impl<T: Clone + Serialize + Send + Sync + 'static> Computed<T> {
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

/// Like [`use_computed`] but also registers a rs2js-translated body for client replay.
pub fn use_computed_with_js<T, F>(
    captures: BTreeMap<String, SignalId>,
    mut compute: F,
    js_body: &str,
) -> Computed<T>
where
    T: Clone + Serialize + Send + Sync + 'static,
    F: FnMut() -> T + Send + Sync + 'static,
{
    let initial = compute();
    let signal = Signal::new(initial);
    let target = signal.id();

    let signal_for_effect = signal.clone();
    let effect_id = current_context()
        .map(|c| EffectId(c.next_effect_id()))
        .unwrap_or(EffectId(0));

    let cb: Arc<RwLock<Box<dyn FnMut() + Send + Sync>>> =
        Arc::new(RwLock::new(Box::new(move || {
            let next = compute();
            signal_for_effect.set(next);
        })));

    if let Some(ctx) = current_context() {
        let cb_clone = cb.clone();
        ctx.register_effect(effect_id, move || {
            (cb_clone.write())();
        });
    }

    let eff = Effect {
        id: effect_id,
        callback: cb,
    };
    eff.run();

    let body = format!(
        "(state, __resuma) => {{ state.{target}.set(({js_body})(state, __resuma)); }}",
        target = target,
        js_body = js_body
    );

    if let Some(ctx) = current_context() {
        let deps = ctx.take_effect_deps(effect_id.0);
        ctx.register_client_effect(ClientEffectSpec {
            id: effect_id.0,
            deps,
            captures,
            kind: "computed".into(),
            body,
            target: Some(target),
            debounce_ms: None,
        });
    }

    Computed {
        signal,
        effect: eff,
    }
}
