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
    if let Some(segment) = path_segment_title(path) {
        format!("{segment} | {}", opts.title)
    } else {
        opts.title.clone()
    }
}

pub fn page_description(opts: &PageOptions, path: &str) -> String {
    if !opts.description.is_empty() {
        return opts.description.clone();
    }
    if let Some(segment) = path_segment_title(path) {
        return format!("{segment} — {}", opts.title);
    }
    opts.title.clone()
}

pub fn json_ld_script(json_ld: &str) -> String {
    if json_ld.is_empty() {
        return String::new();
    }
    let safe = crate::core::serialize::sanitize_json_for_script(json_ld.trim());
    format!("\n<script type=\"application/ld+json\">\n{safe}\n</script>\n",)
}

pub fn seo_head_tags(opts: &PageOptions, path: &str) -> String {
    let mut out = String::new();
    let title = page_title(opts, path);
    let description = page_description(opts, path);

    if !opts.site_url.is_empty() {
        let base = opts.site_url.trim_end_matches('/');
        let canonical = opts
            .canonical
            .clone()
            .unwrap_or_else(|| canonical_url(base, path));

        out.push_str(&format!(
            r#"<link rel="canonical" href="{canonical}" />"#,
            canonical = escape_attr(&canonical),
        ));

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
<meta property="og:image:type" content="image/svg+xml" />
<meta property="og:image:width" content="1200" />
<meta property="og:image:height" content="630" />
<meta property="og:image:alt" content="{og_image_alt}" />
<meta name="og:title" content="{title}" />
<meta name="og:description" content="{description}" />
<meta name="og:image" content="{og_image}" />
<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:title" content="{title}" />
<meta name="twitter:description" content="{description}" />
<meta name="twitter:image" content="{og_image}" />
<meta name="twitter:image:alt" content="{og_image_alt}" />"#,
            og_type = escape_attr(og_type),
            site = escape_attr(&opts.title),
            title = escape_attr(&title),
            description = escape_attr(&description),
            canonical = escape_attr(&canonical),
            og_image = escape_attr(&og_image),
            og_image_alt = escape_attr("Resuma — resumable SSR web framework for Rust"),
        ));
    }

    out.push_str(r#"<meta name="robots" content="index, follow" />"#);

    if let Some(pwa) = &opts.pwa {
        out.push_str(&super::pwa::pwa_head_tags(pwa));
    }

    out
}
