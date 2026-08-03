//! Shared Flow dashboard styles (include once per page via [`flow_styles`] or [`flow_styles_link`]).

use resuma::core::view::{Attr, AttrValue, Child, Element, View};

/// Flow widget stylesheet (served at `GET /_resuma/flow.css` and inlined by [`flow_styles`]).
///
/// Kept as a copy under `crates/resuma-flow/assets/` so the published crate does
/// not depend on a sibling-path `include_str!` into `crates/resuma/assets/`
/// (that path is absent from the crates.io tarball). Keep in sync with
/// `crates/resuma/assets/flow.css` when editing styles.
pub const FLOW_CSS: &str = include_str!("../../assets/flow.css");

/// Emit Flow widget CSS inline (place in layout or page head).
///
/// When CSP is enabled, the per-request nonce from [`resuma::server::page_csp_nonce`] is applied.
/// For HTML injected after the initial page load, prefer [`flow_styles_link`] plus the static
/// `/_resuma/flow.css` route so styles are not blocked by `style-src-elem`.
pub fn flow_styles() -> View {
    let mut attrs = vec![Attr {
        name: "data-r-flow-styles".into(),
        value: AttrValue::Static("true".into()),
    }];
    let nonce = resuma::server::page_csp_nonce();
    if !nonce.is_empty() {
        attrs.push(Attr {
            name: "nonce".into(),
            value: AttrValue::Static(nonce),
        });
    }
    View::Element(Element {
        tag: "style".into(),
        attrs,
        children: vec![Child::Text(FLOW_CSS.into())],
        dom_id: None,
    })
}

/// Link to the static Flow stylesheet (CSP-safe for dynamically injected panels).
pub fn flow_styles_link() -> View {
    View::Element(Element {
        tag: "link".into(),
        attrs: vec![
            Attr {
                name: "rel".into(),
                value: AttrValue::Static("stylesheet".into()),
            },
            Attr {
                name: "href".into(),
                value: AttrValue::Static(resuma::server::flow_css_url().into()),
            },
            Attr {
                name: "data-r-flow-styles".into(),
                value: AttrValue::Static("link".into()),
            },
        ],
        children: vec![],
        dom_id: None,
    })
}
