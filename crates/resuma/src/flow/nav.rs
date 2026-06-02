//! SPA navigation helpers — query links and loader refresh controls.

use crate::core::handler::HandlerRef;
use crate::core::view::{Attr, AttrValue, Child, Element, View};
use crate::core::FlowRequest;

use super::runtime::current_request;

/// Build a path + query string (`/page?a=1&b=2`). Empty values are omitted.
pub fn build_query_href(path: &str, pairs: &[(&str, &str)]) -> String {
    let filtered: Vec<_> = pairs
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .copied()
        .collect();
    if filtered.is_empty() {
        return path.to_string();
    }
    let qs = serde_urlencoded::to_string(filtered).unwrap_or_default();
    if qs.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{qs}")
    }
}

/// Current path + query for active-state checks (e.g. in layouts).
pub fn current_location_href() -> String {
    let Some(req) = current_request() else {
        return "/".into();
    };
    let pairs: Vec<_> = req
        .query
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    build_query_href(&req.path, &pairs)
}

/// `<a>` with `data-r-nav` whose `href` includes query parameters.
pub fn query_nav_link(
    path: &str,
    query: &[(&str, &str)],
    active_class: impl Into<String>,
    class: impl Into<String>,
    children: Vec<Child>,
) -> View {
    let href = build_query_href(path, query);
    let active_class = active_class.into();
    let class = class.into();
    let current = current_location_href();
    let is_active = href == current;
    let merged_class = if is_active && !active_class.is_empty() {
        format!("{class} {active_class}")
    } else {
        class
    };

    View::Element(Element {
        tag: "a".into(),
        attrs: vec![
            Attr {
                name: "href".into(),
                value: AttrValue::Static(href),
            },
            Attr {
                name: "class".into(),
                value: AttrValue::Static(merged_class),
            },
            Attr {
                name: "data-r-nav".into(),
                value: AttrValue::Static("true".into()),
            },
            Attr {
                name: "data-r-active-class".into(),
                value: AttrValue::Static(active_class),
            },
        ],
        children,
        dom_id: None,
    })
}

/// `<input>` / `<select>` that SPA-navigates on change to refresh server `#[load]` data.
///
/// Updates `param` on `path`, preserving `preserve` query keys from the current URL.
/// Uses `__resuma.navigate` + `__resuma.buildUrl` (always use `event.target`, not `currentTarget`).
pub fn loader_refresh_input(
    path: &str,
    param: &str,
    value: &str,
    preserve: &[&str],
    input_type: &str,
    extra_attrs: Vec<(&str, AttrValue)>,
) -> View {
    let preserve_json = serde_json::to_string(preserve).unwrap_or_else(|_| "[]".into());
    let path_js = serde_json::to_string(path).unwrap_or_else(|_| "\"/\"".into());
    let param_js = serde_json::to_string(param).unwrap_or_else(|_| "\"\"".into());
    let handler_body = format!(
        r#"(async (event, state, __resuma) => {{
  const input = event.target;
  if (!(input instanceof HTMLInputElement || input instanceof HTMLSelectElement)) return;
  const v = input.value;
  if (!v) return;
  const q = new URLSearchParams(location.search);
  const params = {{}};
  params[{param_js}] = v;
  for (const k of {preserve_json}) {{
    const cur = q.get(k);
    if (cur) params[k] = cur;
  }}
  await __resuma.navigate(__resuma.buildUrl({path_js}, params));
}})"#,
        param_js = param_js,
        preserve_json = preserve_json,
        path_js = path_js,
    );
    let symbol = format!("loader_refresh_{}", param.replace('-', "_"));
    let change_handler = crate::__private::register_handler(
        "change",
        "__page__",
        &symbol,
        &handler_body,
        vec![],
        vec![],
    );
    let change_handler = match change_handler {
        AttrValue::Handler(h) => h,
        _ => HandlerRef {
            event: "change".into(),
            chunk: "__page__".into(),
            symbol: symbol.clone(),
            captures: vec![],
            inline: Some(handler_body.clone()),
        },
    };

    let mut attrs = vec![
        Attr {
            name: "type".into(),
            value: AttrValue::Static(input_type.into()),
        },
        Attr {
            name: "name".into(),
            value: AttrValue::Static(param.into()),
        },
        Attr {
            name: "value".into(),
            value: AttrValue::Static(value.into()),
        },
        Attr {
            name: "on:change".into(),
            value: AttrValue::Handler(change_handler),
        },
    ];
    for (name, value) in extra_attrs {
        attrs.push(Attr {
            name: name.into(),
            value,
        });
    }

    View::Element(Element {
        tag: "input".into(),
        attrs,
        children: vec![],
        dom_id: None,
    })
}

/// GET form that navigates via SPA instead of full reload (`data-r-loader-refresh`).
pub fn loader_refresh_form(path: &str, preserve_from_url: &[&str], children: Vec<Child>) -> View {
    let preserve_json = serde_json::to_string(preserve_from_url).unwrap_or_else(|_| "[]".into());
    let path_js = serde_json::to_string(path).unwrap_or_else(|_| "\"/\"".into());
    let handler_body = format!(
        r#"(async (event, state, __resuma) => {{
  event.preventDefault();
  const form = event.target;
  if (!(form instanceof HTMLFormElement)) return;
  const fd = new FormData(form);
  const params = {{}};
  fd.forEach((val, key) => {{ if (String(val)) params[key] = String(val); }});
  const q = new URLSearchParams(location.search);
  for (const k of {preserve_json}) {{
    if (!params[k]) {{
      const cur = q.get(k);
      if (cur) params[k] = cur;
    }}
  }}
  await __resuma.navigate(__resuma.buildUrl({path_js}, params));
}})"#,
        preserve_json = preserve_json,
        path_js = path_js,
    );
    let submit_handler = match crate::__private::register_handler(
        "submit",
        "__page__",
        "loader_refresh_form",
        &handler_body,
        vec![],
        vec![],
    ) {
        AttrValue::Handler(h) => h,
        _ => HandlerRef {
            event: "submit".into(),
            chunk: "__page__".into(),
            symbol: "loader_refresh_form".into(),
            captures: vec![],
            inline: Some(handler_body.clone()),
        },
    };

    View::Element(Element {
        tag: "form".into(),
        attrs: vec![
            Attr {
                name: "method".into(),
                value: AttrValue::Static("GET".into()),
            },
            Attr {
                name: "action".into(),
                value: AttrValue::Static(path.into()),
            },
            Attr {
                name: "data-r-loader-refresh".into(),
                value: AttrValue::Static("true".into()),
            },
            Attr {
                name: "on:submit".into(),
                value: AttrValue::Handler(submit_handler),
            },
        ],
        children,
        dom_id: None,
    })
}

/// Merge `Theme` primary/background into PWA colors when no explicit override exists.
pub fn theme_into_pwa(theme: &crate::core::Theme, cfg: &mut super::pwa::FlowPwaConfig) {
    if !theme.primary.is_empty() {
        cfg.theme_color = theme.primary.clone();
    }
    if !theme.background.is_empty() {
        cfg.background_color = theme.background.clone();
    }
}

/// Build query pairs from a request for link helpers.
pub fn query_pairs_from(req: &FlowRequest) -> Vec<(String, String)> {
    req.query
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Convert owned pairs to `&str` slices for [`build_query_href`].
pub fn pairs_as_refs(pairs: &[(String, String)]) -> Vec<(&str, &str)> {
    pairs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_query_href_omits_empty() {
        assert_eq!(build_query_href("/r", &[]), "/r");
        assert_eq!(
            build_query_href("/r", &[("fecha", "2026-06-02"), ("servicio", "")]),
            "/r?fecha=2026-06-02"
        );
    }
}
