//! Staged per-request metadata for the next SSR page response.

use std::cell::RefCell;

thread_local! {
    static STAGED: RefCell<Option<String>> = const { RefCell::new(None) };
    static PAGE_CSRF: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Stage a `Cache-Control` header for the page about to be returned.
pub fn stage_response_cache_control(value: impl Into<String>) {
    STAGED.with(|cell| *cell.borrow_mut() = Some(value.into()));
}

/// Take a staged cache header (consumed once per response).
pub fn take_response_cache_control() -> Option<String> {
    STAGED.with(|cell| cell.borrow_mut().take())
}

/// Stage the CSRF token for forms rendered during this page pass.
pub fn stage_page_csrf(token: impl Into<String>) {
    PAGE_CSRF.with(|cell| *cell.borrow_mut() = token.into());
}

/// CSRF token for the current page render (forms).
pub fn page_csrf() -> String {
    PAGE_CSRF.with(|cell| cell.borrow().clone())
}
