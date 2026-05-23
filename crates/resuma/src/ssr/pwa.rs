//! Progressive Web App `<head>` tags.

use super::escape::escape_attr;
use super::PwaOptions;

pub fn pwa_head_tags(pwa: &PwaOptions) -> String {
    if !pwa.enabled {
        return String::new();
    }

    format!(
        r#"
<link rel="manifest" href="/manifest.webmanifest" />
<meta name="theme-color" content="{theme}" />
<meta name="mobile-web-app-capable" content="yes" />
<meta name="apple-mobile-web-app-capable" content="yes" />
<meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
<meta name="apple-mobile-web-app-title" content="{short_name}" />
<meta name="application-name" content="{short_name}" />
<link rel="apple-touch-icon" href="/icons/apple-touch-icon.svg" sizes="180x180" />
<link rel="apple-touch-icon" href="/icons/icon-192.svg" sizes="192x192" />
<link rel="apple-touch-icon" href="/icons/icon-512.svg" sizes="512x512" />
<script src="/pwa-register.js" defer></script>"#,
        theme = escape_attr(&pwa.theme_color),
        short_name = escape_attr(&pwa.short_name),
    )
}
