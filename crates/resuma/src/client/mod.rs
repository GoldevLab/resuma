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
//! [`FlowApp::client_asset`](crate::flow::FlowApp::client_asset) registers a content digest so
//! [`client_script_url`] emits `?v=<hash>`. Browsers cache `/static/client/{id}.js` as immutable;
//! the query busts the cache when bytes change — no manual `v2`/`v3` renames.
//!
//! In `RESUMA_DEV` (or `RESUMA_CLIENT_DISK=1`), handlers re-read `{client_dir}/{id}.js` so
//! `npm run watch:client` refreshes without a cargo rebuild. After the module runs,
//! `ClientComponent` fires `resuma:client-ready` — use `__resuma.waitClient(id)` instead of polling.
//!
//! ## TypeScript
//!
//! Copy [`client-sdk/resuma-client.ts`](https://github.com/GoldevLab/resuma/blob/main/client-sdk/resuma-client.ts)
//! into your app and call [`bootClientComponent`](https://github.com/GoldevLab/resuma/blob/main/client-sdk/resuma-client.ts)
//! from each bundled entry.

mod disk;

pub use disk::{
    client_asset_body, client_asset_cache_control, client_dir, client_disk_mode, client_disk_rel,
    default_client_dir, read_client_disk_bytes, register_client_disk_file,
    resolve_client_disk_path, set_client_dir, validate_client_disk_file,
};

use std::collections::HashMap;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use sha2::{Digest, Sha256};

use crate::View;

/// URL prefix for bundled client component scripts (`/static/client/{id}.js`).
pub const CLIENT_SCRIPT_PREFIX: &str = "/static/client/";

static CLIENT_DIGESTS: Lazy<RwLock<HashMap<String, String>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Short SHA-256 hex digest (16 chars) for cache-busting query strings.
pub fn content_digest(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
    hex[..16].to_string()
}

/// Record the digest for `id` so [`client_script_url`] can append `?v=…`.
///
/// Called from [`FlowApp::client_asset`](crate::flow::FlowApp::client_asset). Safe to call again
/// with new bytes (rebuild / hot reload) — overwrites the previous digest.
pub fn register_client_asset_digest(id: &str, body: &[u8]) {
    let digest = content_digest(body);
    CLIENT_DIGESTS.write().insert(id.to_string(), digest);
}

/// Route path for a client bundle (no query) — used when mounting axum routes.
pub fn client_script_path(id: &str) -> String {
    format!("{CLIENT_SCRIPT_PREFIX}{id}.js")
}

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
        match serde_json::to_value(props) {
            Ok(v) => self.props = Some(v),
            Err(e) => {
                tracing::warn!(
                    client_id = %self.id,
                    error = %e,
                    "ClientComponent props failed to serialize — mount will have no data-r-client-props"
                );
            }
        }
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
///
/// When the asset was registered via [`register_client_asset_digest`], appends `?v=<digest>`
/// so `Cache-Control: immutable` stays correct across content changes.
///
/// Under [`client_disk_mode`], prefers a live digest of the on-disk file so watch rebuilds
/// bust the module URL without restarting the Rust server.
pub fn client_script_url(id: &str) -> String {
    let memory = CLIENT_DIGESTS.read().get(id).cloned();
    disk::client_script_url_disk_aware(id, memory.as_deref())
}

/// Mount point + deferred module that imports the bundle then fires `resuma:client-ready`.
///
/// Prefer `__resuma.waitClient(id)` (or `onClientReady`) over polling `window.*` globals —
/// the race with deferred scripts is what pokeLegens dogfood hit.
pub fn client_component(comp: ClientComponent) -> View {
    if let Err(()) = validate_client_id(&comp.id) {
        tracing::warn!(
            client_id = %comp.id,
            "invalid client component id — must be 1-64 alphanumeric, dash, or underscore"
        );
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
            attrs.push_str(&format!(r#" data-r-client-props="{}""#, escape_attr(&json)));
        }
    }

    let script_url = comp.script_url();
    let script_js = escape_js_str(&script_url);
    let nonce = crate::server::page_csp_nonce();
    let nonce_attr = if nonce.is_empty() {
        String::new()
    } else {
        format!(r#" nonce="{}""#, escape_attr(&nonce))
    };
    // Import only — `bootClientComponent` / `notifyClientReady` fire
    // `resuma:client-ready` after sync or async init, not at import time.
    View::raw(format!(
        r#"<div {attrs}></div>
<script type="module" defer{nonce_attr}>
import "{script_js}";
</script>"#
    ))
}

fn escape_js_str(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            other => out.push(other),
        }
    }
    out
}

/// Validate a client component bundle id (`/static/client/{id}.js`).
#[allow(clippy::result_unit_err)]
pub fn validate_client_id(id: &str) -> Result<(), ()> {
    if id.is_empty()
        || id.len() > 64
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(());
    }
    Ok(())
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
        assert!(html.contains(r#"import "/static/client/hero-particles.js""#));
        assert!(
            !html.contains("resuma:client-ready"),
            "ready is signaled by bootClientComponent after init, not the import wrapper"
        );
    }

    #[test]
    fn client_script_url_format() {
        assert_eq!(
            client_script_url("chart-unregistered"),
            "/static/client/chart-unregistered.js"
        );
        assert_eq!(
            client_script_path("chart-unregistered"),
            "/static/client/chart-unregistered.js"
        );
    }

    #[test]
    fn client_script_url_includes_digest_after_register() {
        let body = b"export default {};";
        register_client_asset_digest("digest-demo", body);
        let url = client_script_url("digest-demo");
        let expected = format!("/static/client/digest-demo.js?v={}", content_digest(body));
        assert_eq!(url, expected);

        let html = match client_component(ClientComponent::new("digest-demo")) {
            View::Raw(s) => s,
            _ => panic!("expected raw view"),
        };
        assert!(html.contains(&format!(r#"import "{expected}""#)));
    }

    #[test]
    fn digest_changes_when_bytes_change() {
        register_client_asset_digest("digest-flip", b"a");
        let a = client_script_url("digest-flip");
        register_client_asset_digest("digest-flip", b"b");
        let b = client_script_url("digest-flip");
        assert_ne!(a, b);
        assert!(a.starts_with("/static/client/digest-flip.js?v="));
        assert!(b.starts_with("/static/client/digest-flip.js?v="));
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
        assert!(validate_client_id("hero-particles").is_ok());
        assert!(validate_client_id("chart_v2").is_ok());
        assert!(validate_client_id("").is_err());
        assert!(validate_client_id("../escape").is_err());
    }
}
