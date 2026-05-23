//! Tasks & lifecycle hooks (`use_task`, `use_visible_task`).

use crate::context::current_context;
use crate::effect::{use_effect, Effect};

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

/// Debounced signal updates — cookbook pattern.
pub fn use_debounce<T, F>(signal: &crate::signal::Signal<T>, ms: u64, mut on_change: F)
where
    T: Clone + serde::Serialize + Send + Sync + 'static,
    F: FnMut(T) + Send + Sync + 'static,
{
    let signal = signal.clone();
    use_effect(move || {
        let value = signal.peek();
        // SSR: run once. Client debounce is handled by registering a visible
        // task when needed; for now invoke immediately on server.
        on_change(value);
        let _ = ms; // client-side debounce wired in runtime v0.3
    });
}
