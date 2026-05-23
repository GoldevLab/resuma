//! Per-request runtime state for Resuma Flow page renders.

use std::cell::RefCell;
use std::collections::BTreeMap;

use serde::de::DeserializeOwned;
use serde_json::Value;

use super::cache::loader_cache;
use super::load::LoadValue;
use super::load::LoaderError;
use super::registry::dispatch_load;
use super::request::FlowRequest;
use super::stream_load::is_stream_loader;

thread_local! {
    static FLOW: RefCell<Option<FlowRuntime>> = const { RefCell::new(None) };
    static DEFERRED_PLAN: RefCell<Option<DeferredStreamPlan>> = const { RefCell::new(None) };
}

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

/// Enable deferred `#[load(stream)]` resolution for the next page render.
pub fn set_deferred_streaming(enabled: bool) {
    FLOW.with(|cell| {
        if let Some(rt) = cell.borrow_mut().as_mut() {
            rt.deferred_streaming = enabled;
        }
    });
}

pub fn stage_deferred_stream_plan(deferred: Vec<String>, request: FlowRequest) {
    if deferred.is_empty() {
        return;
    }
    DEFERRED_PLAN.with(|cell| {
        *cell.borrow_mut() = Some(DeferredStreamPlan { deferred, request });
    });
}

pub fn take_deferred_stream_plan() -> Option<DeferredStreamPlan> {
    DEFERRED_PLAN.with(|cell| cell.borrow_mut().take())
}

/// Install the active [`FlowRequest`] for the duration of a page render.
pub fn with_request<R>(req: FlowRequest, f: impl FnOnce() -> R) -> (R, FlowRequest) {
    FLOW.with(|cell| {
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
    FLOW.with(|cell| {
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

/// Resolve a `#[load]` handler by name. Panics on failure — prefer [`try_use_load`] in production pages.
pub fn use_load<T: DeserializeOwned>(name: &str) -> T {
    try_use_load(name).unwrap_or_else(|e| panic!("loader `{name}` failed: {e}"))
}

/// Fallible loader accessor with structured errors.
pub fn try_use_load<T: DeserializeOwned>(name: &str) -> Result<T, LoaderError> {
    FLOW.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let rt = borrow
            .as_mut()
            .expect("use_load requires FlowRequest — wrap render in with_request()");

        if let Some(err) = rt.load_errors.get(name) {
            return Err(err.clone());
        }

        if let Some(cached) = rt.loads.get(name) {
            return serde_json::from_value(cached.clone())
                .map_err(|e| LoaderError::new(500, format!("decode `{name}`: {e}")));
        }

        let req = rt.request.clone().expect("FlowRequest missing");
        let value = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(dispatch_load(name, req))
        });

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
    let pending = FLOW.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let rt = borrow
            .as_mut()
            .expect("use_load requires FlowRequest — wrap render in with_request()");

        if rt.deferred_streaming && is_stream_loader(name) {
            if !rt.deferred_loaders.iter().any(|n| n == name) {
                rt.deferred_loaders.push(name.to_string());
            }
            true
        } else {
            false
        }
    });

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
    FLOW.with(|cell| cell.borrow().as_ref()?.request.clone())
}

/// First loader error recorded during this render, if any.
pub fn first_load_error() -> Option<LoaderError> {
    FLOW.with(|cell| cell.borrow().as_ref()?.load_errors.values().next().cloned())
}

/// Set a cache-control hint for a loader (used by `#[load(cache = "...")]`).
pub fn set_load_cache(name: &str, value: impl Into<String>) {
    FLOW.with(|cell| {
        if let Some(rt) = cell.borrow_mut().as_mut() {
            if let Some(req) = rt.request.as_mut() {
                req.cache_control.insert(name.to_string(), value.into());
            }
        }
    });
}
