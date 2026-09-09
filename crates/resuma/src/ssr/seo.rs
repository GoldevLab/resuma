//! SEO helpers — canonical URLs, Open Graph, Twitter cards.

use super::escape::escape_attr;
use super::PageOptions;

fn normalize_path(path: &str) -> String {
    if path.is_empty() || path == "/" {
        return "/".into();
    }
    path.to_string()
}

fn canonical_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    let normalized = normalize_path(path);
    format!("{base}{normalized}")
}

fn path_segment_title(path: &str) -> Option<String> {
    if path.is_empty() || path == "/" {
        return None;
    }
    let segment = path.trim_end_matches('/').rsplit('/').next()?;
    if segment.is_empty() {
        return None;
    }
    Some(
        segment
            .replace('_', " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    )
}

pub fn page_title(opts: &PageOptions, path: &str) -> String {
    if let Some(title) = crate::server::page_cache::page_title_override() {
        return title;
    }
    if let Some(segment) = path_segment_title(path) {
        format!("{segment} | {}", opts.title)
    } else {
        opts.title.clone()
    }
}

pub fn page_description(opts: &PageOptions, path: &str) -> String {
    if let Some(description) = crate::server::page_cache::page_description_override() {
        return description;
    }
    if !opts.description.is_empty() {
        return opts.description.clone();
    }
    if let Some(segment) = path_segment_title(path) {
        return format!("{segment} — {}", opts.title);
    }
    opts.title.clone()
}

fn robots_content() -> String {
    crate::server::page_cache::page_robots_override()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "index, follow, max-image-preview:large".into())
}

fn robots_noindex(robots: &str) -> bool {
    robots
        .split(',')
        .any(|part| part.trim().eq_ignore_ascii_case("noindex"))
}

fn og_image_type(url: &str) -> &'static str {
    let path = url.split('?').next().unwrap_or(url).to_ascii_lowercase();
    if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".webp") {
        "image/webp"
    } else if path.ends_with(".gif") {
        "image/gif"
    } else {
        "image/svg+xml"
    }
}

fn site_name(opts: &PageOptions) -> &str {
    opts.seo_kit
        .as_ref()
        .map(|k| k.site_name.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(opts.title.as_str())
}

/// JSON-LD `<script>` without a CSP nonce.
///
/// JSON-LD is structured data, not executable script. Emitting `nonce=` breaks
/// naive crawler regexes that expect `type="application/ld+json">` with no attrs.
pub fn json_ld_script(json_ld: &str, _nonce: &str) -> String {
    if json_ld.is_empty() {
        return String::new();
    }
    let safe = crate::core::serialize::sanitize_json_for_script(json_ld.trim());
    format!("\n<script type=\"application/ld+json\">\n{safe}\n</script>\n")
}

/// JSON-LD for this document (page override, then [`PageOptions::json_ld`]).
pub fn document_json_ld(opts: &PageOptions) -> String {
    let raw =
        crate::server::page_cache::page_json_ld_override().unwrap_or_else(|| opts.json_ld.clone());
    json_ld_script(&raw, "")
}

pub fn seo_head_tags(opts: &PageOptions, path: &str) -> String {
    let mut out = String::new();
    let title = page_title(opts, path);
    let description = page_description(opts, path);
    let robots = robots_content();
    let indexable = !robots_noindex(&robots);

    out.push_str(&format!(
        r#"<meta name="robots" content="{robots}" />"#,
        robots = escape_attr(&robots),
    ));

    if indexable && !opts.site_url.is_empty() {
        let base = opts.site_url.trim_end_matches('/');
        let canonical = crate::server::page_cache::page_canonical_override()
            .or_else(|| opts.canonical.clone())
            .unwrap_or_else(|| canonical_url(base, path));

        out.push_str(&format!(
            r#"<link rel="canonical" href="{canonical}" />"#,
            canonical = escape_attr(&canonical),
        ));

        if !opts.og_image.is_empty() {
            let og_image =
                if opts.og_image.starts_with("http://") || opts.og_image.starts_with("https://") {
                    opts.og_image.clone()
                } else {
                    format!("{base}{}", opts.og_image)
                };
            let og_type = if opts.og_type.is_empty() {
                "website"
            } else {
                &opts.og_type
            };
            let image_type = og_image_type(&og_image);
            // Social cards describe *this* app's image, never the framework.
            let og_image_alt = format!("{title} — {}", site_name(opts));

            out.push_str(&format!(
                r#"
<link rel="icon" href="/favicon.svg" type="image/svg+xml" />
<link rel="apple-touch-icon" href="/favicon.svg" />
<meta property="og:type" content="{og_type}" />
<meta property="og:site_name" content="{site}" />
<meta property="og:locale" content="en_US" />
<meta property="og:title" content="{title}" />
<meta property="og:description" content="{description}" />
<meta property="og:url" content="{canonical}" />
<meta property="og:image" content="{og_image}" />
<meta property="og:image:type" content="{image_type}" />
<meta property="og:image:width" content="1200" />
<meta property="og:image:height" content="630" />
<meta property="og:image:alt" content="{og_image_alt}" />
<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:title" content="{title}" />
<meta name="twitter:description" content="{description}" />
<meta name="twitter:image" content="{og_image}" />
<meta name="twitter:image:alt" content="{og_image_alt}" />"#,
                og_type = escape_attr(og_type),
                site = escape_attr(site_name(opts)),
                title = escape_attr(&title),
                description = escape_attr(&description),
                canonical = escape_attr(&canonical),
                og_image = escape_attr(&og_image),
                image_type = escape_attr(image_type),
                og_image_alt = escape_attr(&og_image_alt),
            ));
        }
    }

    if let Some(pwa) = &opts.pwa {
        out.push_str(&super::pwa::pwa_head_tags(pwa, &opts.csp_nonce));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::view::View;
    use crate::ssr::render_to_string_at_path;
    use crate::ssr::PageOptions;

    fn html_at(opts: PageOptions, path: &str) -> String {
        crate::server::page_cache::clear_request_staging();
        render_to_string_at_path(&opts, path, || View::text("ok"))
    }

    #[test]
    fn og_tags_use_property_not_name() {
        let html = html_at(
            PageOptions {
                title: "App".into(),
                site_url: "https://example.com".into(),
                og_image: "/og.png".into(),
                ..Default::default()
            },
            "/docs",
        );
        assert!(html.contains(r#"property="og:title""#));
        assert!(!html.contains(r#"name="og:title""#));
        assert!(!html.contains(r#"name="og:description""#));
        assert!(!html.contains(r#"name="og:image""#));
        assert!(html.contains(r#"content="image/png""#));
    }

    #[test]

    #[test]
    fn og_image_alt_describes_the_app_not_resuma() {
        let html = html_at(
            PageOptions {
                title: "YouTube transcript | YouTubeForge".into(),
                site_url: "https://forgeyt.com".into(),
                og_image: "/og.png".into(),
                seo_kit: Some(crate::SeoKit::new("YouTubeForge", "https://forgeyt.com")),
                ..Default::default()
            },
            "/",
        );
        assert!(!html.contains("resumable SSR web framework"), "{html}");
        assert!(
            html.contains(
                r#"property="og:image:alt" content="YouTube transcript | YouTubeForge — YouTubeForge""#
            ),
            "{html}"
        );
        assert!(
            html.contains(
                r#"name="twitter:image:alt" content="YouTube transcript | YouTubeForge — YouTubeForge""#
            ),
            "{html}"
        );
    }

    fn non_200_stages_noindex_robots() {
        crate::server::page_cache::clear_request_staging();
        crate::server::page_cache::stage_response_status(404);
        let html = render_to_string_at_path(
            &PageOptions {
                title: "App".into(),
                site_url: "https://example.com".into(),
                og_image: "/og.svg".into(),
                ..Default::default()
            },
            "/missing",
            || View::text("not found"),
        );
        assert!(html.contains(r#"name="robots" content="noindex""#));
        assert!(!html.contains(r#"rel="canonical""#));
        crate::server::page_cache::clear_request_staging();
    }

    #[test]
    fn json_ld_script_omits_csp_nonce() {
        let tag = json_ld_script(r#"{"@type":"WebSite"}"#, "abc123");
        assert!(
            !tag.contains("nonce="),
            "JSON-LD must stay nonce-free for crawlers: {tag}"
        );
        assert!(tag.contains(r#"type="application/ld+json""#));
        assert!(tag.contains(r#"{"@type":"WebSite"}"#));
    }
}
