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

use crate::effect::EffectId;
use crate::signal::SignalId;

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
        })
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
        self.visible_tasks.borrow_mut().insert(id, source.to_string());
        id
    }

    /// Snapshot the entire reactive state as JSON ready to embed in HTML.
    pub fn snapshot(&self) -> ResumePayload {
        ResumePayload {
            signals: self
                .state
                .borrow()
                .iter()
                .map(|(id, v)| SignalSnapshot { id: *id, value: v.clone() })
                .collect(),
            handlers: self.handler_chunks.borrow().clone(),
            islands: self.islands.borrow().clone(),
            actions: self.actions.borrow().clone(),
            contexts: self.contexts.borrow().clone(),
            visible_tasks: self.visible_tasks.borrow().clone(),
        }
    }
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
}

impl ResumePayload {
    /// True when the serialized payload carries resumable client state.
    pub fn needs_client(&self) -> bool {
        !self.signals.is_empty()
            || !self.handlers.is_empty()
            || !self.islands.is_empty()
            || !self.actions.is_empty()
            || !self.visible_tasks.is_empty()
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

pub fn current_context() -> Option<Rc<RenderContext>> {
    CURRENT.with(|cell| cell.borrow().clone())
}
