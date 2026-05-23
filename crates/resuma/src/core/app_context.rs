//! Component context — `provide_context` / `use_context` for descendant trees.
//!
//! Context values are serializable and travel in the resumability payload so
//! descendant components can read them on the client after resume.

use std::any::TypeId;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use super::context::current_context;

thread_local! {
    static CONTEXT_STACK: RefCell<Vec<Rc<BTreeMap<TypeId, Value>>>> = const { RefCell::new(Vec::new()) };
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

impl<T: 'static> ContextId<T> {
    pub const fn new() -> Self {
        Self {
            id: TypeId::of::<T>(),
            _marker: std::marker::PhantomData,
        }
    }
}

/// Provide a context value visible to this component and its descendants.
pub fn provide_context<T: Serialize + Clone + 'static>(ctx: &ContextId<T>, value: T) {
    let json = serde_json::to_value(&value).unwrap_or(Value::Null);
    CONTEXT_STACK.with(|stack| {
        let mut borrow = stack.borrow_mut();
        if borrow.is_empty() {
            let mut map = BTreeMap::new();
            map.insert(ctx.id, json.clone());
            borrow.push(Rc::new(map));
        } else {
            let top = Rc::make_mut(borrow.last_mut().unwrap());
            top.insert(ctx.id, json.clone());
        }
    });
    if let Some(render) = current_context() {
        render.register_context(ctx.id, json);
    }
}

/// Read a context value from an ancestor provider.
pub fn use_context<T: DeserializeOwned + Clone + 'static>(ctx: &ContextId<T>) -> T {
    CONTEXT_STACK.with(|stack| {
        for frame in stack.borrow().iter().rev() {
            if let Some(v) = frame.get(&ctx.id) {
                if let Ok(val) = serde_json::from_value(v.clone()) {
                    return val;
                }
            }
        }
    });
    // Fallback: read from SSR payload registry (client resume path).
    if let Some(render) = current_context() {
        if let Some(v) = render.get_context(ctx.id) {
            if let Ok(val) = serde_json::from_value(v) {
                return val;
            }
        }
    }
    panic!(
        "use_context: no provider for `{}` — call provide_context first",
        std::any::type_name::<T>()
    );
}

/// Push an empty context frame for a component subtree.
pub fn push_context_frame() -> ContextGuard {
    CONTEXT_STACK.with(|stack| stack.borrow_mut().push(Rc::new(BTreeMap::new())));
    ContextGuard
}

pub struct ContextGuard;

impl Drop for ContextGuard {
    fn drop(&mut self) {
        CONTEXT_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
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
