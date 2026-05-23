//! Staged request path for SEO tags on the next SSR response.

use std::cell::RefCell;

thread_local! {
    static STAGED_PATH: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn stage_response_path(path: impl Into<String>) {
    STAGED_PATH.with(|cell| *cell.borrow_mut() = Some(path.into()));
}

pub fn take_response_path() -> String {
    STAGED_PATH.with(|cell| cell.borrow_mut().take().unwrap_or_else(|| "/".into()))
}
