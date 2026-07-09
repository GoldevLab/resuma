//! Staged per-request metadata for the next SSR page response.
//!
//! Storage is **task-local** when the request runs inside [`scope_page_staging`]
//! (installed around every request by `request_id_middleware`). Thread-local
//! storage is unsafe here: page renders use `block_in_place` + `block_on`,
//! which can interleave other tasks on the same worker thread and clobber the
//! staged CSRF token / CSP nonce mid-render. Code running outside a scoped
//! request task (tests, direct `render_to_string` callers) falls back to a
//! thread-local slot, preserving the previous synchronous behavior.

use std::cell::RefCell;
use std::future::Future;

#[derive(Default)]
struct PageStaging {
    cache_control: Option<String>,
    csrf: String,
    csp_nonce: String,
    status: Option<u16>,
}

tokio::task_local! {
    static PAGE_STAGING: RefCell<PageStaging>;
}

thread_local! {
    static FALLBACK_STAGING: RefCell<PageStaging> = RefCell::new(PageStaging::default());
}

/// Run `fut` with fresh, task-isolated page staging (one scope per request).
pub async fn scope_page_staging<F: Future>(fut: F) -> F::Output {
    PAGE_STAGING
        .scope(RefCell::new(PageStaging::default()), fut)
        .await
}

fn with_staging<R>(f: impl FnOnce(&mut PageStaging) -> R) -> R {
    let mut f = Some(f);
    match PAGE_STAGING.try_with(|cell| (f.take().expect("staging fn"))(&mut cell.borrow_mut())) {
        Ok(out) => out,
        Err(_) => {
            FALLBACK_STAGING.with(|cell| (f.take().expect("staging fn"))(&mut cell.borrow_mut()))
        }
    }
}

/// Clear all staged per-request page metadata (call at request boundaries).
pub fn clear_request_staging() {
    with_staging(|s| *s = PageStaging::default());
}

/// Stage the HTTP status for the page about to be returned (e.g. 404/500 error
/// pages). Defaults to 200 when unset.
pub fn stage_response_status(status: u16) {
    with_staging(|s| s.status = Some(status));
}

/// Take the staged HTTP status (consumed once per response).
pub fn take_response_status() -> Option<u16> {
    with_staging(|s| s.status.take())
}

/// Stage a `Cache-Control` header for the page about to be returned.
pub fn stage_response_cache_control(value: impl Into<String>) {
    let value = value.into();
    with_staging(|s| s.cache_control = Some(value));
}

/// When a response sets a session CSRF cookie, shared caches must not store it.
pub fn sanitize_cache_for_session(
    cache: Option<String>,
    sets_session_cookie: bool,
) -> Option<String> {
    if !sets_session_cookie {
        return cache;
    }
    if cache.as_deref().is_some_and(|c| {
        let lower = c.to_ascii_lowercase();
        lower.contains("public") || lower.contains("max-age")
    }) {
        tracing::warn!(
            "overriding Cache-Control to private, no-store — page sets CSRF cookie"
        );
    }
    Some("private, no-store".to_string())
}

/// Take a staged cache header (consumed once per response).
pub fn take_response_cache_control() -> Option<String> {
    with_staging(|s| s.cache_control.take())
}

/// Stage the CSRF token for forms rendered during this page pass.
pub fn stage_page_csrf(token: impl Into<String>) {
    let token = token.into();
    with_staging(|s| s.csrf = token);
}

/// CSRF token for the current page render (forms).
pub fn page_csrf() -> String {
    with_staging(|s| s.csrf.clone())
}

/// Stage the CSP nonce for inline/module scripts rendered during this page pass.
pub fn stage_page_csp_nonce(nonce: impl Into<String>) {
    let nonce = nonce.into();
    with_staging(|s| s.csp_nonce = nonce);
}

/// CSP nonce for the current page render (client components, inline scripts).
pub fn page_csp_nonce() -> String {
    with_staging(|s| s.csp_nonce.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_cache_for_session_overrides_public() {
        let out = sanitize_cache_for_session(Some("public, max-age=3600".into()), true);
        assert_eq!(out.as_deref(), Some("private, no-store"));
    }

    #[test]
    fn sanitize_cache_for_session_keeps_when_no_cookie() {
        let cache = "public, max-age=60".to_string();
        let out = sanitize_cache_for_session(Some(cache.clone()), false);
        assert_eq!(out, Some(cache));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn page_staging_isolated_per_scoped_task() {
        let mut handles = Vec::new();
        for i in 0..32u32 {
            handles.push(tokio::spawn(async move {
                scope_page_staging(async {
                    let token = format!("csrf-{i:04}");
                    let nonce = format!("nonce-{i:04}");
                    stage_page_csrf(token.clone());
                    stage_page_csp_nonce(nonce.clone());
                    tokio::task::yield_now().await;
                    assert_eq!(page_csrf(), token);
                    assert_eq!(page_csp_nonce(), nonce);
                })
                .await
            }));
        }
        for h in handles {
            h.await.unwrap();
        }
    }
}
