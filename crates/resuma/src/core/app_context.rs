//! Component context — `provide_context` / `use_context` for descendant trees.
//!
//! Context values are serializable and travel in the resumability payload so
//! descendant components can read them on the client after resume.
//!
//! Active frame handles live in **task-local** storage (see [`scope_context_stack`]).
//! Frame data uses `Rc` and is stored in a **thread-local** map keyed by handle so
//! concurrent async tasks on the same worker thread stay isolated during
//! `block_in_place` + `block_on` page renders.

use std::any::TypeId;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::future::Future;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use super::context::current_context;

tokio::task_local! {
    static CONTEXT_FRAMES: RefCell<Vec<usize>>;
}

thread_local! {
    static CONTEXT_FRAME_MAP: RefCell<BTreeMap<usize, Rc<BTreeMap<TypeId, Value>>>> =
        const { RefCell::new(BTreeMap::new()) };
    static FALLBACK_CONTEXT_FRAMES: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

static NEXT_CONTEXT_FRAME: AtomicU32 = AtomicU32::new(1);

/// Run `fut` with a fresh, task-isolated component context stack (one scope per HTTP request).
pub async fn scope_context_stack<F: Future>(fut: F) -> F::Output {
    CONTEXT_FRAMES.scope(RefCell::new(Vec::new()), fut).await
}

fn with_context_frames<R>(f: impl FnOnce(&RefCell<Vec<usize>>) -> R) -> R {
    let mut f = Some(f);
    match CONTEXT_FRAMES.try_with(|cell| (f.take().expect("context frames fn"))(cell)) {
        Ok(out) => out,
        Err(_) => FALLBACK_CONTEXT_FRAMES.with(|cell| (f.take().expect("context frames fn"))(cell)),
    }
}

fn alloc_context_frame() -> usize {
    let handle = NEXT_CONTEXT_FRAME.fetch_add(1, Ordering::Relaxed) as usize;
    CONTEXT_FRAME_MAP.with(|map| {
        map.borrow_mut().insert(handle, Rc::new(BTreeMap::new()));
    });
    handle
}

fn remove_context_frame(handle: usize) {
    CONTEXT_FRAME_MAP.with(|map| {
        map.borrow_mut().remove(&handle);
    });
}

fn top_context_frame_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut BTreeMap<TypeId, Value>) -> R,
{
    with_context_frames(|stack| {
        let handle = *stack
            .borrow()
            .last()
            .expect("context frame stack must not be empty");
        CONTEXT_FRAME_MAP.with(|map| {
            let mut borrow = map.borrow_mut();
            let frame = borrow
                .get_mut(&handle)
                .expect("context frame handle must exist");
            f(Rc::make_mut(frame))
        })
    })
}

/// Typed context handle. Create one per context type:
///
/// ```ignore
/// static THEME: ContextId<Theme> = ContextId::new();
/// ```
pub struct ContextId<T: 'static> {
    id: TypeId,
    _marker: std::marker::PhantomData<T>,
}

impl<T: 'static> Default for ContextId<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> ContextId<T> {
    pub const fn new() -> Self {
        Self {
            id: TypeId::of::<T>(),
            _marker: std::marker::PhantomData,
        }
    }
}

/// Provide a context value visible to this component and its descendants.
/// Returns `false` when serialization fails (value is not registered).
pub fn provide_context<T: Serialize + Clone + 'static>(ctx: &ContextId<T>, value: T) -> bool {
    let json = match serde_json::to_value(&value) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(
                type_name = std::any::type_name::<T>(),
                error = %e,
                "provide_context: failed to serialize context value — context not registered"
            );
            return false;
        }
    };
    with_context_frames(|stack| {
        let mut borrow = stack.borrow_mut();
        if borrow.is_empty() {
            borrow.push(alloc_context_frame());
        }
        drop(borrow);
        top_context_frame_mut(|frame| {
            frame.insert(ctx.id, json.clone());
        });
    });
    if let Some(render) = current_context() {
        // Key the serialized context by the stable type name (not `TypeId`,
        // whose Debug form is opaque and can shift between compiler builds).
        render.register_context(std::any::type_name::<T>(), json);
    }
    true
}

/// Fallible context read — returns `None` when no ancestor provided the value.
pub fn try_use_context<T: DeserializeOwned + Clone + 'static>(ctx: &ContextId<T>) -> Option<T> {
    let from_stack = with_context_frames(|stack| {
        for &handle in stack.borrow().iter().rev() {
            let value = CONTEXT_FRAME_MAP.with(|map| {
                map.borrow()
                    .get(&handle)
                    .and_then(|frame| frame.get(&ctx.id).cloned())
            });
            if let Some(v) = value {
                if let Ok(val) = serde_json::from_value(v) {
                    return Some(val);
                }
            }
        }
        None
    });
    if let Some(val) = from_stack {
        return Some(val);
    }
    if let Some(render) = current_context() {
        if let Some(v) = render.get_context(std::any::type_name::<T>()) {
            return serde_json::from_value(v).ok();
        }
    }
    None
}

/// Read a context value from an ancestor provider.
pub fn use_context<T: DeserializeOwned + Clone + 'static>(ctx: &ContextId<T>) -> T {
    try_use_context(ctx).unwrap_or_else(|| {
        panic!(
            "use_context: no provider for `{}` — call provide_context first (if provide_context \
             returned false, the value failed to serialize)",
            std::any::type_name::<T>()
        )
    })
}

/// Push an empty context frame for a component subtree.
pub fn push_context_frame() -> ContextGuard {
    let handle = alloc_context_frame();
    with_context_frames(|stack| stack.borrow_mut().push(handle));
    ContextGuard { handle }
}

pub struct ContextGuard {
    handle: usize,
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        with_context_frames(|stack| {
            if stack.borrow().last() == Some(&self.handle) {
                stack.borrow_mut().pop();
            }
        });
        remove_context_frame(self.handle);
    }
}

/// Helper for macro-generated component wrappers.
pub fn context_id<T: 'static>() -> TypeId {
    TypeId::of::<T>()
}

/// Opaque context entry in the resumability payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextSnapshot {
    pub type_name: String,
    pub value: Value,
}
