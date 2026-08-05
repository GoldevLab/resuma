//! Server-side data loading — `#[load]` handlers run before page render.

use crate::core::view::View;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::request::FlowRequest;

/// Result of a `#[load]` handler exposed to components.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum LoadValue<T> {
    Ok(T),
    Err(LoaderError),
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderError {
    pub status: u16,
    pub message: String,
}

impl std::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (status {})", self.message, self.status)
    }
}

impl LoaderError {
    pub fn new(status: u16, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

/// Render a `LoadValue` from `#[load]` with explicit pending and error branches.
pub fn load_boundary<T>(
    load: LoadValue<T>,
    ok: impl FnOnce(T) -> View,
    err: impl FnOnce(LoaderError) -> View,
    pending: impl FnOnce() -> View,
) -> View {
    match load {
        LoadValue::Ok(v) => ok(v),
        LoadValue::Err(e) => err(e),
        LoadValue::Pending => pending(),
    }
}

/// Combine two `#[load]` values into one boundary (avoids nested `load_boundary`).
///
/// Pending wins if any loader is still streaming; otherwise the first error wins.
pub fn load_all2<A, B>(
    a: LoadValue<A>,
    b: LoadValue<B>,
    ok: impl FnOnce(A, B) -> View,
    err: impl FnOnce(LoaderError) -> View,
    pending: impl FnOnce() -> View,
) -> View {
    if matches!(a, LoadValue::Pending) || matches!(b, LoadValue::Pending) {
        return pending();
    }
    match (a, b) {
        (LoadValue::Err(e), _) | (_, LoadValue::Err(e)) => err(e),
        (LoadValue::Ok(a), LoadValue::Ok(b)) => ok(a, b),
        _ => pending(),
    }
}

/// Combine three `#[load]` values into one boundary (queue / dashboard pages).
pub fn load_all3<A, B, C>(
    a: LoadValue<A>,
    b: LoadValue<B>,
    c: LoadValue<C>,
    ok: impl FnOnce(A, B, C) -> View,
    err: impl FnOnce(LoaderError) -> View,
    pending: impl FnOnce() -> View,
) -> View {
    if matches!(a, LoadValue::Pending)
        || matches!(b, LoadValue::Pending)
        || matches!(c, LoadValue::Pending)
    {
        return pending();
    }
    match (a, b, c) {
        (LoadValue::Err(e), _, _) | (_, LoadValue::Err(e), _) | (_, _, LoadValue::Err(e)) => err(e),
        (LoadValue::Ok(a), LoadValue::Ok(b), LoadValue::Ok(c)) => ok(a, b, c),
        _ => pending(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::view::View;

    #[test]
    fn load_all3_waits_for_any_pending() {
        let v = load_all3(
            LoadValue::<i32>::Ok(1),
            LoadValue::<i32>::Pending,
            LoadValue::<i32>::Ok(3),
            |_: i32, _: i32, _: i32| View::text("ok"),
            |_| View::text("err"),
            || View::text("pending"),
        );
        assert!(matches!(v, View::Text(t) if t == "pending"));
    }

    #[test]
    fn load_all3_ok_when_all_ready() {
        let v = load_all3(
            LoadValue::Ok(1i32),
            LoadValue::Ok(2i32),
            LoadValue::Ok(3i32),
            |a, b, c| View::text(format!("{a}+{b}+{c}")),
            |_| View::text("err"),
            || View::text("pending"),
        );
        assert!(matches!(v, View::Text(t) if t == "1+2+3"));
    }
}

/// Type-erased loader dispatch signature used by the Flow registry.
pub type LoadFn = fn(&FlowRequest) -> LoadDispatch;

pub enum LoadDispatch {
    Ready(Value),
    Pending,
}
