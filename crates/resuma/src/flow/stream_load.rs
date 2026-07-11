//! Deferred streaming loaders — shell first, loader HTML chunks after.

use std::collections::HashSet;
use std::pin::Pin;

use crate::core::context::ResumePayload;
use crate::core::stream_chunk;
use crate::core::view::{Attr, AttrValue, Child, Element, View};
use crate::ssr::{render_view, stream_head, stream_tail, PageOptions, StreamChunk};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde_json::Value;

use super::cache::loader_cache;
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
    payload: ResumePayload,
    plan: DeferredStreamPlan,
) -> Pin<Box<dyn futures_util::Stream<Item = StreamChunk> + Send>> {
    let path = path.to_string();
    Box::pin(async_stream::stream! {
        let shell_html = render_view(&shell);
        yield Ok(stream_head(&opts, &path));
        yield Ok(shell_html.clone());

        let mut req = plan.request;
        for name in plan.deferred {
            let prefetched = plan.prefetched.get(&name).cloned();
            let resolved = match prefetched {
                Some(result) => result,
                None => {
                    // Loader was not prefetched (should not happen when plan is built correctly).
                    continue;
                }
            };

            match resolved {
                Ok(value) => {
                    if let Some(cache) = loader_cache(&name) {
                        req.cache_control.insert(name.clone(), cache);
                    }
                    let inner = render_view(&render_chunk_view(&name, &value));
                    let chunk = render_view(&stream_chunk(&name, inner));
                    yield Ok(chunk);
                }
                Err(err) => {
                    let inner = render_view(&View::Element(Element {
                        tag: "p".into(),
                        attrs: vec![Attr {
                            name: "class".into(),
                            value: AttrValue::Static("resuma-stream-error".into()),
                        }],
                        children: vec![Child::Text(format!(
                            "Error loading `{name}`: {err}"
                        ))],
                        dom_id: None,
                    }));
                    let chunk = render_view(&stream_chunk(&name, inner));
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
    payload: &ResumePayload,
) -> Option<Pin<Box<dyn futures_util::Stream<Item = StreamChunk> + Send>>> {
    let plan = take_deferred_stream_plan()?;
    Some(render_deferred_page_stream(
        shell,
        opts.clone(),
        path,
        payload.clone(),
        plan,
    ))
}

pub fn install_deferred_stream_hook() {
    crate::server::set_deferred_stream_hook(deferred_stream_hook);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::view::{Child, Element, View};
    use crate::flow::load::LoaderError;
    use crate::ssr::escape_text;

    #[test]
    fn stream_error_html_escapes_loader_message() {
        let err = LoaderError::new(500, "<script>alert(1)</script>");
        let html = render_view(&View::Element(Element {
            tag: "p".into(),
            attrs: vec![crate::core::view::Attr {
                name: "class".into(),
                value: crate::core::view::AttrValue::Static("resuma-stream-error".into()),
            }],
            children: vec![Child::Text(format!(
                "Error loading `items`: {}",
                err.message
            ))],
            dom_id: None,
        }));
        assert!(!html.contains("<script>"));
        assert!(html.contains(&escape_text("<script>alert(1)</script>")));
    }
}

#[ctor::ctor(unsafe)]
fn auto_install_deferred_stream() {
    install_deferred_stream_hook();
}
