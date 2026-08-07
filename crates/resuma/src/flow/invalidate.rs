//! Stable loader invalidation — re-run server `#[load]` handlers via SPA navigation.

use crate::core::task::use_visible_task;
use crate::core::view::{Child, View};

use super::nav::build_query_href;

/// Build an href that triggers SPA navigation and re-runs server loaders for `path`.
///
/// Appends `_r` (cache-bust) so repeated invalidations refetch. Use with
/// `NavLink`, `loader_refresh_input`, or `js! { await __resuma.invalidate("/path"); }`.
pub fn invalidate_href(path: &str, query: &[(&str, &str)]) -> String {
    let mut pairs: Vec<(&str, &str)> = query.to_vec();
    pairs.push(("_r", "1"));
    build_query_href(path, &pairs)
}

/// Same as [`invalidate_href`] but uses a unique `_r` timestamp on each call.
pub fn invalidate_href_now(path: &str, query: &[(&str, &str)]) -> String {
    let bust = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_else(|_| "0".into());
    let mut owned: Vec<(String, String)> = query
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    owned.push(("_r".into(), bust));
    let refs: Vec<(&str, &str)> = owned
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    build_query_href(path, &refs)
}

/// `<a data-r-nav>` that navigates to [`invalidate_href_now`] for the current path + optional query.
pub fn invalidate_link(path: &str, query: &[(&str, &str)], label: impl Into<String>) -> View {
    let href = invalidate_href_now(path, query);
    View::Element(crate::core::view::Element {
        tag: "a".into(),
        attrs: vec![
            crate::core::view::Attr {
                name: "href".into(),
                value: crate::core::view::AttrValue::Static(href),
            },
            crate::core::view::Attr {
                name: "data-r-nav".into(),
                value: crate::core::view::AttrValue::Static("true".into()),
            },
        ],
        children: vec![Child::Text(label.into())],
        dom_id: None,
    })
}

/// Poll server `#[load]` data for `path` on an interval via SPA `__resuma.invalidate`.
///
/// Prefer this over `<meta http-equiv="refresh">` — it reuses the resumable mount
/// pipeline, skips ticks while the tab is hidden, and tears the timer down on SPA
/// remount (visible-task cleanup).
///
/// Soft invalidate semantics (runtime): `history.replaceState` (no history spam),
/// no scroll-to-top, cache-bust `_r` stripped from the visible URL, and NavLink
/// prefetch is bypassed so polls always see fresh `#[load]` data. This still
/// remounts `#resuma-root` (full SPA swap) — not a partial loader patch.
///
/// Call during page render (side-effect registers a visible task). Returns
/// [`View::empty`] so it can sit inside `view!` trees:
///
/// ```rust,ignore
/// view! {
///     <div>
///         {loader_poll("/", 8_000)}
///         // dashboard body…
///     </div>
/// }
/// ```
pub fn loader_poll(path: &str, interval_ms: u64) -> View {
    let path_js = serde_json::to_string(path).unwrap_or_else(|_| "\"/\"".into());
    let ms = interval_ms.max(250);
    let body = format!(
        r#"(async (state, __resuma) => {{
  const path = {path_js};
  const ms = {ms};
  let busy = false;
  const tick = async () => {{
    if (busy || document.visibilityState === "hidden") return;
    busy = true;
    try {{ await __resuma.invalidate(path); }} catch (_) {{}}
    finally {{ busy = false; }}
  }};
  const id = setInterval(tick, ms);
  return () => {{ clearInterval(id); }};
}})"#,
        path_js = path_js,
        ms = ms,
    );
    let _ = use_visible_task(body);
    View::empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalidate_href_adds_cache_bust_param() {
        let href = invalidate_href("/users", &[("q", "a")]);
        assert!(href.starts_with("/users?"));
        assert!(href.contains("q=a"));
        assert!(href.contains("_r=1"));
    }
}
