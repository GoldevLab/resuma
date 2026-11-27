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

/// AI search and assistant crawlers that get their own `robots.txt` group by default.
///
/// - `OAI-SearchBot`, `Claude-SearchBot`, `PerplexityBot`: build the search index.
/// - `ChatGPT-User`, `Claude-User`, `Perplexity-User`: fetch a page when a person asks.
pub const AI_SEARCH_AGENTS: &[&str] = &[
    "OAI-SearchBot",
    "ChatGPT-User",
    "Claude-SearchBot",
    "Claude-User",
    "PerplexityBot",
    "Perplexity-User",
];

/// AI training / data-use control tokens that get their own group by default.
///
/// `GPTBot` is emitted separately because it honors [`AiCrawlerPolicy::gptbot_allow`].
/// `Claude-Web` is the legacy Anthropic name, kept so older rules keep matching.
pub const AI_TRAINING_AGENTS: &[&str] = &[
    "Google-Extended",
    "ClaudeBot",
    "Claude-Web",
    "Applebot-Extended",
];

/// Policy for AI / LLM crawlers (GEO).
///
/// Every crawler group in the generated `robots.txt` repeats [`Self::disallow`]. A crawler
/// that has its own group ignores the `*` group, so a path blocked only under `*` would
/// otherwise stay open to it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AiCrawlerPolicy {
    /// Emit explicit AI crawler groups (GPTBot, search/assistant bots, training tokens).
    /// When false only the `*` group and the sitemap line are written.
    pub allow_gptbot: bool,
    /// Extra `Allow:` paths for GPTBot section in robots.txt.
    pub gptbot_allow: Vec<String>,
    /// Paths always disallowed for all bots.
    pub disallow: Vec<String>,
    /// More user agents to allow with the same rules as the built-in AI groups.
    pub extra_allow: Vec<String>,
    /// User agents to block entirely (`Disallow: /`). Wins over every allow list.
    pub block: Vec<String>,
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
    /// Full `robots.txt` body. When set, [`SeoKit::robots_txt`] returns it unchanged and the
    /// generated groups, `Sitemap:` line and AI policy are skipped.
    pub robots_override: Option<String>,
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

    pub fn with_llms_section(mut self, title: impl Into<String>, desc: impl Into<String>) -> Self {
        self.llms_sections.push((title.into(), desc.into()));
        self
    }

    pub fn with_theme_color(mut self, color: impl Into<String>) -> Self {
        self.theme_color = Some(color.into());
        self
    }

    /// Serve this exact text at `/robots.txt` instead of the generated one.
    ///
    /// You own the whole file, including the `Sitemap:` line. To keep the generated file and
    /// only tweak it, use [`with_disallow`](Self::with_disallow),
    /// [`with_ai_agent`](Self::with_ai_agent) or [`with_blocked_agent`](Self::with_blocked_agent).
    pub fn with_robots_txt(mut self, body: impl Into<String>) -> Self {
        self.robots_override = Some(body.into());
        self
    }

    /// Disallow `path` for every crawler group in the generated `robots.txt`.
    pub fn with_disallow(mut self, path: impl Into<String>) -> Self {
        self.ai.disallow.push(path.into());
        self
    }

    /// Allow one more AI user agent with the same rules as the built-in AI groups.
    pub fn with_ai_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.ai.extra_allow.push(user_agent.into());
        self
    }

    /// Block a user agent entirely (`Disallow: /`), for example `CCBot` or `Bytespider`.
    pub fn with_blocked_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.ai.block.push(user_agent.into());
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
        // Robots belong in `seo_head_tags` so pages can set `noindex` without
        // a conflicting `index, follow` tag from the kit.
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

    /// `robots.txt` body.
    ///
    /// Returns [`robots_override`](Self::with_robots_txt) as-is when set. Otherwise writes a
    /// `*` group, one group per AI crawler ([`AI_SEARCH_AGENTS`], [`AI_TRAINING_AGENTS`],
    /// `GPTBot` and [`AiCrawlerPolicy::extra_allow`]), a `Disallow: /` group for each
    /// [`AiCrawlerPolicy::block`] entry, and the sitemap line.
    ///
    /// Each crawler gets its own group on purpose: parsers differ on groups with several
    /// `User-agent` lines, and a crawler with its own group ignores `*`, so the shared
    /// [`AiCrawlerPolicy::disallow`] list is repeated in every group.
    pub fn robots_txt(&self) -> String {
        if let Some(custom) = &self.robots_override {
            let mut body = custom.clone();
            if !body.ends_with('\n') {
                body.push('\n');
            }
            return body;
        }

        let origin = &self.site_url;
        let disallow = robots_values(&self.ai.disallow);
        let blocked = robots_agents(&self.ai.block);
        let mut body = String::from("User-agent: *\nAllow: /\n");
        push_robots_disallow(&mut body, &disallow);

        // Lowercased names already written; blocked agents never get an allow group.
        let mut seen: std::collections::HashSet<String> =
            blocked.iter().map(|a| a.to_ascii_lowercase()).collect();

        if self.ai.allow_gptbot {
            if seen.insert("gptbot".into()) {
                body.push_str("\nUser-agent: GPTBot\n");
                let allow = robots_values(&self.ai.gptbot_allow);
                if allow.is_empty() {
                    body.push_str("Allow: /\n");
                } else {
                    for a in &allow {
                        body.push_str(&format!("Allow: {a}\n"));
                    }
                }
                push_robots_disallow(&mut body, &disallow);
            }
            let extra = robots_agents(&self.ai.extra_allow);
            let agents = AI_SEARCH_AGENTS
                .iter()
                .chain(AI_TRAINING_AGENTS.iter())
                .copied()
                .chain(extra.iter().map(String::as_str));
            for agent in agents {
                if !seen.insert(agent.to_ascii_lowercase()) {
                    continue;
                }
                body.push_str(&format!("\nUser-agent: {agent}\nAllow: /\n"));
                push_robots_disallow(&mut body, &disallow);
            }
        }

        for agent in &blocked {
            body.push_str(&format!("\nUser-agent: {agent}\nDisallow: /\n"));
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
        out.push_str(&format!(
            "\n## Optional\n\n- [Sitemap]({origin}/sitemap.xml)\n- [Robots]({origin}/robots.txt)\n",
            origin = self.site_url.trim_end_matches('/')
        ));
        out
    }
}

/// Trimmed values that are safe to write on one `robots.txt` line (no control characters).
fn robots_values(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty() && !v.chars().any(char::is_control))
        .map(str::to_string)
        .collect()
}

/// `User-agent` tokens: ASCII letters, digits, `-`, `_` and `.`, deduplicated ignoring case.
/// Anything else (spaces, `*`, `:`, newlines) is dropped so a value cannot add directives.
fn robots_agents(values: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    values
        .iter()
        .map(|v| v.trim())
        .filter(|v| {
            !v.is_empty()
                && v.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        })
        .filter(|v| seen.insert(v.to_ascii_lowercase()))
        .map(str::to_string)
        .collect()
}

fn push_robots_disallow(body: &mut String, paths: &[String]) {
    for path in paths {
        body.push_str(&format!("Disallow: {path}\n"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robots_includes_gptbot_when_enabled() {
        let kit = SeoKit::new("Test", "https://example.com");
        let txt = kit.robots_txt();
        assert!(txt.contains("User-agent: GPTBot\nAllow: /"));
        assert!(txt.contains("User-agent: OAI-SearchBot\nAllow: /"));
        assert!(txt.contains("User-agent: Google-Extended\nAllow: /"));
        assert!(txt.contains("Sitemap"));
    }

    #[test]
    fn robots_lists_claude_perplexity_and_apple_crawlers() {
        let txt = SeoKit::new("Test", "https://example.com").robots_txt();
        for agent in [
            "ClaudeBot",
            "Claude-SearchBot",
            "Claude-User",
            "Claude-Web",
            "PerplexityBot",
            "Perplexity-User",
            "Applebot-Extended",
        ] {
            assert!(
                txt.contains(&format!("User-agent: {agent}\nAllow: /")),
                "missing {agent}"
            );
        }
    }

    #[test]
    fn robots_repeats_disallow_in_every_crawler_group() {
        let kit = SeoKit::new("Test", "https://example.com")
            .with_disallow("/api/")
            .with_disallow("/admin");
        let txt = kit.robots_txt();
        let groups: Vec<&str> = txt
            .split("\n\n")
            .filter(|g| g.starts_with("User-agent:"))
            .collect();
        assert!(groups.len() > 5);
        for group in groups {
            assert!(
                group.contains("Disallow: /api/"),
                "group lacks /api/: {group}"
            );
            assert!(
                group.contains("Disallow: /admin"),
                "group lacks /admin: {group}"
            );
        }
    }

    #[test]
    fn robots_gptbot_allow_list_replaces_allow_root() {
        let mut kit = SeoKit::new("Test", "https://example.com");
        kit.ai.gptbot_allow = vec!["/blog/".into(), "/docs/".into()];
        kit.ai.disallow = vec!["/admin".into()];
        let txt = kit.robots_txt();
        assert!(
            txt.contains("User-agent: GPTBot\nAllow: /blog/\nAllow: /docs/\nDisallow: /admin\n")
        );
        assert!(!txt.contains("User-agent: GPTBot\nAllow: /\n"));
    }

    #[test]
    fn robots_blocked_agent_wins_over_allow_lists() {
        let kit = SeoKit::new("Test", "https://example.com")
            .with_ai_agent("CCBot")
            .with_blocked_agent("CCBot")
            .with_blocked_agent("ccbot")
            .with_blocked_agent("GPTBot");
        let txt = kit.robots_txt();
        assert!(txt.contains("User-agent: CCBot\nDisallow: /\n"));
        assert!(!txt.contains("User-agent: CCBot\nAllow: /"));
        assert_eq!(txt.matches("User-agent: CCBot").count(), 1);
        assert!(txt.contains("User-agent: GPTBot\nDisallow: /\n"));
        assert!(!txt.contains("User-agent: GPTBot\nAllow:"));
    }

    #[test]
    fn robots_extra_agent_gets_allow_group() {
        let kit = SeoKit::new("Test", "https://example.com")
            .with_ai_agent("MistralAI-User")
            .with_ai_agent("claudebot");
        let txt = kit.robots_txt();
        assert!(txt.contains("User-agent: MistralAI-User\nAllow: /\n"));
        // Already a built-in group: not written twice.
        assert_eq!(
            txt.to_ascii_lowercase()
                .matches("user-agent: claudebot")
                .count(),
            1
        );
    }

    #[test]
    fn robots_blocks_apply_even_without_ai_groups() {
        let mut kit = SeoKit::new("Test", "https://example.com").with_blocked_agent("Bytespider");
        kit.ai.allow_gptbot = false;
        let txt = kit.robots_txt();
        assert!(!txt.contains("GPTBot"));
        assert!(txt.contains("User-agent: Bytespider\nDisallow: /\n"));
        assert!(txt.contains("Sitemap: https://example.com/sitemap.xml"));
    }

    #[test]
    fn robots_rejects_values_that_could_inject_directives() {
        let kit = SeoKit::new("Test", "https://example.com")
            .with_ai_agent("Good-Bot\nAllow: /secret")
            .with_ai_agent("*")
            .with_ai_agent("two words")
            .with_blocked_agent("Evil\r\nDisallow:")
            .with_disallow("/ok\nAllow: /")
            .with_disallow("/fine");
        let txt = kit.robots_txt();
        assert!(!txt.contains("Good-Bot"));
        assert!(!txt.contains("two words"));
        assert!(!txt.contains("Evil"));
        assert!(!txt.contains("/secret"));
        assert!(!txt.contains("/ok"));
        assert!(txt.contains("Disallow: /fine"));
        // Only the generated `*` group may mention `Allow: /` on its own line before GPTBot.
        assert_eq!(txt.matches("User-agent: *").count(), 1);
    }

    #[test]
    fn robots_override_is_returned_unchanged() {
        let kit = SeoKit::new("Test", "https://example.com")
            .with_disallow("/ignored")
            .with_robots_txt("User-agent: *\nDisallow: /private");
        let txt = kit.robots_txt();
        assert_eq!(txt, "User-agent: *\nDisallow: /private\n");
    }

    #[test]
    fn robots_default_matches_previous_shape_for_search_and_sitemap() {
        let kit = SeoKit::new("Test", "https://example.com")
            .with_disallow("/api/")
            .with_llms_summary("Hi.");
        let txt = kit.robots_txt();
        assert!(txt.starts_with("User-agent: *\nAllow: /\nDisallow: /api/\n"));
        assert!(txt.contains("\nSitemap: https://example.com/sitemap.xml\n"));
        assert!(txt.ends_with("# LLM-readable summary: https://example.com/llms.txt\n"));
    }

    #[test]
    fn llms_txt_lists_sitemap() {
        let kit = SeoKit::new("Test", "https://example.com").with_llms_summary("Hello.");
        let txt = kit.llms_txt();
        assert!(txt.contains("https://example.com/sitemap.xml"));
        assert!(txt.contains("Hello."));
    }

    #[test]
    fn head_extras_does_not_emit_robots() {
        let kit = SeoKit::new("Test", "https://example.com");
        assert!(!kit.head_extras().contains(r#"name="robots""#));
    }

    #[test]
    fn meta_pixel_in_head() {
        let kit = SeoKit::new("Test", "https://example.com").with_meta_pixel("123456");
        assert!(kit.head_extras().contains("fbevents.js"));
        assert!(kit.head_extras().contains("123456"));
    }
}
