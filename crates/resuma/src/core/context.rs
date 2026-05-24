//! Per-render context.
//!
//! The `RenderContext` keeps track of every reactive primitive allocated
//! during a SSR pass. After rendering, the context's serialized state is
//! embedded into the HTML payload so the client runtime can pick up where
//! the server left off — the very definition of resumability.

use std::any::TypeId;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use serde::Serialize;
use serde_json::Value;

use super::effect::EffectId;
use super::signal::SignalId;

/// Max handler JS source bytes kept inline in the HTML payload (`__page__` only).
pub const INLINE_HANDLER_MAX_BYTES: usize = 256;

thread_local! {
    static CURRENT: RefCell<Option<Rc<RenderContext>>> = const { RefCell::new(None) };
}

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
    effects: RefCell<BTreeMap<u32, Box<dyn FnMut()>>>,
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
    /// Client-only visible tasks (id → JS source).
    visible_tasks: RefCell<BTreeMap<u32, String>>,
    next_visible_task: AtomicU32,
    /// Effect id → signal dependencies collected during SSR.
    effect_deps: RefCell<BTreeMap<u32, Vec<SignalId>>>,
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
            current_effect: RefCell::new(None),
            handler_chunks: RefCell::new(BTreeMap::new()),
            islands: RefCell::new(Vec::new()),
            actions: RefCell::new(Vec::new()),
            contexts: RefCell::new(BTreeMap::new()),
            visible_tasks: RefCell::new(BTreeMap::new()),
            next_visible_task: AtomicU32::new(1),
            effect_deps: RefCell::new(BTreeMap::new()),
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
        self.effects.borrow_mut().insert(id.0, Box::new(f));
    }

    pub fn run_effect(&self, id: u32) {
        if let Some(eff) = self.effects.borrow_mut().get_mut(&id) {
            eff();
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

    pub fn register_context(&self, id: TypeId, value: Value) {
        let key = format!("{:?}", id);
        self.contexts.borrow_mut().insert(key, value);
    }

    pub fn get_context(&self, id: TypeId) -> Option<Value> {
        let key = format!("{:?}", id);
        self.contexts.borrow().get(&key).cloned()
    }

    pub fn register_visible_task(&self, source: &str) -> u32 {
        let id = self.next_visible_task.fetch_add(1, Ordering::Relaxed);
        self.visible_tasks
            .borrow_mut()
            .insert(id, source.to_string());
        id
    }

    pub fn record_effect_dep(&self, effect_id: u32, signal_id: SignalId) {
        let mut deps = self.effect_deps.borrow_mut();
        let list = deps.entry(effect_id).or_default();
        if !list.contains(&signal_id) {
            list.push(signal_id);
        }
    }

    pub fn take_effect_deps(&self, effect_id: u32) -> Vec<SignalId> {
        self.effect_deps
            .borrow_mut()
            .remove(&effect_id)
            .unwrap_or_default()
    }

    pub fn register_client_effect(&self, spec: ClientEffectSpec) {
        self.client_effects.borrow_mut().push(spec);
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
        }
    }
}

impl ResumePayload {
    /// Strip external handler JS from the client payload (lazy-fetched by chunk id).
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

/// The JSON blob that travels in `<script type="resuma/state">…</script>`.
#[derive(Debug, Clone, Serialize)]
pub struct ResumePayload {
    pub signals: Vec<SignalSnapshot>,
    pub handlers: BTreeMap<String, BTreeMap<String, String>>,
    pub islands: Vec<String>,
    pub actions: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub contexts: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub visible_tasks: BTreeMap<u32, String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub effects: Vec<ClientEffectSpec>,
    /// Handler chunk ids fetched lazily from `/_resuma/handler/:chunk.js`.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub lazy_chunks: Vec<String>,
    /// Double-submit CSRF token (sent as `X-Resuma-CSRF` on POST requests).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csrf_token: Option<String>,
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
        "data-r-bind:",
        "data-r-portal",
        "data-r-stream",
        "data-r-vt",
    ];
    MARKERS.iter().any(|marker| body_html.contains(marker))
}

pub fn with_context<R>(ctx: Rc<RenderContext>, f: impl FnOnce() -> R) -> R {
    CURRENT.with(|cell| {
        let prev = cell.borrow_mut().replace(ctx);
        let result = f();
        *cell.borrow_mut() = prev;
        result
    })
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
    CURRENT.with(|cell| cell.borrow().clone())
}
