//! Site-wide SEO defaults (JSON-LD, copy).

pub fn site_title() -> &'static str {
    "Resuma — Resumable SSR Web Framework for Rust | Docs"
}

pub fn site_description() -> &'static str {
    "Resumable Rust SSR framework. Every component is interactive by default — \
     ~884 B loader, lazy handler chunks, server actions, and Flow. Zero hydration."
}

pub fn json_ld(site_url: &str) -> String {
    let base = site_url.trim_end_matches('/');
    let description = site_description().replace('"', "\\\"");
    format!(
        r#"{{"@context":"https://schema.org","@graph":[{{"@type":"Organization","@id":"{base}/#organization","name":"Resuma","url":"{base}/","logo":"{base}/og.svg","sameAs":["https://github.com/GolfredoPerezFernandez/resuma","https://crates.io/crates/resuma","https://docs.rs/resuma"]}},{{"@type":"WebSite","@id":"{base}/#website","url":"{base}/","name":"Resuma Documentation","description":"{description}","inLanguage":"en","publisher":{{"@id":"{base}/#organization"}}}},{{"@type":"SoftwareApplication","@id":"{base}/#software","name":"Resuma","applicationCategory":"DeveloperApplication","applicationSubCategory":"Web Framework","operatingSystem":"Cross-platform","programmingLanguage":"Rust","softwareVersion":"0.3.1","description":"{description}","url":"{base}/","downloadUrl":"https://crates.io/crates/resuma","documentation":"https://docs.rs/resuma","offers":{{"@type":"Offer","price":"0","priceCurrency":"USD"}},"author":{{"@id":"{base}/#organization"}}}}]}}"#
    )
}

pub fn site_url() -> String {
    std::env::var("SITE_URL").unwrap_or_else(|_| "https://resuma-docs.fly.dev".into())
}
