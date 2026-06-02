//! Streaming SSR — send the HTML shell first, then stream body chunks.

use std::pin::Pin;

use crate::core::context::{RenderContext, RenderMode, ResumePayload};
use crate::core::{with_context, View};
use futures_util::Stream;

use super::escape::escape_text;
use super::seo;
use crate::{render_view, PageOptions};

/// Head + open body sent before streamed content.
pub fn stream_head(opts: &PageOptions, path: &str) -> String {
    let lang = if opts.lang.is_empty() {
        "en"
    } else {
        &opts.lang
    };
    let title = seo::page_title(opts, path);
    let description = seo::page_description(opts, path);
    let seo_tags = seo::seo_head_tags(opts, path);
    let json_ld = seo::json_ld_script(&opts.json_ld);
    let head = super::apply_head_csp_nonce(&opts.head, &opts.csp_nonce);

    let stylesheet = opts
        .stylesheet
        .as_ref()
        .map(|s| format!(r#"<link rel="stylesheet" href="{s}" />"#))
        .unwrap_or_default();

    format!(
        r#"<!doctype html>
<html lang="{lang}">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<meta name="description" content="{description}" />
<title>{title}</title>
{json_ld}{seo_tags}
{stylesheet}
{head}
</head>
<body>
<div id="resuma-root">"#,
        lang = lang,
        title = escape_text(&title),
        description = escape_text(&description),
        seo_tags = seo_tags,
        json_ld = json_ld,
        head = head,
        stylesheet = stylesheet,
    )
}

/// Closing tags + optional resumability payload + loader bootstrap.
pub fn stream_tail(opts: &PageOptions, body_html: &str, payload: &ResumePayload) -> String {
    let scripts = super::client_scripts(opts, body_html, payload);
    let dev_script = crate::server::dev::dev_reload_script(&opts.csp_nonce);
    format!(
        r#"</div>
{scripts}
{dev_script}
</body>
</html>"#,
        scripts = scripts,
        dev_script = dev_script,
    )
}

/// Placeholder rendered while a streamed loader slot resolves.
pub fn stream_placeholder(name: &str) -> String {
    format!(
        r#"<template data-r-stream="{name}"><p class="resuma-stream-loading">Loading…</p></template>"#,
        name = escape_text(name),
    )
}

/// A single chunk in a streaming SSR response.
pub type StreamChunk = Result<String, String>;

/// Build a simple stream: head → body chunks → tail.
pub fn build_page_stream(
    opts: PageOptions,
    path: &str,
    body_html: String,
    payload: ResumePayload,
    body_chunks: Vec<String>,
) -> Pin<Box<dyn Stream<Item = StreamChunk> + Send>> {
    let path = path.to_string();
    Box::pin(async_stream::stream! {
        yield Ok(stream_head(&opts, &path));
        for chunk in body_chunks {
            yield Ok(chunk);
        }
        yield Ok(stream_tail(&opts, &body_html, &payload));
    })
}

/// Render a view and split it into streamable parts (head, body HTML, tail).
pub fn render_stream_parts<F>(
    opts: &PageOptions,
    path: &str,
    build_view: F,
) -> (String, String, String)
where
    F: FnOnce() -> View,
{
    let ctx = RenderContext::new(RenderMode::Ssr);
    let (body, payload) = with_context(ctx.clone(), || {
        let view = build_view();
        (render_view(&view), ctx.snapshot())
    });
    (
        stream_head(opts, path),
        body.clone(),
        stream_tail(opts, &body, &payload),
    )
}

/// Full streaming page: head is sent before body rendering completes when used
/// with an async wrapper; this helper returns the three chunks synchronously.
pub fn render_to_stream<F>(
    opts: &PageOptions,
    path: &str,
    build_view: F,
) -> Pin<Box<dyn Stream<Item = StreamChunk> + Send>>
where
    F: FnOnce() -> View + Send + 'static,
{
    let opts = opts.clone();
    let path = path.to_string();
    Box::pin(async_stream::stream! {
        let (head, body, tail) = render_stream_parts(&opts, &path, build_view);
        yield Ok(head);
        yield Ok(body);
        yield Ok(tail);
    })
}
