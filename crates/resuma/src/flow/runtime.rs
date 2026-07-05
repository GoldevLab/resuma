//! Per-request runtime state for Resuma Flow page renders.
//!
//! Storage is **task-local** when the request runs inside [`scope_flow_runtime`]
//! (installed around every request by `request_id_middleware`). Thread-local
//! storage is unsafe here: page renders use `block_in_place` + `block_on`,
//! which can interleave other tasks on the same worker thread and clobber loader
//! state mid-render. Code running outside a scoped request task (tests, direct
//! render callers) falls back to a thread-local slot.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::future::Future;

use serde::de::DeserializeOwned;
use serde_json::Value;

use super::cache::loader_cache;
use super::load::LoadValue;
use super::load::LoaderError;
use super::registry::dispatch_load;
use super::request::FlowRequest;
use super::stream_load::is_stream_loader;

#[derive(Default)]
struct FlowRuntime {
    request: Option<FlowRequest>,
    loads: BTreeMap<String, Value>,
    load_errors: BTreeMap<String, LoaderError>,
    deferred_loaders: Vec<String>,
    deferred_streaming: bool,
}

/// Plan staged after shell render for deferred streaming SSR.
#[derive(Debug, Clone)]
pub struct DeferredStreamPlan {
    pub deferred: Vec<String>,
    pub request: FlowRequest,
}

tokio::task_local! {
    static FLOW: RefCell<Option<FlowRuntime>>;
    static DEFERRED_PLAN: RefCell<Option<DeferredStreamPlan>>;
}

thread_local! {
    static FALLBACK_FLOW: RefCell<Option<FlowRuntime>> = const { RefCell::new(None) };
    static FALLBACK_DEFERRED_PLAN: RefCell<Option<DeferredStreamPlan>> =
        const { RefCell::new(None) };
}

fn with_flow<R>(f: impl FnOnce(&RefCell<Option<FlowRuntime>>) -> R) -> R {
    let mut f = Some(f);
    match FLOW.try_with(|cell| (f.take().expect("flow fn"))(cell)) {
        Ok(out) => out,
        Err(_) => FALLBACK_FLOW.with(|cell| (f.take().expect("flow fn"))(cell)),
    }
}

fn with_deferred_plan<R>(f: impl FnOnce(&RefCell<Option<DeferredStreamPlan>>) -> R) -> R {
    let mut f = Some(f);
    match DEFERRED_PLAN.try_with(|cell| (f.take().expect("deferred fn"))(cell)) {
        Ok(out) => out,
        Err(_) => FALLBACK_DEFERRED_PLAN.with(|cell| (f.take().expect("deferred fn"))(cell)),
    }
}

/// Run `fut` with fresh, task-isolated Flow loader state (one scope per request).
pub async fn scope_flow_runtime<F: Future>(fut: F) -> F::Output {
    FLOW.scope(RefCell::new(None), async {
        DEFERRED_PLAN.scope(RefCell::new(None), fut).await
    })
    .await
}

/// Enable deferred `#[load(stream)]` resolution for the next page render.
pub fn set_deferred_streaming(enabled: bool) {
    with_flow(|cell| {
        if let Some(rt) = cell.borrow_mut().as_mut() {
            rt.deferred_streaming = enabled;
        }
    });
}

pub fn stage_deferred_stream_plan(deferred: Vec<String>, request: FlowRequest) {
    if deferred.is_empty() {
        return;
    }
    with_deferred_plan(|cell| {
        *cell.borrow_mut() = Some(DeferredStreamPlan { deferred, request });
    });
}

pub fn take_deferred_stream_plan() -> Option<DeferredStreamPlan> {
    with_deferred_plan(|cell| cell.borrow_mut().take())
}

/// Install the active [`FlowRequest`] for the duration of a page render.
pub fn with_request<R>(req: FlowRequest, f: impl FnOnce() -> R) -> (R, FlowRequest) {
    with_flow(|cell| {
        let prev = cell.borrow_mut().replace(FlowRuntime {
            request: Some(req.clone()),
            loads: BTreeMap::new(),
            load_errors: BTreeMap::new(),
            deferred_loaders: Vec::new(),
            deferred_streaming: false,
        });
        let out = f();
        let final_req = cell
            .borrow()
            .as_ref()
            .and_then(|rt| rt.request.clone())
            .unwrap_or(req);
        *cell.borrow_mut() = prev;
        (out, final_req)
    })
}

/// Begin a page render with optional deferred streaming mode.
pub fn with_request_deferred<R>(
    req: FlowRequest,
    deferred_streaming: bool,
    f: impl FnOnce() -> R,
) -> (R, FlowRequest, Vec<String>) {
    with_flow(|cell| {
        let prev = cell.borrow_mut().replace(FlowRuntime {
            request: Some(req.clone()),
            loads: BTreeMap::new(),
            load_errors: BTreeMap::new(),
            deferred_loaders: Vec::new(),
            deferred_streaming,
        });
        let out = f();
        let (final_req, deferred) = {
            let borrow = cell.borrow();
            let rt = borrow.as_ref().expect("FlowRuntime missing");
            (
                rt.request.clone().unwrap_or(req),
                rt.deferred_loaders.clone(),
            )
        };
        *cell.borrow_mut() = prev;
        (out, final_req, deferred)
    })
}

/// Resolve a `#[load]` handler by name. Returns [`LoadValue::Err`] on failure — use
/// [`load_boundary`] or match on the result. Prefer [`try_use_load`] when you need
/// a plain `Result`.
pub fn use_load<T: DeserializeOwned>(name: &str) -> LoadValue<T> {
    match try_use_load(name) {
        Ok(v) => LoadValue::Ok(v),
        Err(e) => LoadValue::Err(e),
    }
}

fn loader_no_runtime(name: &str) -> LoaderError {
    LoaderError::new(
        500,
        format!("loader `{name}` requires FlowRequest — wrap render in with_request()"),
    )
}

/// Fallible loader accessor with structured errors.
pub fn try_use_load<T: DeserializeOwned>(name: &str) -> Result<T, LoaderError> {
    enum Prepared {
        NoRuntime,
        Err(LoaderError),
        Cached(Value),
        Fetch(FlowRequest),
    }

    // Read what we need under a short borrow, then release it: holding the
    // RefCell borrow across `block_on(dispatch_load(..))` panics with
    // "already borrowed" when a loader nests another `use_load` call.
    let prepared = with_flow(|cell| {
        let borrow = cell.borrow();
        let Some(rt) = borrow.as_ref() else {
            return Prepared::NoRuntime;
        };

        if let Some(err) = rt.load_errors.get(name) {
            return Prepared::Err(err.clone());
        }
        if let Some(cached) = rt.loads.get(name) {
            return Prepared::Cached(cached.clone());
        }
        let Some(req) = rt.request.clone() else {
            return Prepared::NoRuntime;
        };
        Prepared::Fetch(req)
    });

    let req = match prepared {
        Prepared::NoRuntime => return Err(loader_no_runtime(name)),
        Prepared::Err(err) => return Err(err),
        Prepared::Cached(cached) => {
            return serde_json::from_value(cached)
                .map_err(|e| LoaderError::new(500, format!("decode `{name}`: {e}")));
        }
        Prepared::Fetch(req) => req,
    };

    let value = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(dispatch_load(name, req))
    });

    // Re-borrow to store the result (the runtime may have been used by nested
    // loads while the async work ran, but it is no longer borrowed here).
    with_flow(|cell| {
        let mut borrow = cell.borrow_mut();
        let Some(rt) = borrow.as_mut() else {
            return Err(loader_no_runtime(name));
        };

        match value {
            Ok(v) => {
                rt.loads.insert(name.to_string(), v.clone());
                if let Some(cache) = loader_cache(name) {
                    if let Some(req) = rt.request.as_mut() {
                        req.cache_control.insert(name.to_string(), cache);
                    }
                }
                serde_json::from_value(v)
                    .map_err(|e| LoaderError::new(500, format!("decode `{name}`: {e}")))
            }
            Err(err) => {
                let loader_err = LoaderError::new(500, err.to_string());
                rt.load_errors.insert(name.to_string(), loader_err.clone());
                Err(loader_err)
            }
        }
    })
}

/// Loader accessor for `#[load(stream)]` — returns [`LoadValue::Pending`] during deferred SSR.
pub fn try_use_load_value<T: DeserializeOwned>(name: &str) -> LoadValue<T> {
    let pending = with_flow(|cell| {
        let mut borrow = cell.borrow_mut();
        match borrow.as_mut() {
            None => None,
            Some(rt) => {
                if rt.deferred_streaming && is_stream_loader(name) {
                    if !rt.deferred_loaders.iter().any(|n| n == name) {
                        rt.deferred_loaders.push(name.to_string());
                    }
                    Some(true)
                } else {
                    Some(false)
                }
            }
        }
    });

    let Some(pending) = pending else {
        return LoadValue::Err(loader_no_runtime(name));
    };

    if pending {
        return LoadValue::Pending;
    }

    match try_use_load(name) {
        Ok(value) => LoadValue::Ok(value),
        Err(err) => LoadValue::Err(err),
    }
}

/// Current request, if inside a Flow page render.
pub fn current_request() -> Option<FlowRequest> {
    with_flow(|cell| cell.borrow().as_ref()?.request.clone())
}

/// First loader error recorded during this render, if any.
pub fn first_load_error() -> Option<LoaderError> {
    with_flow(|cell| cell.borrow().as_ref()?.load_errors.values().next().cloned())
}

/// Set a cache-control hint for a loader (used by `#[load(cache = "...")]`).
pub fn set_load_cache(name: &str, value: impl Into<String>) {
    with_flow(|cell| {
        if let Some(rt) = cell.borrow_mut().as_mut() {
            if let Some(req) = rt.request.as_mut() {
                req.cache_control.insert(name.to_string(), value.into());
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_use_load_without_runtime_returns_err() {
        let err = try_use_load::<serde_json::Value>("missing").unwrap_err();
        assert!(err.message.contains("with_request"));
    }

    #[test]
    fn try_use_load_value_without_runtime_returns_err() {
        match try_use_load_value::<serde_json::Value>("missing") {
            LoadValue::Err(e) => assert!(e.message.contains("with_request")),
            other => panic!("expected Err, got {other:?}"),
        }
    }
}
