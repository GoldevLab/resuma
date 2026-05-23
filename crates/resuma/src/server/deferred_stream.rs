//! Optional deferred streaming hook installed by `resuma-flow`.

use std::pin::Pin;

use crate::core::view::View;
use crate::ssr::{PageOptions, StreamChunk};
use futures_util::Stream;

type DeferredStreamHook =
    fn(View, &PageOptions, &str) -> Option<Pin<Box<dyn Stream<Item = StreamChunk> + Send>>>;

static HOOK: parking_lot::RwLock<Option<DeferredStreamHook>> = parking_lot::RwLock::new(None);

/// Register renderer for deferred loader streaming (called from `resuma-flow` ctor).
pub fn set_deferred_stream_hook(hook: DeferredStreamHook) {
    *HOOK.write() = Some(hook);
}

/// When a deferred stream plan was staged, return the chunked response stream.
pub fn try_deferred_stream(
    shell: View,
    opts: &PageOptions,
    path: &str,
) -> Option<Pin<Box<dyn Stream<Item = StreamChunk> + Send>>> {
    (*HOOK.read())?(shell, opts, path)
}
