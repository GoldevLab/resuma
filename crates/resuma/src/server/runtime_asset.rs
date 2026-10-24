//! Embedded JS runtime bundles (loader + core + legacy monolith).

use once_cell::sync::Lazy;

pub const LOADER_JS: &str = include_str!("../../assets/loader.js");
pub const CORE_JS: &str = include_str!("../../assets/core.js");
pub const LIFECYCLE_JS: &str = include_str!("../../assets/lifecycle.js");
pub const FLOW_JS: &str = include_str!("../../assets/flow.js");
pub const FLOW_CSS: &str = include_str!("../../assets/flow.css");
pub const UI_JS: &str = include_str!("../../assets/ui.js");
pub const UI_CSS: &str = include_str!("../../assets/ui.css");
pub const RUNTIME_JS: &str = include_str!("../../assets/runtime.js");

/// Href for the Flow stylesheet with a content digest (`?v=…`) for cache busting.
pub fn flow_css_url() -> &'static str {
    static URL: Lazy<String> = Lazy::new(|| {
        let digest = crate::client::content_digest(FLOW_CSS.as_bytes());
        format!("/_resuma/flow.css?v={digest}")
    });
    URL.as_str()
}

/// Href for overlay stylesheet (Popup / Modal / GestureView / VirtualFor).
pub fn ui_css_url() -> &'static str {
    static URL: Lazy<String> = Lazy::new(|| {
        let digest = crate::client::content_digest(UI_CSS.as_bytes());
        format!("/_resuma/ui.css?v={digest}")
    });
    URL.as_str()
}
