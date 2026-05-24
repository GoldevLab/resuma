//! TypeScript / JavaScript client components — prebuilt ESM bundles outside the resumability runtime.
//!
//! Use [`ClientComponent`] for heavy widgets (Three.js, charts, editors) that ship as separate
//! modules. Resumable Rust UI (`#[component]`, `onClick`, `computed!`) stays the default; client
//! components complement it rather than replacing it.
//!
//! ## Rust
//!
//! ```rust,ignore
//! use resuma::prelude::*;
//!
//! FlowApp::new()
//!     .client_asset("hero-particles", include_bytes!("../static/client/hero-particles.js"))
//!     .page("/", || view! { {client_component(ClientComponent::new("hero-particles").class("hero-particles"))} });
//! ```
//!
//! ## TypeScript
//!
//! Copy [`client-sdk/resuma-client.ts`](https://github.com/GolfredoPerezFernandez/resuma/blob/main/client-sdk/resuma-client.ts)
//! into your app and call [`bootClientComponent`](https://github.com/GolfredoPerezFernandez/resuma/blob/main/client-sdk/resuma-client.ts)
//! from each bundled entry.

use crate::View;

/// URL prefix for bundled client component scripts (`/static/client/{id}.js`).
pub const CLIENT_SCRIPT_PREFIX: &str = "/static/client/";

/// Declarative mount point for a prebuilt TypeScript/JavaScript client component.
#[derive(Debug, Clone)]
pub struct ClientComponent {
    pub id: String,
    pub class: Option<String>,
    pub props: Option<serde_json::Value>,
    pub aria_hidden: bool,
}

impl ClientComponent {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            class: None,
            props: None,
            aria_hidden: true,
        }
    }

    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    pub fn props(mut self, props: impl serde::Serialize) -> Self {
        self.props = serde_json::to_value(props).ok();
        self
    }

    pub fn aria_hidden(mut self, hidden: bool) -> Self {
        self.aria_hidden = hidden;
        self
    }

    pub fn script_url(&self) -> String {
        client_script_url(&self.id)
    }

    pub fn mount_id(&self) -> String {
        format!("r-client-{}", self.id)
    }
}

/// Script URL for a client component bundle.
pub fn client_script_url(id: &str) -> String {
    format!("{CLIENT_SCRIPT_PREFIX}{id}.js")
}

/// Mount point + deferred module script for a [`ClientComponent`].
pub fn client_component(comp: ClientComponent) -> View {
    if !valid_client_id(&comp.id) {
        return View::empty();
    }

    let mut attrs = format!(
        r#"data-r-client="{}" id="{}""#,
        escape_attr(&comp.id),
        escape_attr(&comp.mount_id()),
    );

    if let Some(class) = &comp.class {
        attrs.push_str(&format!(r#" class="{}""#, escape_attr(class)));
    }

    if comp.aria_hidden {
        attrs.push_str(r#" aria-hidden="true""#);
    }

    if let Some(props) = &comp.props {
        if !props.is_null() {
            let json = props.to_string();
            attrs.push_str(&format!(
                r#" data-r-client-props="{}""#,
                escape_attr(&json)
            ));
        }
    }

    let script = escape_attr(&comp.script_url());
    View::raw(format!(
        r#"<div {attrs}></div>
<script type="module" src="{script}" defer></script>"#
    ))
}

fn valid_client_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn escape_attr(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_component_emits_mount_and_script() {
        let html = match client_component(
            ClientComponent::new("hero-particles").class("hero-particles"),
        ) {
            View::Raw(s) => s,
            _ => panic!("expected raw view"),
        };
        assert!(html.contains(r#"data-r-client="hero-particles""#));
        assert!(html.contains(r#"id="r-client-hero-particles""#));
        assert!(html.contains(r#"class="hero-particles""#));
        assert!(html.contains(r#"src="/static/client/hero-particles.js""#));
    }

    #[test]
    fn client_script_url_format() {
        assert_eq!(client_script_url("chart"), "/static/client/chart.js");
    }

    #[test]
    fn rejects_invalid_client_id() {
        assert!(matches!(
            client_component(ClientComponent::new(r#"bad"id"#)),
            View::Empty
        ));
    }

    #[test]
    fn valid_client_id_accepts_common_names() {
        assert!(valid_client_id("hero-particles"));
        assert!(valid_client_id("chart_v2"));
        assert!(!valid_client_id(""));
        assert!(!valid_client_id("../escape"));
    }
}
