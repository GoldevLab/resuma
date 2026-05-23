//! PWA configuration for the docs site.

use resuma::FlowPwaConfig;

pub fn config() -> FlowPwaConfig {
    FlowPwaConfig {
        name: "Resuma — SSR + Resumability for Rust".into(),
        short_name: "Resuma".into(),
        description: super::seo::site_description().into(),
        theme_color: "#712cf9".into(),
        background_color: "#0f0a1a".into(),
        start_url: "/".into(),
        scope: "/".into(),
        cache_version: "1".into(),
        display: "standalone".into(),
        orientation: "any".into(),
    }
}
