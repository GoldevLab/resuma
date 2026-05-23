//! Site-wide SEO defaults (JSON-LD, copy).

pub fn site_description() -> &'static str {
    "Resuma is a resumable Rust web framework with SSR, islands, and server actions. \
     Ship HTML plus a tiny loader — no hydration, no eager JS. Full-stack Flow included."
}

pub fn json_ld(site_url: &str) -> String {
    format!(
        r#"{{
  "@context": "https://schema.org",
  "@graph": [
    {{
      "@type": "WebSite",
      "@id": "{site_url}/#website",
      "url": "{site_url}/",
      "name": "Resuma",
      "description": "{description}",
      "inLanguage": "en"
    }},
    {{
      "@type": "SoftwareApplication",
      "@id": "{site_url}/#software",
      "name": "Resuma",
      "applicationCategory": "DeveloperApplication",
      "operatingSystem": "Cross-platform",
      "description": "{description}",
      "url": "{site_url}/",
      "offers": {{
        "@type": "Offer",
        "price": "0",
        "priceCurrency": "USD"
      }}
    }}
  ]
}}"#,
        site_url = site_url.trim_end_matches('/'),
        description = site_description().replace('"', "\\\""),
    )
}

pub fn site_url() -> String {
    std::env::var("SITE_URL").unwrap_or_else(|_| "https://resuma-docs.fly.dev".into())
}
