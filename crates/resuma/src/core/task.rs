//! Tasks & lifecycle hooks (`use_task`, `use_visible_task`).

use std::collections::BTreeMap;

use super::context::current_context;
use super::effect::{register_client_effect, use_effect, Effect};
use super::signal::Signal;

/// Side effect that runs during SSR and re-runs when tracked signals change.
pub fn use_task<F>(callback: F) -> Effect
where
    F: FnMut() + Send + Sync + 'static,
{
    use_effect(callback)
}

/// Client-only task registered in the resumability payload. The runtime
/// executes the JS body after the component becomes visible.
pub fn use_visible_task(source: impl Into<String>) -> VisibleTaskId {
    let source = source.into();
    let id = current_context()
        .map(|c| c.register_visible_task(&source))
        .unwrap_or(0);
    VisibleTaskId(id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleTaskId(pub u32);

/// Build a JS snippet for a visible task from Rust code via `js!` or rs2js.
pub fn visible_task_js(body: &str) -> String {
    format!("(async (state, __resuma) => {{ {} }})", body)
}

/// Debounced signal reaction — SSR runs immediately; use `debounce!` for client replay.
pub fn use_debounce<T, F>(signal: &Signal<T>, ms: u64, mut on_change: F)
where
    T: Clone + serde::Serialize + Send + Sync + 'static,
    F: FnMut(T) + Send + Sync + 'static,
{
    let signal = signal.clone();
    use_effect(move || {
        on_change(signal.peek());
        let _ = ms;
    });
}

/// Register a debounced client effect with an rs2js-translated callback body.
pub fn register_debounce_effect<T>(
    signal: &Signal<T>,
    ms: u64,
    captures: BTreeMap<String, super::signal::SignalId>,
    js_body: &str,
) where
    T: Clone + serde::Serialize + Send + Sync + 'static,
{
    let signal_id = signal.id();
    let body = format!(
        "(state, __resuma) => {{ const src = state.{sid}; const key = '__deb_{n}'; const run = {js_body}; src.subscribe(() => {{ clearTimeout(state[key]); state[key] = setTimeout(() => run(state, __resuma), {ms}); }}); }}",
        sid = signal_id,
        n = signal_id.0,
        js_body = js_body,
        ms = ms
    );
    register_client_effect("debounce", body, captures, None, Some(ms));
}
