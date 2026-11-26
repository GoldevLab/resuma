//! Internationalization for Flow apps: locale negotiation, message catalogs,
//! localized URLs, and `hreflang` alternates.
//!
//! Locales live in the URL prefix (`/es/fuentes`, `/en/fuentes`), which keeps
//! each language crawlable on its own URL. Resolution order per request:
//!
//! 1. First path segment (`/en/...`) when it is a supported locale.
//! 2. `?lang=xx` query parameter.
//! 3. `Accept-Language` header (q-values respected, `pt-BR` falls back to `pt`).
//! 4. The default locale.
//!
//! ```rust,ignore
//! use resuma::prelude::*;
//!
//! static ES: &[(&str, &str)] = &[("home.title", "Noticias crypto")];
//! static EN: &[(&str, &str)] = &[("home.title", "Crypto news")];
//!
//! fn i18n() -> I18n {
//!     I18n::new("es").locale("es", ES).locale("en", EN)
//! }
//!
//! pub fn page(req: FlowRequest) -> View {
//!     let scope = i18n().scope(&req);
//!     scope.apply_page_meta("https://example.com"); // <html lang> + hreflang
//!     view! { <h1>{scope.t("home.title")}</h1> }
//! }
//! ```
//!
//! Every locale must serve the page at [`LocaleScope::path`]; `hreflang`
//! alternates point at those URLs, so a missing translation route is a
//! broken alternate.

use std::collections::HashMap;

use crate::flow::FlowRequest;
use crate::server::{set_page_head_extra, set_page_lang};

/// A static message catalog: `(key, text)` pairs.
pub type Messages = &'static [(&'static str, &'static str)];

/// Locale registry and message catalogs for a Flow app.
#[derive(Debug, Clone)]
pub struct I18n {
    default: &'static str,
    locales: Vec<&'static str>,
    catalogs: HashMap<&'static str, HashMap<&'static str, &'static str>>,
}

impl I18n {
    /// Create a registry whose fallback locale is `default` (for example `"es"`).
    ///
    /// The default locale must also be registered with [`I18n::locale`].
    pub fn new(default: &'static str) -> Self {
        Self {
            default,
            locales: Vec::new(),
            catalogs: HashMap::new(),
        }
    }

    /// Register a supported locale and its messages. Codes are lowercase BCP 47 tags.
    pub fn locale(mut self, code: &'static str, messages: Messages) -> Self {
        debug_assert!(
            !code.is_empty()
                && code
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
            "locale code must be lowercase ascii, got {code:?}"
        );
        if !self.catalogs.contains_key(code) {
            self.locales.push(code);
        }
        self.catalogs
            .insert(code, messages.iter().copied().collect());
        self
    }

    /// Fallback locale used when nothing else matches.
    pub fn default_locale(&self) -> &'static str {
        self.default
    }

    /// Supported locale codes, in registration order.
    pub fn locales(&self) -> &[&'static str] {
        &self.locales
    }

    /// Whether `code` is a registered locale (ASCII case-insensitive).
    pub fn is_supported(&self, code: &str) -> bool {
        self.match_code(code).is_some()
    }

    /// Resolve the locale for `req` (path prefix, `?lang=`, `Accept-Language`, default).
    pub fn locale_for(&self, req: &FlowRequest) -> &'static str {
        if let (Some(loc), _) = self.strip_locale(&req.path) {
            return loc;
        }
        if let Some(loc) = req.query_param("lang").and_then(|c| self.match_code(c)) {
            return loc;
        }
        if let Some(loc) = req
            .header("accept-language")
            .and_then(|h| self.match_accept_language(h))
        {
            return loc;
        }
        self.default
    }

    /// Split a leading locale segment off `path`.
    ///
    /// Returns the matched locale and the remainder (`"/es/fuentes"` → `(Some("es"), "/fuentes")`,
    /// `"/es"` → `(Some("es"), "/")`). Unsupported prefixes return `(None, path)`.
    pub fn strip_locale<'a>(&self, path: &'a str) -> (Option<&'static str>, &'a str) {
        let Some(trimmed) = path.strip_prefix('/') else {
            return (None, path);
        };
        let (first, rest) = match trimmed.find('/') {
            Some(i) => (&trimmed[..i], &trimmed[i..]),
            None => (trimmed, "/"),
        };
        match self.match_code(first) {
            Some(loc) => (Some(loc), rest),
            None => (None, path),
        }
    }

    /// Localized URL path for `path` under `locale`, e.g. `("en", "/fuentes/")` → `"/en/fuentes"`.
    ///
    /// Unknown locales fall back to the default locale. The root becomes `"/en"`.
    pub fn localized_path(&self, locale: &str, path: &str) -> String {
        let loc = self.match_code(locale).unwrap_or(self.default);
        let trimmed = path.trim_end_matches('/');
        let trimmed = if trimmed.is_empty() {
            String::new()
        } else if trimmed.starts_with('/') {
            trimmed.to_string()
        } else {
            format!("/{trimmed}")
        };
        format!("/{loc}{trimmed}")
    }

    /// Translated text for `key` in `locale`, falling back to the default locale, then to `key`.
    pub fn t(&self, locale: &str, key: &str) -> String {
        self.lookup(locale, key)
            .map(str::to_string)
            .unwrap_or_else(|| key.to_string())
    }

    /// Like [`I18n::t`], then replaces `{name}` placeholders with the given values.
    pub fn t_args(&self, locale: &str, key: &str, args: &[(&str, &str)]) -> String {
        let mut out = self.t(locale, key);
        for (name, value) in args {
            out = out.replace(&format!("{{{name}}}"), value);
        }
        out
    }

    /// `<link rel="alternate" hreflang>` tags for every locale plus `x-default`.
    ///
    /// `site_url` is the public origin without a trailing slash (for example `https://example.com`).
    /// `path` is the page path without a locale prefix.
    pub fn alternates_html(&self, site_url: &str, path: &str) -> String {
        let base = site_url.trim_end_matches('/');
        let mut out = String::new();
        for code in &self.locales {
            let href = attr_escape(&format!("{base}{}", self.localized_path(code, path)));
            out.push_str(&format!(
                r#"<link rel="alternate" hreflang="{code}" href="{href}">"#
            ));
        }
        let href = attr_escape(&format!(
            "{base}{}",
            self.localized_path(self.default, path)
        ));
        out.push_str(&format!(
            r#"<link rel="alternate" hreflang="x-default" href="{href}">"#
        ));
        out
    }

    /// Resolve the request once and return a scope for translations and page metadata.
    pub fn scope(&self, req: &FlowRequest) -> LocaleScope<'_> {
        let locale = self.locale_for(req);
        let page_path = self.strip_locale(&req.path).1.to_string();
        LocaleScope {
            i18n: self,
            locale,
            page_path,
        }
    }

    fn lookup(&self, locale: &str, key: &str) -> Option<&'static str> {
        self.catalogs
            .get(locale)
            .and_then(|c| c.get(key).copied())
            .or_else(|| {
                self.catalogs
                    .get(self.default)
                    .and_then(|c| c.get(key).copied())
            })
    }

    fn match_code(&self, code: &str) -> Option<&'static str> {
        self.locales
            .iter()
            .copied()
            .find(|loc| loc.eq_ignore_ascii_case(code))
    }

    fn match_accept_language(&self, header: &str) -> Option<&'static str> {
        let mut prefs: Vec<(f32, &str)> = header
            .split(',')
            .filter_map(|part| {
                let mut fields = part.split(';');
                let tag = fields.next()?.trim();
                if tag.is_empty() || tag == "*" {
                    return None;
                }
                let q = fields
                    .find_map(|p| p.trim().strip_prefix("q="))
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(1.0);
                (q > 0.0).then_some((q, tag))
            })
            .collect();
        // Stable sort keeps header order for equal q-values.
        prefs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        prefs.into_iter().find_map(|(_, tag)| {
            self.match_code(tag).or_else(|| {
                let primary = tag.split('-').next().unwrap_or(tag);
                self.match_code(primary)
            })
        })
    }
}

/// Locale resolved for one request. Created with [`I18n::scope`].
#[derive(Debug, Clone)]
pub struct LocaleScope<'a> {
    i18n: &'a I18n,
    locale: &'static str,
    page_path: String,
}

impl LocaleScope<'_> {
    /// The resolved locale code (for example `"en"`).
    pub fn locale(&self) -> &'static str {
        self.locale
    }

    /// Translated text for `key` in the resolved locale.
    pub fn t(&self, key: &str) -> String {
        self.i18n.t(self.locale, key)
    }

    /// Translated text with `{name}` placeholders replaced.
    pub fn t_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        self.i18n.t_args(self.locale, key, args)
    }

    /// Localized link to `path` in the resolved locale (`"/en/fuentes"`).
    pub fn path(&self, path: &str) -> String {
        self.i18n.localized_path(self.locale, path)
    }

    /// Set `<html lang>` and `hreflang` alternates for this response.
    ///
    /// `site_url` is the public origin without a trailing slash. Call once per page render.
    pub fn apply_page_meta(&self, site_url: &str) {
        set_page_lang(self.locale);
        set_page_head_extra(self.i18n.alternates_html(site_url, &self.page_path));
    }
}

fn attr_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    static ES: Messages = &[
        ("home.title", "Noticias crypto"),
        ("home.count", "{n} artículos"),
        ("only.es", "Solo en español"),
    ];
    static EN: Messages = &[
        ("home.title", "Crypto news"),
        ("home.count", "{n} articles"),
    ];

    fn i18n() -> I18n {
        I18n::new("es").locale("es", ES).locale("en", EN)
    }

    fn req(path: &str, query: &[(&str, &str)], accept: Option<&str>) -> FlowRequest {
        let mut headers = BTreeMap::new();
        if let Some(a) = accept {
            headers.insert("accept-language".to_string(), a.to_string());
        }
        FlowRequest {
            method: "GET".into(),
            path: path.into(),
            params: BTreeMap::new(),
            query: query
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers,
            extensions: BTreeMap::new(),
            cache_control: BTreeMap::new(),
        }
    }

    #[test]
    fn strip_locale_splits_prefix() {
        let i = i18n();
        assert_eq!(i.strip_locale("/es/fuentes"), (Some("es"), "/fuentes"));
        assert_eq!(i.strip_locale("/en"), (Some("en"), "/"));
        assert_eq!(i.strip_locale("/esx"), (None, "/esx"));
        assert_eq!(i.strip_locale("/"), (None, "/"));
    }

    #[test]
    fn localized_path_normalizes_slashes() {
        let i = i18n();
        assert_eq!(i.localized_path("en", "/"), "/en");
        assert_eq!(i.localized_path("en", "/fuentes/"), "/en/fuentes");
        assert_eq!(i.localized_path("en", "fuentes"), "/en/fuentes");
        assert_eq!(i.localized_path("xx", "/sobre"), "/es/sobre");
    }

    #[test]
    fn resolves_path_then_query_then_accept_language() {
        let i = i18n();
        assert_eq!(i.locale_for(&req("/en/fuentes", &[], None)), "en");
        assert_eq!(
            i.locale_for(&req("/fuentes", &[("lang", "en")], None)),
            "en"
        );
        assert_eq!(
            i.locale_for(&req("/fuentes", &[], Some("en-US,en;q=0.9,es;q=0.8"))),
            "en"
        );
        assert_eq!(i.locale_for(&req("/", &[], Some("pt-BR,pt;q=0.9"))), "es");
        assert_eq!(i.locale_for(&req("/", &[], None)), "es");
    }

    #[test]
    fn accept_language_respects_q_values() {
        let i = I18n::new("es").locale("es", ES).locale("en", EN);
        assert_eq!(
            i.locale_for(&req("/", &[], Some("en;q=0.2,es;q=0.9"))),
            "es"
        );
        assert_eq!(i.locale_for(&req("/", &[], Some("en;q=0"))), "es");
    }

    #[test]
    fn translation_falls_back_to_default_then_key() {
        let i = i18n();
        assert_eq!(i.t("en", "only.es"), "Solo en español");
        assert_eq!(i.t("en", "home.title"), "Crypto news");
        assert_eq!(i.t("en", "missing.key"), "missing.key");
    }

    #[test]
    fn t_args_replaces_placeholders() {
        let i = i18n();
        assert_eq!(i.t_args("en", "home.count", &[("n", "12")]), "12 articles");
        assert_eq!(i.t_args("es", "home.count", &[("n", "3")]), "3 artículos");
    }

    #[test]
    fn alternates_include_every_locale_and_x_default() {
        let i = i18n();
        let html = i.alternates_html("https://example.com/", "/fuentes");
        assert!(html.contains(r#"hreflang="es" href="https://example.com/es/fuentes""#));
        assert!(html.contains(r#"hreflang="en" href="https://example.com/en/fuentes""#));
        assert!(html.contains(r#"hreflang="x-default" href="https://example.com/es/fuentes""#));
    }

    #[test]
    fn alternates_escape_attribute_breakouts() {
        let i = i18n();
        let html = i.alternates_html("https://example.com", r#"/x"><script>"#);
        assert!(!html.contains("<script"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn scope_exposes_locale_and_localized_paths() {
        let i = i18n();
        let scope = i.scope(&req("/en/fuentes", &[], None));
        assert_eq!(scope.locale(), "en");
        assert_eq!(scope.t("home.title"), "Crypto news");
        assert_eq!(scope.path("/sobre"), "/en/sobre");
    }
}
