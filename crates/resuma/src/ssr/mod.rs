//! Server-Side Rendering for Resuma.
//!
//! Two flavours of renderer live here:
//!
//!  * [`render_to_string`] — full page render. Wraps the view in a `<!doctype html>`
//!    document, embeds the resumability payload as a `<script type="resuma/state">…</script>`
//!    block, and injects the bootstrap loader for the tiny client runtime.
//!
//!  * [`render_view`] — partial render. Returns just the HTML for a `View` tree.
//!    Used by the dev server for island-only re-renders.
//!
//! The renderer never executes JavaScript itself. It only walks the tree
//! and writes characters. Everything needed for resumability lives inside
//! the HTML payload it produces.

use std::fmt::Write;
use std::rc::Rc;

use crate::core::{
    context::{RenderContext, RenderMode, ResumePayload, page_needs_client},
    handler::HandlerRef,
    serialize::encode_payload,
    view::{Attr, AttrValue, Child, Element, Fragment, Island, View},
    with_context,
};

mod escape;
pub mod pwa;
pub mod seo;
pub mod stream;
use escape::{escape_attr, escape_text};

pub use stream::{build_page_stream, render_stream_parts, render_to_stream, stream_head, stream_tail, stream_placeholder, StreamChunk};

/// PWA install / theming options injected into `<head>`.
#[derive(Debug, Clone, Default)]
pub struct PwaOptions {
    pub enabled: bool,
    pub name: String,
    pub short_name: String,
    pub description: String,
    pub theme_color: String,
    pub background_color: String,
}

/// Configuration for full-page rendering.
#[derive(Debug, Clone, Default)]
pub struct PageOptions {
    pub title: String,
    pub description: String,
    pub head: String,
    pub lang: String,
    /// Public site origin, e.g. `https://resuma-docs.fly.dev` (no trailing slash).
    pub site_url: String,
    /// Open Graph image path or absolute URL.
    pub og_image: String,
    pub og_type: String,
    /// Optional JSON-LD `<script>` inner JSON (not HTML-escaped).
    pub json_ld: String,
    /// Override canonical URL (absolute). Defaults to `site_url + path`.
    pub canonical: Option<String>,
    /// Progressive Web App options (`manifest`, service worker registration).
    pub pwa: Option<PwaOptions>,
    /// Client bootstrap script. Defaults to `/_resuma/loader.js`.
    pub loader_src: String,
    /// Legacy alias for `loader_src` when set explicitly.
    pub runtime_src: String,
    pub stylesheet: Option<String>,
    /// Per-request CSP nonce for inline resumability script.
    #[doc(hidden)]
    pub csp_nonce: String,
    /// Per-request CSRF token embedded in the state payload.
    #[doc(hidden)]
    pub csrf_token: String,
}

/// Render a `View` produced by a component to a complete HTML document.
pub fn render_to_string<F>(opts: &PageOptions, build_view: F) -> String
where
    F: FnOnce() -> View,
{
    render_to_string_at_path(opts, "/", build_view)
}

/// Like [`render_to_string`] but sets canonical/OG tags from the request path.
pub fn render_to_string_at_path<F>(opts: &PageOptions, path: &str, build_view: F) -> String
where
    F: FnOnce() -> View,
{
    let ctx = RenderContext::new(RenderMode::Ssr);
    let (body, payload) = with_context(ctx.clone(), || {
        let view = build_view();
        let mut buf = String::new();
        write_view(&mut buf, &view);
        (buf, ctx.snapshot())
    });

    wrap_document(opts, &body, &payload, path)
}

fn loader_src(opts: &PageOptions) -> &str {
    if !opts.runtime_src.is_empty() {
        &opts.runtime_src
    } else if !opts.loader_src.is_empty() {
        &opts.loader_src
    } else {
        "/_resuma/loader.js"
    }
}

pub(crate) fn client_scripts(opts: &PageOptions, body_html: &str, payload: &ResumePayload) -> String {
    if !page_needs_client(payload, body_html) {
        return String::new();
    }
    let mut payload = payload.clone();
    if !opts.csrf_token.is_empty() {
        payload.csrf_token = Some(opts.csrf_token.clone());
    }
    let payload_json = encode_payload(&payload);
    let nonce_attr = if opts.csp_nonce.is_empty() {
        String::new()
    } else {
        format!(r#" nonce="{}""#, escape_attr(&opts.csp_nonce))
    };
    format!(
        r#"<script type="resuma/state" id="resuma-state"{nonce_attr}>{payload}</script>
<script type="module" src="{loader}"></script>"#,
        payload = payload_json,
        loader = loader_src(opts),
        nonce_attr = nonce_attr,
    )
}

/// Render a `View` body and capture the resumability payload in one pass.
pub fn render_body_and_payload(view: &View) -> (String, ResumePayload) {
    let ctx = RenderContext::new(RenderMode::Ssr);
    let body = with_context(ctx.clone(), || {
        let mut buf = String::new();
        write_view(&mut buf, view);
        buf
    });
    (body, ctx.snapshot())
}

/// Render only the body of a `View`, no document scaffolding.
pub fn render_view(view: &View) -> String {
    let mut buf = String::new();
    write_view(&mut buf, view);
    buf
}

/// Render a view in a context — used by the server when it has its own ctx.
pub fn render_with_context(ctx: Rc<RenderContext>, view: &View) -> String {
    with_context(ctx, || {
        let mut buf = String::new();
        write_view(&mut buf, view);
        buf
    })
}

fn wrap_document(opts: &PageOptions, body_html: &str, payload: &ResumePayload, path: &str) -> String {
    let lang = if opts.lang.is_empty() { "en" } else { &opts.lang };
    let title = seo::page_title(opts, path);
    let description = seo::page_description(opts, path);
    let seo_tags = seo::seo_head_tags(opts, path);
    let stylesheet = opts
        .stylesheet
        .as_ref()
        .map(|s| format!(r#"<link rel="stylesheet" href="{}" />"#, s))
        .unwrap_or_default();
    let scripts = client_scripts(opts, body_html, payload);

    format!(
        r#"<!doctype html>
<html lang="{lang}">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<meta name="description" content="{description}" />
<title>{title}</title>
{seo_tags}
{stylesheet}
{head}
</head>
<body>
<div id="resuma-root">{body}</div>
{scripts}
</body>
</html>"#,
        lang = lang,
        title = escape_text(&title),
        description = escape_text(&description),
        seo_tags = seo_tags,
        head = opts.head,
        stylesheet = stylesheet,
        body = body_html,
        scripts = scripts,
    )
}

fn write_view(buf: &mut String, view: &View) {
    match view {
        View::Empty => {}
        View::Text(t) => buf.push_str(&escape_text(t)),
        View::Raw(html) => buf.push_str(html),
        View::Dynamic(d) => {
            // SSR-time we render the snapshot value. Wrap in a marker so the
            // runtime knows where to bind reactivity.
            let value = match &d.snapshot {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            let formatted = match &d.format {
                Some(fmt) => fmt.replace("{}", &value),
                None => value,
            };
            let _ = write!(buf, r#"<resuma-dyn data-r-signal="{}">{}</resuma-dyn>"#, d.signal, escape_text(&formatted));
        }
        View::Element(el) => write_element(buf, el),
        View::Fragment(Fragment { children }) => {
            for c in children { write_child(buf, c); }
        }
        View::Component(c) => write_view(buf, &c.view),
        View::Island(island) => write_island(buf, island),
        View::Slot(slot) => {
            let resolved = crate::core::resolve_slot(slot.name.as_deref());
            write_view(buf, &resolved);
        }
    }
}

fn write_child(buf: &mut String, child: &Child) {
    match child {
        Child::Text(t) => buf.push_str(&escape_text(t)),
        Child::View(v) => write_view(buf, v),
    }
}

fn write_element(buf: &mut String, el: &Element) {
    let _ = write!(buf, "<{}", el.tag);

    if let Some(id) = &el.dom_id {
        let _ = write!(buf, r#" id="{}""#, escape_attr(id));
    }

    for attr in &el.attrs {
        write_attr(buf, attr);
    }

    if is_void_element(&el.tag) && el.children.is_empty() {
        let _ = write!(buf, " />");
        return;
    }

    let _ = write!(buf, ">");
    for c in &el.children { write_child(buf, c); }
    let _ = write!(buf, "</{}>", el.tag);
}

fn write_attr(buf: &mut String, attr: &Attr) {
    let name = &attr.name;
    match &attr.value {
        AttrValue::Static(s) => {
            let _ = write!(buf, r#" {}="{}""#, name, escape_attr(s));
        }
        AttrValue::Bool(true) => {
            let _ = write!(buf, " {}", name);
        }
        AttrValue::Bool(false) => {}
        AttrValue::Dynamic { signal, format } => {
            let f = format.as_deref().unwrap_or("{}");
            let _ = write!(buf, r#" {}="" data-r-bind:{}="{}|{}""#, name, name, signal, escape_attr(f));
        }
        AttrValue::Handler(h) => write_handler_attr(buf, h),
        AttrValue::PreventDefault(ev) => {
            let _ = write!(buf, r#" data-r-prevent:{ev}="" "#, ev = ev);
        }
        AttrValue::StopPropagation(ev) => {
            let _ = write!(buf, r#" data-r-stop:{ev}="" "#, ev = ev);
        }
    }
}

fn write_handler_attr(buf: &mut String, h: &HandlerRef) {
    // data-r-on:click="<chunk>#<symbol>" — runtime resolves this lazily.
    let _ = write!(
        buf,
        r#" data-r-on:{ev}="{chunk}#{sym}""#,
        ev = h.event,
        chunk = h.chunk,
        sym = h.symbol,
    );

    if !h.captures.is_empty() {
        // Format: `name:s1,other:s5` — the runtime parses each pair to map
        // the Rust identifier to its stable signal id.
        let captures = h
            .captures
            .iter()
            .map(|c| format!("{}:{}", c.name, c.id))
            .collect::<Vec<_>>()
            .join(",");
        let _ = write!(buf, r#" data-r-cap:{ev}="{cap}""#, ev = h.event, cap = captures);
    }

    // Handler source lives only in the resumability JSON payload — not duplicated
    // in `data-r-inline:*` attributes. The runtime lazy-compiles on first interaction.
}

fn write_island(buf: &mut String, island: &Island) {
    let signals = island
        .signal_ids
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let props = serde_json::to_string(&island.props).unwrap_or_else(|_| "{}".into());
    let _ = write!(
        buf,
        r#"<resuma-island data-r-chunk="{chunk}" data-r-instance="{inst}" data-r-signals="{signals}" data-r-props="{props}">"#,
        chunk = island.chunk_id,
        inst = island.instance_id,
        signals = signals,
        props = escape_attr(&props),
    );
    write_view(buf, &island.view);
    let _ = write!(buf, "</resuma-island>");
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input"
            | "link" | "meta" | "source" | "track" | "wbr"
    )
}
