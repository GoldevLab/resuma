//! Staged `Cache-Control` for the next SSR page response.

use std::cell::RefCell;

thread_local! {
    static STAGED: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Stage a `Cache-Control` header for the page about to be returned.
pub fn stage_response_cache_control(value: impl Into<String>) {
    STAGED.with(|cell| *cell.borrow_mut() = Some(value.into()));
}

/// Take a staged cache header (consumed once per response).
pub fn take_response_cache_control() -> Option<String> {
    STAGED.with(|cell| cell.borrow_mut().take())
}
