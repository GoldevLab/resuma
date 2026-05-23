//! SEO helpers — canonical URLs, Open Graph, Twitter cards.

use crate::escape::escape_attr;
use crate::PageOptions;

fn normalize_path(path: &str) -> String {
    if path.is_empty() || path == "/" {
        return String::new();
    }
    path.to_string()
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

pub fn seo_head_tags(opts: &PageOptions, path: &str) -> String {
    let mut out = String::new();
    let title = page_title(opts, path);
    let description = page_description(opts, path);

    if !opts.site_url.is_empty() {
        let base = opts.site_url.trim_end_matches('/');
        let canonical = opts
            .canonical
            .clone()
            .unwrap_or_else(|| format!("{base}{}", normalize_path(path)));

        out.push_str(&format!(
            r#"<link rel="canonical" href="{canonical}" />"#,
            canonical = escape_attr(&canonical),
        ));

        let og_image = if opts.og_image.starts_with("http://") || opts.og_image.starts_with("https://")
        {
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
<meta property="og:type" content="{og_type}" />
<meta property="og:site_name" content="{site}" />
<meta property="og:title" content="{title}" />
<meta property="og:description" content="{description}" />
<meta property="og:url" content="{canonical}" />
<meta property="og:image" content="{og_image}" />
<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:title" content="{title}" />
<meta name="twitter:description" content="{description}" />
<meta name="twitter:image" content="{og_image}" />"#,
            og_type = escape_attr(og_type),
            site = escape_attr(&opts.title),
            title = escape_attr(&title),
            description = escape_attr(&description),
            canonical = escape_attr(&canonical),
            og_image = escape_attr(&og_image),
        ));
    }

    out.push_str(r#"<meta name="robots" content="index, follow" />"#);

    if !opts.json_ld.is_empty() {
        out.push_str(&format!(
            r#"<script type="application/ld+json">{json_ld}</script>"#,
            json_ld = opts.json_ld,
        ));
    }

    out
}
