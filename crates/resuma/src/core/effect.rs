//! Effects and computed values.
//!
//! | API | SSR | Client replay |
//! |-----|-----|---------------|
//! | [`use_effect`] / [`use_computed`] | yes | no |
//! | [`computed!`](crate::computed) / [`effect!`](crate::effect) / [`debounce!`](crate::debounce) | yes | yes (rs2js) |
//!
//! SSR always runs effects once to capture derived state. When a client JS body is registered
//! (via [`use_computed_with_js`] or the macros), the runtime replays them when dependencies change.

use std::collections::BTreeMap;
use std::sync::Arc;

use parking_lot::RwLock;
use serde::Serialize;

use super::context::{current_context, ClientEffectSpec};
use super::signal::{Signal, SignalId};

fn merge_capture_deps(deps: &mut Vec<SignalId>, captures: &BTreeMap<String, SignalId>) {
    for id in captures.values() {
        if !deps.contains(id) {
            deps.push(*id);
        }
    }
}

/// Opaque effect id. Stable within a single render pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectId(pub u32);

/// Unique fallback id when no [`RenderContext`] is active (tests / direct calls).
/// Starts far above typical SSR ids so it never collides with in-context effects.
fn fallback_effect_id() -> EffectId {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(2_000_000);
    EffectId(COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// A user-supplied side effect bound to a closure.
#[derive(Clone)]
pub struct Effect {
    pub id: EffectId,
    callback: Arc<RwLock<Box<dyn FnMut() + Send + Sync>>>,
}

impl Effect {
    pub fn run(&self) {
        // Route the initial run through the context's `run_effect` so it shares
        // the same re-entrancy guard as change-driven runs. Otherwise a cyclic
        // effect that re-enters *itself* during its first run would try to
        // re-acquire its own callback `RwLock` (non-reentrant) and deadlock,
        // because `running_effects` would not yet contain its id.
        if let Some(ctx) = current_context() {
            ctx.run_effect(self.id.0);
        } else {
            (self.callback.write())();
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
        .unwrap_or_else(fallback_effect_id);

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
        let mut deps = ctx.take_effect_deps(effect.id.0);
        merge_capture_deps(&mut deps, &captures);
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
        .unwrap_or_else(fallback_effect_id);

    if let Some(ctx) = current_context() {
        let mut deps = ctx.take_effect_deps(id.0);
        merge_capture_deps(&mut deps, &captures);
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
#[derive(Clone)]
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
    // Lazily create the signal inside the effect's first run so `compute()` is
    // called exactly once while still recording signal dependencies.
    let signal_slot: Arc<RwLock<Option<Signal<T>>>> = Arc::new(RwLock::new(None));
    let slot_for_effect = signal_slot.clone();

    let id = current_context()
        .map(|c| EffectId(c.next_effect_id()))
        .unwrap_or_else(fallback_effect_id);

    let cb: Arc<RwLock<Box<dyn FnMut() + Send + Sync>>> =
        Arc::new(RwLock::new(Box::new(move || {
            let next = compute();
            let mut guard = slot_for_effect.write();
            if let Some(sig) = guard.as_ref() {
                sig.set(next);
            } else {
                *guard = Some(Signal::new(next));
            }
        })));

    if let Some(ctx) = current_context() {
        let cb_clone = cb.clone();
        ctx.register_effect(id, move || {
            (cb_clone.write())();
        });
    }

    let eff = Effect {
        id,
        callback: cb,
    };
    eff.run();

    let signal = signal_slot
        .read()
        .clone()
        .expect("use_computed: effect must initialize signal");

    Computed { signal, effect: eff }
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
    let signal_slot: Arc<RwLock<Option<Signal<T>>>> = Arc::new(RwLock::new(None));
    let slot_for_effect = signal_slot.clone();

    let effect_id = current_context()
        .map(|c| EffectId(c.next_effect_id()))
        .unwrap_or_else(fallback_effect_id);

    let cb: Arc<RwLock<Box<dyn FnMut() + Send + Sync>>> =
        Arc::new(RwLock::new(Box::new(move || {
            let next = compute();
            let mut guard = slot_for_effect.write();
            if let Some(sig) = guard.as_ref() {
                sig.set(next);
            } else {
                *guard = Some(Signal::new(next));
            }
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

    let signal = signal_slot
        .read()
        .clone()
        .expect("use_computed_with_js: effect must initialize signal");
    let target = signal.id();

    let body = format!(
        "(state, __resuma) => {{ state.{target}.set(({js_body})(state, __resuma)); }}",
        target = target,
        js_body = js_body
    );

    if let Some(ctx) = current_context() {
        let mut deps = ctx.take_effect_deps(effect_id.0);
        merge_capture_deps(&mut deps, &captures);
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
