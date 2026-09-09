//! Third-party script deferral and ops/SEO helpers dogfooded on GoldevLab microsaas
//! (forgeyt, under200kb, pdfforge, placaqr).
//!
//! Lab tools (PageSpeed / monitor.works) measure Total Blocking Time without user
//! input. Eager AdSense/GA tanks mobile scores; loading after engagement (or a long
//! idle fallback) keeps real users monetized while lab TBT stays green.

use super::escape::escape_attr;

/// RFC 9116 `security.txt` body for `/.well-known/security.txt`.
///
/// Pass the result through [`crate::flow::FlowApp::static_asset`] (leak once at boot):
///
/// ```ignore
/// let body = resuma::ssr::third_party::security_txt("https://example.com", Some("ops@example.com"));
/// let leaked: &'static [u8] = Box::leak(body.into_bytes().into_boxed_slice());
/// app = app.static_asset("/.well-known/security.txt", leaked, "text/plain; charset=utf-8");
/// ```
pub fn security_txt(canonical_origin: &str, contact_email: Option<&str>) -> String {
    let origin = canonical_origin.trim().trim_end_matches('/');
    let mut out = String::new();
    if let Some(email) = contact_email
        .map(str::trim)
        .filter(|e| e.contains('@') && !e.contains(' ') && !e.contains('<'))
    {
        out.push_str("Contact: mailto:");
        out.push_str(email);
        out.push('\n');
    }
    if !origin.is_empty() {
        out.push_str("Canonical: ");
        out.push_str(origin);
        out.push_str("/.well-known/security.txt\n");
    }
    out.push_str("Preferred-Languages: en\n");
    out.push_str("Expires: 2027-12-31T23:59:59Z\n");
    out
}

/// Inline `<script>` that injects a `type="module"` tag after first engagement
/// (pointer/key/touch/scroll) or `requestIdleCallback` with `idle_timeout_ms`.
///
/// `boot_flag` must be a unique JS identifier (e.g. `__myAppAdsJs`) so the loader
/// runs once. `src` is the module URL (`/js/ads.js?v=3`).
pub fn deferred_module_loader(src: &str, boot_flag: &str, idle_timeout_ms: u32) -> String {
    deferred_loader(src, boot_flag, idle_timeout_ms, true)
}

/// Same as [`deferred_module_loader`], but for classic `async` scripts (gtag, etc.).
pub fn deferred_external_script(src: &str, boot_flag: &str, idle_timeout_ms: u32) -> String {
    deferred_loader(src, boot_flag, idle_timeout_ms, false)
}

fn deferred_loader(src: &str, boot_flag: &str, idle_timeout_ms: u32, module: bool) -> String {
    let src = escape_attr(src.trim());
    let flag = sanitize_js_ident(boot_flag);
    let idle = idle_timeout_ms.max(1000);
    let fallback = idle.saturating_mul(2) / 3;
    let type_line = if module {
        "s.type='module';"
    } else {
        "s.async=true;"
    };
    format!(
        r#"<script>
(function(){{
  function load(){{
    if(window.{flag})return;
    window.{flag}=1;
    var s=document.createElement('script');
    {type_line}
    s.src={src:?};
    s.fetchPriority='low';
    document.head.appendChild(s);
  }}
  function arm(){{
    var start=function(){{load();}};
    ['pointerdown','keydown','touchstart','scroll'].forEach(function(e){{
      window.addEventListener(e,start,{{once:true,passive:true}});
    }});
    if('requestIdleCallback' in window)requestIdleCallback(start,{{timeout:{idle}}});
    else window.addEventListener('load',function(){{setTimeout(start,{fallback});}},{{once:true}});
  }}
  if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',arm,{{once:true}});
  else arm();
}})();
</script>"#
    )
}

fn sanitize_js_ident(raw: &str) -> String {
    let s: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '$')
        .collect();
    if s.is_empty() || s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("__resumaBoot_{}", s.chars().take(24).collect::<String>())
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn security_txt_includes_canonical_and_optional_contact() {
        let body = security_txt("https://forgeyt.com/", Some("ops@forgeyt.com"));
        assert!(body.contains("Contact: mailto:ops@forgeyt.com\n"));
        assert!(body.contains("Canonical: https://forgeyt.com/.well-known/security.txt\n"));
        assert!(body.contains("Expires:"));
    }

    #[test]
    fn deferred_module_mentions_engagement_and_idle() {
        let html = deferred_module_loader("/js/ads.js?v=3", "__appAdsJs", 12_000);
        assert!(html.contains("type='module'"));
        assert!(html.contains("/js/ads.js?v=3"));
        assert!(html.contains("__appAdsJs"));
        assert!(html.contains("timeout:12000"));
        assert!(html.contains("pointerdown"));
    }

    #[test]
    fn deferred_external_is_async_not_module() {
        let html = deferred_external_script(
            "https://www.googletagmanager.com/gtag/js?id=G-TEST",
            "__appGa",
            12_000,
        );
        assert!(html.contains("async=true"));
        assert!(!html.contains("type='module'"));
        assert!(html.contains("googletagmanager.com"));
    }
}
