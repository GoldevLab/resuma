//! Deferred streaming loaders — shell first, loader HTML chunks after.

use std::collections::HashSet;
use std::pin::Pin;

use crate::core::stream_chunk;
use crate::core::view::View;
use crate::ssr::{render_body_and_payload, stream_head, stream_tail, PageOptions, StreamChunk};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde_json::Value;

use super::cache::loader_cache;
use super::registry::dispatch_load;
use super::runtime::{take_deferred_stream_plan, DeferredStreamPlan};

static STREAM_LOADERS: Lazy<RwLock<HashSet<String>>> = Lazy::new(|| RwLock::new(HashSet::new()));

pub type StreamChunkFn = fn(&Value) -> View;

static STREAM_CHUNKS: Lazy<RwLock<std::collections::HashMap<String, StreamChunkFn>>> =
    Lazy::new(|| RwLock::new(std::collections::HashMap::new()));

/// Mark a `#[load(stream)]` handler as deferred during streaming SSR.
pub fn register_stream_loader(name: &str) {
    STREAM_LOADERS.write().insert(name.to_string());
}

pub fn is_stream_loader(name: &str) -> bool {
    STREAM_LOADERS.read().contains(name)
}

/// Register HTML renderer for a streamed loader chunk (`{name}_stream_view`).
pub fn register_stream_chunk(name: &str, f: StreamChunkFn) {
    STREAM_CHUNKS.write().insert(name.to_string(), f);
}

fn render_chunk_view(name: &str, value: &Value) -> View {
    if let Some(f) = STREAM_CHUNKS.read().get(name).copied() {
        return f(value);
    }
    View::Element(crate::core::view::Element {
        tag: "pre".into(),
        attrs: vec![crate::core::view::Attr {
            name: "class".into(),
            value: crate::core::view::AttrValue::Static("resuma-stream-loaded".into()),
        }],
        children: vec![crate::core::view::Child::Text(
            serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()),
        )],
        dom_id: None,
    })
}

/// Build a chunked HTTP stream: head → shell → deferred loader chunks → tail.
pub fn render_deferred_page_stream(
    shell: View,
    opts: PageOptions,
    path: &str,
    plan: DeferredStreamPlan,
) -> Pin<Box<dyn futures_util::Stream<Item = StreamChunk> + Send>> {
    let path = path.to_string();
    Box::pin(async_stream::stream! {
        let (shell_html, payload) = render_body_and_payload(&shell);
        yield Ok(stream_head(&opts, &path));
        yield Ok(shell_html.clone());

        let mut req = plan.request;
        for name in plan.deferred {
            match dispatch_load(&name, req.clone()).await {
                Ok(value) => {
                    if let Some(cache) = loader_cache(&name) {
                        req.cache_control.insert(name.clone(), cache);
                    }
                    let inner = crate::ssr::render_view(&render_chunk_view(&name, &value));
                    let chunk = crate::ssr::render_view(&stream_chunk(&name, inner));
                    yield Ok(chunk);
                }
                Err(err) => {
                    let inner = format!(
                        r#"<p class="resuma-error">Error loading `{name}`: {err}</p>"#,
                        name = name,
                        err = err,
                    );
                    let chunk = crate::ssr::render_view(&stream_chunk(&name, inner));
                    yield Ok(chunk);
                }
            }
        }

        yield Ok(stream_tail(&opts, &shell_html, &payload));
    })
}

fn deferred_stream_hook(
    shell: View,
    opts: &PageOptions,
    path: &str,
) -> Option<Pin<Box<dyn futures_util::Stream<Item = StreamChunk> + Send>>> {
    let plan = take_deferred_stream_plan()?;
    Some(render_deferred_page_stream(shell, opts.clone(), path, plan))
}

pub fn install_deferred_stream_hook() {
    crate::server::set_deferred_stream_hook(deferred_stream_hook);
}

#[ctor::ctor]
fn auto_install_deferred_stream() {
    install_deferred_stream_hook();
}
