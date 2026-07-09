//! Per-render context.
//!
//! The `RenderContext` keeps track of every reactive primitive allocated
//! during a SSR pass. After rendering, the context's serialized state is
//! embedded into the HTML payload so the client runtime can pick up where
//! the server left off — the very definition of resumability.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use serde::Serialize;
use serde_json::Value;

use super::effect::EffectId;
use super::signal::SignalId;

/// Max handler JS source bytes kept inline in the HTML payload (`__page__` only).
pub const INLINE_HANDLER_MAX_BYTES: usize = 256;

tokio::task_local! {
    /// Active render-context handles for this async task (innermost last).
    static RENDER_HANDLES: RefCell<Vec<usize>>;
}

thread_local! {
    static RENDER_CONTEXTS: RefCell<BTreeMap<usize, Rc<RenderContext>>> =
        const { RefCell::new(BTreeMap::new()) };
    static FALLBACK_RENDER_HANDLES: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

static NEXT_RENDER_HANDLE: AtomicU32 = AtomicU32::new(1);

/// What we are rendering for. Mostly used to tweak hydration markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// Full server side render including resumability payload.
    Ssr,
    /// Render an island in isolation — used by the dev server to update one
    /// island after a hot reload.
    Island,
    /// Static export, no resumability needed.
    Static,
}

/// Snapshot of a single signal as captured by the SSR pass.
#[derive(Debug, Clone, Serialize)]
pub struct SignalSnapshot {
    pub id: SignalId,
    pub value: Value,
}

/// Per-render reactive bookkeeping.
pub struct RenderContext {
    pub mode: RenderMode,
    next_signal: AtomicU32,
    next_effect: AtomicU32,
    state: RefCell<BTreeMap<SignalId, Value>>,
    #[allow(clippy::type_complexity)]
    effects: RefCell<BTreeMap<u32, Rc<RefCell<Box<dyn FnMut()>>>>>,
    /// Effect ids currently executing — guards against re-entrant cascades and
    /// dependency cycles that would otherwise deadlock or panic.
    running_effects: RefCell<BTreeSet<u32>>,
    current_effect: RefCell<Option<u32>>,
    /// Handler chunks referenced by this page. Maps chunk id → symbol → JS
    /// source. Populated by the macro layer.
    handler_chunks: RefCell<BTreeMap<String, BTreeMap<String, String>>>,
    /// Islands instantiated in this page.
    islands: RefCell<Vec<String>>,
    /// Server actions referenced in this page.
    actions: RefCell<Vec<String>>,
    /// Serializable component contexts (type key → JSON value).
    contexts: RefCell<BTreeMap<String, Value>>,
    /// Client-only visible tasks (id → spec).
    visible_tasks: RefCell<BTreeMap<u32, VisibleTaskSpec>>,
    next_visible_task: AtomicU32,
    /// Effect id → signal dependencies collected during SSR.
    effect_deps: RefCell<BTreeMap<u32, Vec<SignalId>>>,
    /// Signal id → effect ids subscribed during the current dependency pass.
    signal_subscribers: RefCell<BTreeMap<SignalId, Vec<u32>>>,
    /// Client-replayable effects (computed, debounce, side effects with JS).
    client_effects: RefCell<Vec<ClientEffectSpec>>,
    /// Active component/island boundary stack for handler chunk ids.
    handler_chunk_stack: RefCell<Vec<String>>,
}

impl RenderContext {
    pub fn new(mode: RenderMode) -> Rc<Self> {
        Rc::new(Self {
            mode,
            next_signal: AtomicU32::new(1),
            next_effect: AtomicU32::new(1),
            state: RefCell::new(BTreeMap::new()),
            effects: RefCell::new(BTreeMap::new()),
            running_effects: RefCell::new(BTreeSet::new()),
            current_effect: RefCell::new(None),
            handler_chunks: RefCell::new(BTreeMap::new()),
            islands: RefCell::new(Vec::new()),
            actions: RefCell::new(Vec::new()),
            contexts: RefCell::new(BTreeMap::new()),
            visible_tasks: RefCell::new(BTreeMap::new()),
            next_visible_task: AtomicU32::new(1),
            effect_deps: RefCell::new(BTreeMap::new()),
            signal_subscribers: RefCell::new(BTreeMap::new()),
            client_effects: RefCell::new(Vec::new()),
            handler_chunk_stack: RefCell::new(Vec::new()),
        })
    }

    /// Innermost handler chunk id (`__page__` when no component boundary is active).
    pub fn current_handler_chunk(&self) -> String {
        self.handler_chunk_stack
            .borrow()
            .last()
            .cloned()
            .unwrap_or_else(|| "__page__".to_string())
    }

    pub fn push_handler_chunk(&self, chunk: impl Into<String>) {
        self.handler_chunk_stack.borrow_mut().push(chunk.into());
    }

    pub fn pop_handler_chunk(&self) {
        self.handler_chunk_stack.borrow_mut().pop();
    }

    pub fn next_signal_id(&self) -> SignalId {
        SignalId(self.next_signal.fetch_add(1, Ordering::Relaxed))
    }

    pub fn next_effect_id(&self) -> u32 {
        self.next_effect.fetch_add(1, Ordering::Relaxed)
    }

    pub fn current_effect_id(&self) -> Option<u32> {
        *self.current_effect.borrow()
    }

    pub fn set_current_effect(&self, id: Option<EffectId>) {
        *self.current_effect.borrow_mut() = id.map(|e| e.0);
    }

    pub fn register_signal(&self, id: SignalId, value: Value) {
        self.state.borrow_mut().insert(id, value);
    }

    pub fn update_signal(&self, id: SignalId, value: Value) {
        self.state.borrow_mut().insert(id, value);
    }

    pub fn register_effect<F: FnMut() + 'static>(&self, id: EffectId, f: F) {
        self.effects
            .borrow_mut()
            .insert(id.0, Rc::new(RefCell::new(Box::new(f))));
    }

    /// Run a registered effect by id.
    ///
    /// The callback is cloned out (via `Rc`) before invocation so the `effects`
    /// map is not borrowed while the effect runs — this is what allows one
    /// effect to trigger another (cascading `computed`/`effect` chains) without
    /// hitting a `RefCell already borrowed` panic. A `running_effects` guard
    /// short-circuits re-entrant cycles (A → B → A) instead of deadlocking.
    pub fn run_effect(&self, id: u32) {
        // The effect currently tracking dependencies (initial run) must not be
        // re-entered, and neither must an effect already on the run stack.
        if *self.current_effect.borrow() == Some(id) {
            tracing::warn!(
                effect_id = id,
                "effect re-entered while tracking dependencies — skipped (possible dependency cycle)"
            );
            return;
        }
        if self.running_effects.borrow().contains(&id) {
            let msg = format!(
                "effect cycle detected — effect {id} re-entered while running (derived state may be stale)"
            );
            if effect_cycle_panic_enabled() {
                panic!("{msg}");
            }
            tracing::error!(effect_id = id, "{msg}");
            return;
        }
        let cb = self.effects.borrow().get(&id).cloned();
        if let Some(cb) = cb {
            self.clear_effect_deps(id);
            self.running_effects.borrow_mut().insert(id);
            // Save/restore the previously-tracking effect so nested effect runs
            // don't leave `current_effect` as `None` for the remainder of the
            // parent's execution (which would drop dependency tracking).
            let prev = *self.current_effect.borrow();
            self.set_current_effect(Some(EffectId(id)));
            (cb.borrow_mut())();
            *self.current_effect.borrow_mut() = prev;
            self.running_effects.borrow_mut().remove(&id);
            self.sync_client_effect_deps(id);
        }
    }

    /// Keep client-replay effect deps aligned with the latest SSR dependency pass.
    pub fn sync_client_effect_deps(&self, effect_id: u32) {
        let deps = self
            .effect_deps
            .borrow()
            .get(&effect_id)
            .cloned()
            .unwrap_or_default();
        let mut effects = self.client_effects.borrow_mut();
        if let Some(spec) = effects.iter_mut().find(|e| e.id == effect_id) {
            spec.deps = deps;
        }
    }

    pub fn register_handler(&self, chunk: &str, symbol: &str, source: &str) {
        self.handler_chunks
            .borrow_mut()
            .entry(chunk.to_string())
            .or_default()
            .insert(symbol.to_string(), source.to_string());
    }

    pub fn register_island(&self, chunk_id: &str) {
        self.islands.borrow_mut().push(chunk_id.to_string());
    }

    pub fn register_action(&self, name: &str) {
        self.actions.borrow_mut().push(name.to_string());
    }

    pub fn register_context(&self, key: &str, value: Value) {
        self.contexts.borrow_mut().insert(key.to_string(), value);
    }

    pub fn get_context(&self, key: &str) -> Option<Value> {
        self.contexts.borrow().get(key).cloned()
    }

    pub fn register_visible_task(
        &self,
        source: &str,
        captures: BTreeMap<String, super::signal::SignalId>,
    ) -> u32 {
        let id = self.next_visible_task.fetch_add(1, Ordering::Relaxed);
        self.visible_tasks.borrow_mut().insert(
            id,
            VisibleTaskSpec {
                body: source.to_string(),
                captures,
            },
        );
        id
    }

    pub fn record_effect_dep(&self, effect_id: u32, signal_id: SignalId) {
        {
            let mut deps = self.effect_deps.borrow_mut();
            let list = deps.entry(effect_id).or_default();
            if !list.contains(&signal_id) {
                list.push(signal_id);
            }
        }
        let mut subs = self.signal_subscribers.borrow_mut();
        let list = subs.entry(signal_id).or_default();
        if !list.contains(&effect_id) {
            list.push(effect_id);
        }
    }

    /// Drop tracked deps for an effect before re-running it (conditional branches).
    pub fn clear_effect_deps(&self, effect_id: u32) {
        let old = self
            .effect_deps
            .borrow_mut()
            .remove(&effect_id)
            .unwrap_or_default();
        let mut subs = self.signal_subscribers.borrow_mut();
        for sig_id in old {
            if let Some(list) = subs.get_mut(&sig_id) {
                list.retain(|&e| e != effect_id);
            }
        }
    }

    pub fn signal_subscriber_ids(&self, signal_id: SignalId) -> Vec<u32> {
        self.signal_subscribers
            .borrow()
            .get(&signal_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn take_effect_deps(&self, effect_id: u32) -> Vec<SignalId> {
        self.effect_deps
            .borrow_mut()
            .remove(&effect_id)
            .unwrap_or_default()
    }

    pub fn register_client_effect(&self, spec: ClientEffectSpec) {
        let mut effects = self.client_effects.borrow_mut();
        if let Some(existing) = effects.iter_mut().find(|e| e.id == spec.id) {
            for dep in spec.deps {
                if !existing.deps.contains(&dep) {
                    existing.deps.push(dep);
                }
            }
            for (k, v) in spec.captures {
                existing.captures.entry(k).or_insert(v);
            }
            if !spec.body.is_empty() {
                existing.body = spec.body;
            }
            if !spec.kind.is_empty() {
                existing.kind = spec.kind;
            }
            if spec.target.is_some() {
                existing.target = spec.target;
            }
            if spec.debounce_ms.is_some() {
                existing.debounce_ms = spec.debounce_ms;
            }
        } else {
            effects.push(spec);
        }
    }

    /// Snapshot for embedding in HTML (strips external handler sources).
    pub fn snapshot(&self) -> ResumePayload {
        self.snapshot_internal().for_client()
    }

    /// Full snapshot including all handler sources (server-side chunk registration).
    pub fn snapshot_full(&self) -> ResumePayload {
        self.snapshot_internal()
    }

    fn snapshot_internal(&self) -> ResumePayload {
        ResumePayload {
            signals: self
                .state
                .borrow()
                .iter()
                .map(|(id, v)| SignalSnapshot {
                    id: *id,
                    value: v.clone(),
                })
                .collect(),
            handlers: self.handler_chunks.borrow().clone(),
            islands: self.islands.borrow().clone(),
            actions: self.actions.borrow().clone(),
            contexts: self.contexts.borrow().clone(),
            visible_tasks: self.visible_tasks.borrow().clone(),
            effects: self.client_effects.borrow().clone(),
            lazy_chunks: self
                .handler_chunks
                .borrow()
                .keys()
                .filter(|k| *k != "__page__")
                .cloned()
                .collect(),
            csrf_token: None,
            serialization_error: None,
        }
    }
}

impl ResumePayload {
    /// Strip external handler JS from the payload sent to the browser.
    ///
    /// Keeps only `__page__` handlers under [`INLINE_HANDLER_MAX_BYTES`].
    /// All other chunk sources are served from `/_resuma/handler/:chunk.js`.
    pub fn for_client(&self) -> Self {
        let mut client = self.clone();
        let mut inline_page = BTreeMap::new();
        let mut lazy = self.lazy_chunks.clone();

        if let Some(page) = self.handlers.get("__page__") {
            for (sym, src) in page {
                if src.len() <= INLINE_HANDLER_MAX_BYTES {
                    inline_page.insert(sym.clone(), src.clone());
                } else {
                    lazy.push("__page__".to_string());
                }
            }
        }

        client.handlers = BTreeMap::new();
        if !inline_page.is_empty() {
            client.handlers.insert("__page__".into(), inline_page);
        }

        lazy.sort();
        lazy.dedup();
        client.lazy_chunks = lazy;
        client
    }
}

/// Client-side effect registered during SSR (replayed by the runtime).
#[derive(Debug, Clone, Serialize)]
pub struct ClientEffectSpec {
    pub id: u32,
    pub deps: Vec<SignalId>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub captures: BTreeMap<String, SignalId>,
    pub kind: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<SignalId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debounce_ms: Option<u64>,
}

/// Client-only task registered during SSR (`use_visible_task` / `visible_task!`).
#[derive(Debug, Clone, Serialize)]
pub struct VisibleTaskSpec {
    pub body: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub captures: BTreeMap<String, super::signal::SignalId>,
}

/// The JSON blob embedded in `<script type="resuma/state">…</script>`.
///
/// Built by [`RenderContext::snapshot`] during SSR. The client-facing version
/// ([`for_client`](Self::for_client)) strips external handler sources; chunk ids
/// appear in [`lazy_chunks`](Self::lazy_chunks) and load from `/_resuma/handler/:chunk.js`.
///
/// # Fields
///
/// * `signals` — serialized [`SignalSnapshot`] values
/// * `handlers` — inline handler JS (typically small `__page__` handlers only)
/// * `lazy_chunks` — component/island chunk ids prefetched or fetched on demand
/// * `effects` — client-replay specs from `computed!` / `effect!` / `debounce!`
/// * `islands` — optional `#[island]` instances on the page
/// * `actions` — `#[server]` action names referenced by handlers
#[derive(Debug, Clone, Serialize)]
pub struct ResumePayload {
    pub signals: Vec<SignalSnapshot>,
    pub handlers: BTreeMap<String, BTreeMap<String, String>>,
    pub islands: Vec<String>,
    pub actions: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub contexts: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub visible_tasks: BTreeMap<u32, VisibleTaskSpec>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub effects: Vec<ClientEffectSpec>,
    /// Handler chunk ids fetched lazily from `/_resuma/handler/:chunk.js`.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub lazy_chunks: Vec<String>,
    /// Double-submit CSRF token (sent as `X-Resuma-CSRF` on POST requests).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csrf_token: Option<String>,
    /// Set when the resumability payload failed to serialize (page interactivity broken).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serialization_error: Option<bool>,
}

impl ResumePayload {
    /// True when the serialized payload carries resumable client state.
    pub fn needs_client(&self) -> bool {
        !self.signals.is_empty()
            || !self.handlers.is_empty()
            || !self.islands.is_empty()
            || !self.actions.is_empty()
            || !self.visible_tasks.is_empty()
            || !self.effects.is_empty()
            || !self.lazy_chunks.is_empty()
    }
}

/// Whether a rendered page should ship the resumability payload + loader.
pub fn page_needs_client(payload: &ResumePayload, body_html: &str) -> bool {
    if payload.needs_client() {
        return true;
    }
    const MARKERS: &[&str] = &[
        "data-r-on:",
        "data-r-submit",
        "resuma-island",
        "resuma-boundary",
        "resuma-dyn",
        "resuma-show",
        "resuma-for",
        "resuma-match",
        "data-r-bind:",
        "data-r-nav",
        "data-r-portal",
        "data-r-stream",
        "data-r-vt",
    ];
    MARKERS.iter().any(|marker| body_html.contains(marker))
}

/// Run `fut` with a fresh, task-isolated render-context handle stack (one scope per HTTP request).
pub async fn scope_render_context<F: Future>(fut: F) -> F::Output {
    RENDER_HANDLES
        .scope(RefCell::new(Vec::new()), fut)
        .await
}

fn with_render_handles<R>(f: impl FnOnce(&RefCell<Vec<usize>>) -> R) -> R {
    let mut f = Some(f);
    match RENDER_HANDLES.try_with(|cell| (f.take().expect("render handles fn"))(cell)) {
        Ok(out) => out,
        Err(_) => FALLBACK_RENDER_HANDLES.with(|cell| (f.take().expect("render handles fn"))(cell)),
    }
}

fn insert_render_context(ctx: Rc<RenderContext>) -> usize {
    let handle = NEXT_RENDER_HANDLE.fetch_add(1, Ordering::Relaxed) as usize;
    RENDER_CONTEXTS.with(|map| {
        map.borrow_mut().insert(handle, ctx);
    });
    handle
}

fn remove_render_context(handle: usize) {
    RENDER_CONTEXTS.with(|map| {
        map.borrow_mut().remove(&handle);
    });
}

struct ContextRestore {
    handle: usize,
}

impl Drop for ContextRestore {
    fn drop(&mut self) {
        with_render_handles(|stack| {
            if stack.borrow().last() == Some(&self.handle) {
                stack.borrow_mut().pop();
            }
        });
        remove_render_context(self.handle);
    }
}

pub fn with_context<R>(ctx: Rc<RenderContext>, f: impl FnOnce() -> R) -> R {
    let handle = insert_render_context(ctx);
    with_render_handles(|stack| stack.borrow_mut().push(handle));
    let _guard = ContextRestore { handle };
    f()
}

/// Run `f` while handlers register under `chunk` (component / island boundary).
pub fn with_handler_chunk<R>(chunk: impl Into<String>, f: impl FnOnce() -> R) -> R {
    if let Some(ctx) = current_context() {
        ctx.push_handler_chunk(chunk);
        let out = f();
        ctx.pop_handler_chunk();
        out
    } else {
        f()
    }
}

pub fn current_context() -> Option<Rc<RenderContext>> {
    with_render_handles(|stack| {
        let handle = stack.borrow().last().copied()?;
        RENDER_CONTEXTS.with(|map| map.borrow().get(&handle).cloned())
    })
}

fn effect_cycle_panic_enabled() -> bool {
    matches!(
        std::env::var("RESUMA_DEV").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::signal;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn render_context_isolated_per_scoped_task() {
        let mut handles = Vec::new();
        for i in 0..32u32 {
            handles.push(tokio::spawn(async move {
                scope_render_context(async {
                    let ctx = RenderContext::new(RenderMode::Ssr);
                    let value = with_context(ctx.clone(), || {
                        let _s = signal(i);
                        tokio::task::block_in_place(|| {
                            std::thread::sleep(std::time::Duration::from_millis(1));
                        });
                        ctx.snapshot().signals[0].value.clone()
                    });
                    assert_eq!(value, serde_json::json!(i));
                })
                .await
            }));
        }
        for h in handles {
            h.await.unwrap();
        }
    }

    #[test]
    fn with_context_restores_after_panic() {
        let outer = RenderContext::new(RenderMode::Ssr);
        with_context(outer.clone(), || {
            let inner = RenderContext::new(RenderMode::Ssr);
            let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                with_context(inner, || {
                    panic!("simulated render failure");
                });
            }));
            assert!(caught.is_err());
            assert!(
                current_context().is_some(),
                "outer render context must be restored after inner panic"
            );
            assert!(
                Rc::ptr_eq(&current_context().unwrap(), &outer),
                "outer render context must be the active context"
            );
        });
        assert!(current_context().is_none());
    }

    #[test]
    fn page_needs_client_detects_resuma_for_marker() {
        let payload = ResumePayload {
            signals: vec![],
            handlers: Default::default(),
            islands: vec![],
            actions: vec![],
            contexts: Default::default(),
            visible_tasks: Default::default(),
            effects: vec![],
            lazy_chunks: vec![],
            csrf_token: None,
            serialization_error: None,
        };
        let body = r#"<resuma-for data-r-for="s1"><div data-r-for-list></div></resuma-for>"#;
        assert!(page_needs_client(&payload, body));
    }

    #[test]
    fn page_needs_client_detects_resuma_match_marker() {
        let payload = ResumePayload {
            signals: vec![],
            handlers: Default::default(),
            islands: vec![],
            actions: vec![],
            contexts: Default::default(),
            visible_tasks: Default::default(),
            effects: vec![],
            lazy_chunks: vec![],
            csrf_token: None,
            serialization_error: None,
        };
        let body = r#"<resuma-match data-r-match="s1"></resuma-match>"#;
        assert!(page_needs_client(&payload, body));
    }
}
