//! SEO, GEO (Generative Engine Optimization), and analytics helpers.
//!
//! Inspired by production patterns from apps like ACUPATAS: rich meta tags,
//! JSON-LD, Meta Pixel + SPA route tracking, `robots.txt` / `llms.txt` for AI crawlers.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::escape::escape_attr;
use super::PageOptions;

/// Extra `<meta>` / `<link>` tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaTag {
    pub name: Option<String>,
    pub property: Option<String>,
    pub content: String,
    pub http_equiv: Option<String>,
}

impl MetaTag {
    pub fn name(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            property: None,
            content: content.into(),
            http_equiv: None,
        }
    }

    pub fn property(property: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            name: None,
            property: Some(property.into()),
            content: content.into(),
            http_equiv: None,
        }
    }

    fn render(&self) -> String {
        if let Some(n) = &self.name {
            return format!(
                r#"<meta name="{}" content="{}" />"#,
                escape_attr(n),
                escape_attr(&self.content)
            );
        }
        if let Some(p) = &self.property {
            return format!(
                r#"<meta property="{}" content="{}" />"#,
                escape_attr(p),
                escape_attr(&self.content)
            );
        }
        if let Some(h) = &self.http_equiv {
            return format!(
                r#"<meta http-equiv="{}" content="{}" />"#,
                escape_attr(h),
                escape_attr(&self.content)
            );
        }
        String::new()
    }
}

/// Policy for AI / LLM crawlers (GEO).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AiCrawlerPolicy {
    /// Allow GPTBot on public paths (default: allow marketing + blog).
    pub allow_gptbot: bool,
    /// Extra `Allow:` paths for GPTBot section in robots.txt.
    pub gptbot_allow: Vec<String>,
    /// Paths always disallowed for all bots.
    pub disallow: Vec<String>,
}

/// Marketing SEO kit — merge into [`PageOptions`] via [`SeoKit::apply`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeoKit {
    pub site_name: String,
    pub site_url: String,
    pub locale: String,
    pub author: String,
    pub keywords: String,
    /// Meta (Facebook) Pixel ID — injects fbevents.js + PageView.
    pub meta_pixel_id: Option<String>,
    /// Google Tag Manager container ID (optional).
    pub gtm_id: Option<String>,
    pub twitter_site: Option<String>,
    pub theme_color: Option<String>,
    pub extra_meta: Vec<MetaTag>,
    pub json_ld_blocks: Vec<Value>,
    pub ai: AiCrawlerPolicy,
    /// Human-readable summary for /llms.txt (GEO — helps ChatGPT, Perplexity, etc.).
    pub llms_summary: String,
    pub llms_sections: Vec<(String, String)>,
}

impl SeoKit {
    pub fn new(site_name: impl Into<String>, site_url: impl Into<String>) -> Self {
        Self {
            site_name: site_name.into(),
            site_url: site_url.into().trim_end_matches('/').to_string(),
            locale: "en_US".into(),
            ai: AiCrawlerPolicy {
                allow_gptbot: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn with_locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = locale.into();
        self
    }

    pub fn with_keywords(mut self, keywords: impl Into<String>) -> Self {
        self.keywords = keywords.into();
        self
    }

    pub fn with_meta_pixel(mut self, pixel_id: impl Into<String>) -> Self {
        self.meta_pixel_id = Some(pixel_id.into());
        self
    }

    pub fn with_gtm(mut self, gtm_id: impl Into<String>) -> Self {
        self.gtm_id = Some(gtm_id.into());
        self
    }

    pub fn with_llms_summary(mut self, summary: impl Into<String>) -> Self {
        self.llms_summary = summary.into();
        self
    }

    pub fn push_json_ld(mut self, block: Value) -> Self {
        self.json_ld_blocks.push(block);
        self
    }

    /// Standard Organization + WebSite JSON-LD (recommended for all apps).
    pub fn with_default_json_ld(mut self) -> Self {
        let origin = &self.site_url;
        let name = &self.site_name;
        self.json_ld_blocks.push(json!({
            "@context": "https://schema.org",
            "@type": "Organization",
            "name": name,
            "url": origin,
        }));
        self.json_ld_blocks.push(json!({
            "@context": "https://schema.org",
            "@type": "WebSite",
            "name": name,
            "url": origin,
        }));
        self
    }

    /// WebPage JSON-LD for a specific route.
    pub fn webpage_json_ld(title: &str, description: &str, url: &str) -> Value {
        json!({
            "@context": "https://schema.org",
            "@type": "WebPage",
            "name": title,
            "description": description,
            "url": url,
        })
    }

    /// Merge kit defaults into page options (title/description/json_ld/head extras).
    pub fn apply(&self, opts: &mut PageOptions) {
        if opts.site_url.is_empty() {
            opts.site_url = self.site_url.clone();
        }
        if !self.locale.is_empty() && opts.lang.is_empty() {
            opts.lang = self.locale.split('_').next().unwrap_or("en").to_string();
        }
        if opts.json_ld.is_empty() && !self.json_ld_blocks.is_empty() {
            opts.json_ld =
                serde_json::to_string(&self.json_ld_blocks).unwrap_or_else(|_| "[]".into());
        }
        let extras = self.head_extras();
        if !extras.is_empty() {
            opts.head = format!("{}{}", opts.head, extras);
        }
    }

    /// Analytics + extra meta tags for `<head>`.
    pub fn head_extras(&self) -> String {
        let mut out = String::new();

        if !self.keywords.is_empty() {
            out.push_str(&MetaTag::name("keywords", &self.keywords).render());
        }
        if !self.author.is_empty() {
            out.push_str(&MetaTag::name("author", &self.author).render());
        }
        out.push_str(
            &MetaTag::name(
                "robots",
                "index, follow, max-image-preview:large, max-snippet:-1",
            )
            .render(),
        );
        out.push_str(
            &MetaTag::name(
                "format-detection",
                "telephone=no, date=no, address=no, email=no",
            )
            .render(),
        );

        if let Some(color) = &self.theme_color {
            out.push_str(&MetaTag::name("theme-color", color).render());
        }
        if let Some(tw) = &self.twitter_site {
            out.push_str(&MetaTag::name("twitter:site", tw).render());
        }

        for tag in &self.extra_meta {
            out.push_str(&tag.render());
        }

        if let Some(gtm) = &self.gtm_id {
            let id = escape_attr(gtm);
            out.push_str(&format!(
                r#"<link rel="preconnect" href="https://www.googletagmanager.com" />
<script>(function(w,d,s,l,i){{w[l]=w[l]||[];w[l].push({{'gtm.start':new Date().getTime(),event:'gtm.js'}});var f=d.getElementsByTagName(s)[0],j=d.createElement(s),dl=l!='dataLayer'?'&l='+l:'';j.async=true;j.src='https://www.googletagmanager.com/gtm.js?id='+i+dl;f.parentNode.insertBefore(j,f);}})(window,document,'script','dataLayer','{id}');</script>"#
            ));
        }

        if let Some(pixel) = &self.meta_pixel_id {
            let id = escape_attr(pixel);
            out.push_str(&format!(
                r#"<link rel="preconnect" href="https://connect.facebook.net" crossorigin="anonymous" />
<script>
!function(f,b,e,v,n,t,s){{if(f.fbq)return;n=f.fbq=function(){{n.callMethod?n.callMethod.apply(n,arguments):n.queue.push(arguments)}};if(!f._fbq)f._fbq=n;n.push=n;n.loaded=!0;n.version='2.0';n.queue=[];t=b.createElement(e);t.async=!0;t.src=v;s=b.getElementsByTagName(e)[0];s.parentNode.insertBefore(t,s)}}(window,document,'script','https://connect.facebook.net/en_US/fbevents.js');
fbq('init','{id}');fbq('track','PageView');
</script>
<noscript><img height="1" width="1" style="display:none" alt="" src="https://www.facebook.com/tr?id={id}&ev=PageView&noscript=1" /></noscript>
<script>
(function(){{
  var first=true;
  document.addEventListener('resuma:navigate',function(){{
    if(typeof fbq!=='function')return;
    if(first){{first=false;return;}}
    fbq('track','PageView');
  }});
}})();
</script>"#
            ));
        }

        // GEO: hint LLM crawlers where machine-readable docs live.
        if !self.llms_summary.is_empty() {
            out.push_str(
                r#"<link rel="alternate" type="text/plain" href="/llms.txt" title="LLM site summary" />"#,
            );
        }

        out
    }

    /// `robots.txt` body with AI crawler sections (GPTBot, Claude, etc.).
    pub fn robots_txt(&self) -> String {
        let origin = &self.site_url;
        let mut body = String::from("User-agent: *\nAllow: /\n");
        for d in &self.ai.disallow {
            body.push_str(&format!("Disallow: {d}\n"));
        }
        if self.ai.allow_gptbot {
            body.push_str("\nUser-agent: GPTBot\n");
            for a in &self.ai.gptbot_allow {
                body.push_str(&format!("Allow: {a}\n"));
            }
            for d in &self.ai.disallow {
                body.push_str(&format!("Disallow: {d}\n"));
            }
            body.push_str("\nUser-agent: ChatGPT-User\nAllow: /\n");
            body.push_str("\nUser-agent: Claude-Web\nAllow: /\n");
        }
        body.push_str(&format!("\nSitemap: {origin}/sitemap.xml\n"));
        if !self.llms_summary.is_empty() {
            body.push_str(&format!("# LLM-readable summary: {origin}/llms.txt\n"));
        }
        body
    }

    /// `/llms.txt` — plain-text site summary for AI systems (GEO best practice).
    pub fn llms_txt(&self) -> String {
        let mut out = format!("# {}\n\n", self.site_name);
        if !self.llms_summary.is_empty() {
            out.push_str(&self.llms_summary);
            out.push('\n');
        }
        if !self.llms_sections.is_empty() {
            out.push_str("\n## Sections\n\n");
            for (title, desc) in &self.llms_sections {
                out.push_str(&format!("- **{title}**: {desc}\n"));
            }
        }
        out.push_str(&format!("\n## Canonical origin\n\n{}\n", self.site_url));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robots_includes_gptbot_when_enabled() {
        let kit = SeoKit::new("Test", "https://example.com");
        let txt = kit.robots_txt();
        assert!(txt.contains("GPTBot"));
        assert!(txt.contains("llms.txt") || txt.contains("Sitemap"));
    }

    #[test]
    fn meta_pixel_in_head() {
        let kit = SeoKit::new("Test", "https://example.com").with_meta_pixel("123456");
        assert!(kit.head_extras().contains("fbevents.js"));
        assert!(kit.head_extras().contains("123456"));
    }
}
