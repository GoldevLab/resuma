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
    context::{page_needs_client, RenderContext, RenderMode, ResumePayload},
    handler::HandlerRef,
    serialize::encode_payload,
    view::{Attr, AttrValue, Child, Element, Fragment, Island, ShowView, View},
    with_context,
};

mod escape;
pub mod pwa;
pub mod seo;
pub mod seo_kit;
pub mod stream;
use escape::{escape_attr, escape_text};

pub use stream::{
    build_page_stream, render_stream_parts, render_to_stream, stream_head, stream_placeholder,
    stream_tail, StreamChunk,
};

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
    /// Optional SEO / analytics / GEO kit merged at render time.
    pub seo_kit: Option<seo_kit::SeoKit>,
}

/// Render a complete HTML document for an already-built view (merges payload handlers server-side).
pub fn render_document(opts: &PageOptions, path: &str, view: &View) -> (String, ResumePayload) {
    let (body, payload) = render_body_and_payload(view);
    (wrap_document(opts, &body, &payload, path), payload)
}

/// Render a view that was already built inside an external [`RenderContext`].
pub fn render_prebuilt_document(
    opts: &PageOptions,
    path: &str,
    view: &View,
    payload: &ResumePayload,
) -> String {
    let body = render_view(view);
    wrap_document(opts, &body, payload, path)
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

pub(crate) fn client_scripts(
    opts: &PageOptions,
    body_html: &str,
    payload: &ResumePayload,
) -> String {
    if !page_needs_client(payload, body_html) {
        return String::new();
    }
    let mut payload = payload.for_client();
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
<script type="module" src="{loader}"{nonce_attr}></script>"#,
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
    (body, ctx.snapshot_full())
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

pub(crate) fn apply_head_csp_nonce(head: &str, nonce: &str) -> String {
    if nonce.is_empty() {
        return head.to_string();
    }
    let nonce_attr = format!(r#" nonce="{}""#, escape_attr(nonce));
    inject_csp_nonce_into_head(head, &nonce_attr)
}

fn inject_csp_nonce_into_head(head: &str, nonce_attr: &str) -> String {
    let mut out = String::with_capacity(head.len() + 64);
    let mut rest = head;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        let Some(end) = rest.find('>') else {
            out.push_str(rest);
            break;
        };
        let tag = &rest[..=end];
        out.push_str(&inject_nonce_on_tag(tag, nonce_attr));
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    out
}

fn inject_nonce_on_tag(tag: &str, nonce_attr: &str) -> String {
    let lower = tag.to_ascii_lowercase();
    if !(lower.starts_with("<style") || lower.starts_with("<script")) {
        return tag.to_string();
    }
    if lower.contains("nonce=") {
        return tag.to_string();
    }
    if lower.starts_with("<script") && lower.contains("src=") {
        return tag.to_string();
    }
    if let Some(gt) = tag.rfind('>') {
        format!("{}{}{}", &tag[..gt], nonce_attr, &tag[gt..])
    } else {
        tag.to_string()
    }
}

fn wrap_document(
    opts: &PageOptions,
    body_html: &str,
    payload: &ResumePayload,
    path: &str,
) -> String {
    let lang = if opts.lang.is_empty() {
        "en"
    } else {
        &opts.lang
    };
    let title = seo::page_title(opts, path);
    let description = seo::page_description(opts, path);
    let seo_tags = seo::seo_head_tags(opts, path);
    let json_ld = seo::json_ld_script(&opts.json_ld);
    let stylesheet = opts
        .stylesheet
        .as_ref()
        .map(|s| format!(r#"<link rel="stylesheet" href="{}" />"#, escape_attr(s)))
        .unwrap_or_default();
    let scripts = client_scripts(opts, body_html, payload);
    let dev_script = crate::server::dev::dev_reload_script(&opts.csp_nonce);
    let head = apply_head_csp_nonce(&opts.head, &opts.csp_nonce);

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
<div id="resuma-root">{body}</div>
{scripts}
{dev_script}
</body>
</html>"#,
        lang = lang,
        title = escape_text(&title),
        description = escape_text(&description),
        seo_tags = seo_tags,
        json_ld = json_ld,
        head = head,
        stylesheet = stylesheet,
        body = body_html,
        scripts = scripts,
        dev_script = dev_script,
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
            let _ = write!(
                buf,
                r#"<resuma-dyn data-r-signal="{}">{}</resuma-dyn>"#,
                d.signal,
                escape_text(&formatted)
            );
        }
        View::Element(el) => write_element(buf, el),
        View::Fragment(Fragment { children }) => {
            for c in children {
                write_child(buf, c);
            }
        }
        View::Component(c) => write_view(buf, &c.view),
        View::Island(island) => write_island(buf, island),
        View::Boundary(boundary) => write_boundary(buf, boundary),
        View::Show(show) => write_show(buf, show),
        View::For(for_view) => write_for(buf, for_view),
        View::Match(m) => write_match(buf, m),
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
    for c in &el.children {
        write_child(buf, c);
    }
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
            let _ = write!(
                buf,
                r#" {}="" data-r-bind:{}="{}|{}""#,
                name,
                name,
                signal,
                escape_attr(f)
            );
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
        let _ = write!(
            buf,
            r#" data-r-cap:{ev}="{cap}""#,
            ev = h.event,
            cap = captures
        );
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
    let load = match island.load {
        crate::core::view::IslandLoad::Visible => "visible",
        crate::core::view::IslandLoad::Eager => "eager",
    };
    let mut inner = String::new();
    write_view(&mut inner, &island.view);
    crate::server::island_cache::cache_island_html(
        &island.instance_id,
        &inner,
        &island.chunk_id,
        load,
    );
    let _ = write!(
        buf,
        r#"<resuma-island data-r-chunk="{chunk}" data-r-instance="{inst}" data-r-signals="{signals}" data-r-props="{props}" data-r-load="{load}">"#,
        chunk = island.chunk_id,
        inst = island.instance_id,
        signals = signals,
        props = escape_attr(&props),
        load = load,
    );
    buf.push_str(&inner);
    let _ = write!(buf, "</resuma-island>");
}

fn write_show(buf: &mut String, show: &ShowView) {
    let _ = write!(
        buf,
        r#"<resuma-show data-r-show="{}" data-r-inverted="{}">"#,
        show.signal, show.inverted
    );
    let _ = write!(buf, r#"<div data-r-show-if"#);
    if !show.initial {
        buf.push_str(" hidden");
    }
    buf.push('>');
    for c in &show.children {
        write_child(buf, c);
    }
    buf.push_str("</div>");
    if let Some(fb) = &show.fallback {
        let _ = write!(buf, r#"<div data-r-show-else"#);
        if show.initial {
            buf.push_str(" hidden");
        }
        buf.push('>');
        write_view(buf, fb);
        buf.push_str("</div>");
    }
    buf.push_str("</resuma-show>");
}

fn write_for(buf: &mut String, for_view: &crate::core::view::ForView) {
    let key_attr = for_view
        .key_field
        .as_deref()
        .map(|k| format!(r#" data-r-key="{}""#, escape_attr(k)))
        .unwrap_or_default();
    let _ = write!(
        buf,
        r#"<resuma-for data-r-for="{}"{key_attr}>"#,
        for_view.signal,
        key_attr = key_attr,
    );
    buf.push_str(r#"<div data-r-for-list>"#);
    for item in &for_view.items {
        let _ = write!(
            buf,
            r#"<div data-r-for-item data-r-for-key="{}">"#,
            escape_attr(&item.key),
        );
        for c in &item.children {
            write_child(buf, c);
        }
        buf.push_str("</div>");
    }
    buf.push_str("</div></resuma-for>");
}

fn write_match(buf: &mut String, m: &crate::core::view::MatchView) {
    let _ = write!(
        buf,
        r#"<resuma-match data-r-match="{}" data-r-match-initial="{}">"#,
        m.signal,
        escape_attr(&m.initial),
    );
    for case in &m.cases {
        let hidden = if case.when == m.initial {
            ""
        } else {
            " hidden"
        };
        let _ = write!(
            buf,
            r#"<div data-r-match-case="{}" data-r-match-when="{}"{hidden}>"#,
            escape_attr(&case.when),
            escape_attr(&case.when),
            hidden = hidden,
        );
        for c in &case.children {
            write_child(buf, c);
        }
        buf.push_str("</div>");
    }
    if let Some(default) = &m.default {
        let hidden = if m.cases.iter().any(|c| c.when == m.initial) {
            " hidden"
        } else {
            ""
        };
        let _ = write!(
            buf,
            r#"<div data-r-match-default{hidden}>"#,
            hidden = hidden
        );
        for c in default {
            write_child(buf, c);
        }
        buf.push_str("</div>");
    }
    buf.push_str("</resuma-match>");
}

fn write_boundary(buf: &mut String, boundary: &crate::core::view::Boundary) {
    let _ = write!(
        buf,
        r#"<resuma-boundary data-r-chunk="{chunk}" hidden aria-hidden="true"></resuma-boundary>"#,
        chunk = escape_attr(&boundary.chunk_id),
    );
    write_view(buf, &boundary.view);
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "source"
            | "track"
            | "wbr"
    )
}

#[cfg(test)]
mod head_nonce_tests {
    use super::apply_head_csp_nonce;

    #[test]
    fn injects_nonce_on_inline_style_and_script() {
        let head = r#"<style>.x{color:red}</style><script>console.log(1)</script>"#;
        let out = apply_head_csp_nonce(head, "abc123");
        assert!(out.contains(r#"<style nonce="abc123">"#));
        assert!(out.contains(r#"<script nonce="abc123">"#));
    }

    #[test]
    fn skips_external_script_with_src() {
        let head = r#"<script type="module" src="/static/app.js"></script>"#;
        let out = apply_head_csp_nonce(head, "abc123");
        assert!(!out.contains("nonce="));
    }
}
