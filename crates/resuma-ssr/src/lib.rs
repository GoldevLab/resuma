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

use resuma_core::{
    context::{RenderContext, RenderMode, ResumePayload},
    handler::HandlerRef,
    serialize::encode_payload,
    view::{Attr, AttrValue, Child, Element, Fragment, Island, View},
    with_context,
};

mod escape;
use escape::{escape_attr, escape_text};

/// Configuration for full-page rendering.
#[derive(Debug, Clone, Default)]
pub struct PageOptions {
    pub title: String,
    pub head: String,
    pub lang: String,
    pub runtime_src: String,
    pub stylesheet: Option<String>,
}

/// Render a `View` produced by a component to a complete HTML document.
pub fn render_to_string<F>(opts: &PageOptions, build_view: F) -> String
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

    wrap_document(opts, &body, &payload)
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

fn wrap_document(opts: &PageOptions, body_html: &str, payload: &ResumePayload) -> String {
    let lang = if opts.lang.is_empty() { "en" } else { &opts.lang };
    let payload_json = encode_payload(payload);
    let stylesheet = opts
        .stylesheet
        .as_ref()
        .map(|s| format!(r#"<link rel="stylesheet" href="{}" />"#, s))
        .unwrap_or_default();

    let runtime = if opts.runtime_src.is_empty() {
        "/_resuma/runtime.js"
    } else {
        opts.runtime_src.as_str()
    };

    format!(
        r#"<!doctype html>
<html lang="{lang}">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>{title}</title>
{stylesheet}
{head}
</head>
<body>
<div id="resuma-root">{body}</div>
<script type="resuma/state" id="resuma-state">{payload}</script>
<script type="module" src="{runtime}"></script>
</body>
</html>"#,
        lang = lang,
        title = escape_text(&opts.title),
        head = opts.head,
        stylesheet = stylesheet,
        body = body_html,
        payload = payload_json,
        runtime = runtime,
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

    if let Some(inline) = &h.inline {
        let _ = write!(buf, r#" data-r-inline:{ev}="{js}""#, ev = h.event, js = escape_attr(inline));
    }
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
