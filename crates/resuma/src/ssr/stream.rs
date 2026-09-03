//! Streaming SSR — send the HTML shell first, then stream body chunks.

use std::pin::Pin;

use crate::core::context::{RenderContext, RenderMode, ResumePayload};
use crate::core::{with_context, View};
use futures_util::Stream;

use super::escape::escape_attr;
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
    let dir_attr = super::document_dir_attr(opts);
    let theme_attr = super::document_theme_attr(opts);
    let theme_boot = super::html_theme_boot(opts);
    let title = seo::page_title(opts, path);
    let description = seo::page_description(opts, path);
    let seo_tags = seo::seo_head_tags(opts, path);
    let json_ld = seo::document_json_ld(opts);
    let head = super::apply_head_csp_nonce(&opts.head, &opts.csp_nonce);

    let stylesheet = opts
        .stylesheet
        .as_ref()
        .map(|s| format!(r#"<link rel="stylesheet" href="{}" />"#, escape_attr(s)))
        .unwrap_or_default();

    format!(
        r#"<!doctype html>
<html lang="{lang}"{dir_attr}{theme_attr}>
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
{theme_boot}<meta name="description" content="{description}" />
<title>{title}</title>
{json_ld}{seo_tags}
{stylesheet}
<link rel="stylesheet" href="{ui_css}" />
{head}
</head>
<body>
<div id="resuma-root">"#,
        lang = escape_attr(lang),
        dir_attr = dir_attr,
        theme_attr = theme_attr,
        theme_boot = theme_boot,
        title = escape_text(&title),
        description = escape_attr(&description),
        seo_tags = seo_tags,
        json_ld = json_ld,
        head = head,
        stylesheet = stylesheet,
        ui_css = escape_attr(crate::server::runtime_asset::ui_css_url()),
    )
}

/// Closing tags + optional resumability payload + loader bootstrap.
pub fn stream_tail(opts: &PageOptions, body_html: &str, payload: &ResumePayload) -> String {
    let scripts = super::client_scripts(opts, body_html, payload);
    let live = super::live_region_html(body_html, payload);
    let dev_script = crate::server::dev::dev_reload_script(&opts.csp_nonce);
    format!(
        r#"</div>
{live}
{scripts}
{dev_script}
</body>
</html>"#,
        live = live,
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
///
/// `stream_head` runs **before** the async generator so per-request staging
/// (`set_page_title`, robots, JSON-LD) is read on the same task that rendered
/// the view. Yielding `stream_head` inside `async_stream` can run on another
/// worker and miss task-local overrides.
pub fn build_page_stream(
    opts: PageOptions,
    path: &str,
    body_html: String,
    payload: ResumePayload,
    body_chunks: Vec<String>,
) -> Pin<Box<dyn Stream<Item = StreamChunk> + Send>> {
    let head = stream_head(&opts, path);
    Box::pin(async_stream::stream! {
        yield Ok(head);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssr::PageOptions;

    #[test]
    fn stream_head_escapes_description_and_lang() {
        crate::server::page_cache::clear_request_staging();
        crate::server::page_cache::set_page_description(r#"x" http-equiv="refresh"#);
        let head = stream_head(
            &PageOptions {
                title: "App".into(),
                lang: r#"en" onclick="x"#.into(),
                ..Default::default()
            },
            "/",
        );
        assert!(
            !head.contains(r#"content="x" http-equiv"#),
            "stream head leaked description quotes: {head}"
        );
        assert!(
            !head.contains(r#"lang="en" onclick"#),
            "stream head leaked lang quotes: {head}"
        );
        crate::server::page_cache::clear_request_staging();
    }

    #[test]
    fn stream_tail_includes_live_region_for_overlays() {
        crate::server::page_cache::clear_request_staging();
        let (_head, body, tail) = render_stream_parts(
            &PageOptions {
                title: "t".into(),
                ..Default::default()
            },
            "/",
            || {
                crate::popup(
                    crate::PopupOpts {
                        id: "m".into(),
                        ..Default::default()
                    },
                    vec![crate::Child::Text("Open".into())],
                    vec![crate::Child::Text("Panel".into())],
                )
            },
        );
        assert!(body.contains("data-r-popup"), "{body}");
        assert!(tail.contains("id=\"r-live\""), "{tail}");
        crate::server::page_cache::clear_request_staging();
    }
}
