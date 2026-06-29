//! Cooperative cancellation — pause aborts in-flight workers (no external broker).

use std::future::Future;

use tokio_util::sync::CancellationToken;

use crate::core::{Result, ResumaError};

/// Create a fresh cancellation scope for one graph execution.
pub fn new_scope() -> CancellationToken {
    CancellationToken::new()
}

/// Run an async block until completion or cancellation.
pub async fn run_cancellable<T, F>(token: &CancellationToken, fut: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    tokio::select! {
        result = fut => result,
        () = token.cancelled() => Err(ResumaError::Cancelled),
    }
}

/// Returns `Err(Cancelled)` when the scope has been signalled (e.g. pause).
pub fn check(token: &CancellationToken) -> Result<()> {
    if token.is_cancelled() {
        Err(ResumaError::Cancelled)
    } else {
        Ok(())
    }
}
