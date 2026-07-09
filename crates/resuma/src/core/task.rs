//! Tasks & lifecycle hooks (`use_task`, `use_visible_task`).

use std::collections::BTreeMap;

use super::context::current_context;
use super::effect::{register_client_effect, use_effect, Effect};
use super::signal::{Signal, SignalId};

/// Side effect that runs during SSR and re-runs when tracked signals change.
pub fn use_task<F>(callback: F) -> Effect
where
    F: FnMut() + Send + Sync + 'static,
{
    use_effect(callback)
}

/// Client-only task registered in the resumability payload. The runtime
/// executes the JS body after the component becomes visible.
///
/// Prefer [`visible_task!`](crate::visible_task) when the body references
/// `state.todos`-style names — it wires signal captures automatically.
pub fn use_visible_task(source: impl Into<String>) -> VisibleTaskId {
    use_visible_task_with_captures(source, BTreeMap::new())
}

/// Like [`use_visible_task`] but maps Rust signal names to ids in the payload.
pub fn use_visible_task_with_captures(
    source: impl Into<String>,
    captures: BTreeMap<String, SignalId>,
) -> VisibleTaskId {
    let source = source.into();
    let id = current_context()
        .map(|c| c.register_visible_task(&source, captures))
        .unwrap_or(0);
    VisibleTaskId(id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleTaskId(pub u32);

/// Build a JS snippet for a visible task from Rust code via `js!` or rs2js.
pub fn visible_task_js(body: &str) -> String {
    format!("(async (state, __resuma) => {{ {} }})", body)
}

/// Debounced signal reaction for **SSR dependency tracking only**.
///
/// The `ms` delay is intentionally ignored here: SSR runs the callback once so
/// derived state / deps are captured. Honour the delay on the client with
/// [`debounce!`] / [`register_debounce_effect`], which emit a timers-based
/// replay body. Calling this without a matching client registration means the
/// reaction will re-fire immediately on every dependency change during SSR
/// cascading — never with a wall-clock delay.
pub fn use_debounce<T, F>(signal: &Signal<T>, ms: u64, mut on_change: F)
where
    T: Clone + serde::Serialize + Send + Sync + 'static,
    F: FnMut(T) + Send + Sync + 'static,
{
    let _ = ms; // client-only; see module docs
    let signal = signal.clone();
    use_effect(move || {
        on_change(signal.get());
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
    let mut watch_ids: Vec<super::signal::SignalId> = captures.values().copied().collect();
    if !watch_ids.contains(&signal_id) {
        watch_ids.push(signal_id);
    }
    let subscribe_lines: String = watch_ids
        .iter()
        .map(|id| {
            format!(
                "if (state.{id}) cleanups.push(state.{id}.subscribe(schedule));",
                id = id
            )
        })
        .collect::<Vec<_>>()
        .join("\n        ");
    let body = format!(
        "(state, __resuma) => {{
            const key = '__deb_{n}';
            const run = {js_body};
            const cleanups = [];
            const schedule = () => {{
                clearTimeout(state[key]);
                state[key] = setTimeout(() => run(state, __resuma), {ms});
            }};
            {subscribe_lines}
            schedule();
            return () => {{
                clearTimeout(state[key]);
                for (const unsub of cleanups) unsub?.();
            }};
        }}",
        n = signal_id.0,
        js_body = js_body,
        ms = ms,
        subscribe_lines = subscribe_lines,
    );
    register_client_effect("debounce", body, captures, None, Some(ms));
}
