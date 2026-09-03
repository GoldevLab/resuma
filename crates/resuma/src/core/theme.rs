//! Theme tokens (SSR context) and live document palettes (`html[data-theme]`).
//!
//! [`provide_theme`] / [`theme_css_vars`] are a **snapshot** — inline styles on a
//! wrapper do not update when the user picks another palette. For a whole-app
//! restyle (navbar, docs, marketing), use [`HtmlTheme`] + `[data-r-theme]`
//! buttons. The blocking head boot writes `data-theme` before first paint and
//! does **not** wrap the swap in View Transitions (that races native popovers).

use serde::{Deserialize, Serialize};

use super::app_context::{provide_context, use_context, ContextId};
use super::view::{Attr, AttrValue, Child, Element, View};

/// Application theme tokens (cookbook pattern — SSR / `<Show>` branches).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub mode: String,
    pub primary: String,
    pub background: String,
    pub foreground: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            mode: "dark".into(),
            primary: "#6366f1".into(),
            background: "#0b1020".into(),
            foreground: "#e6e8ee".into(),
        }
    }
}

static THEME_CTX: ContextId<Theme> = ContextId::new();

/// Provide theme for descendant components (serialized in resumability payload).
pub fn provide_theme(theme: Theme) {
    provide_context(&THEME_CTX, theme);
}

/// Read the active theme.
pub fn use_theme() -> Theme {
    use_context(&THEME_CTX)
}

/// Inline CSS variables for a **one-shot** snapshot (`style={theme_css_vars(&theme)}`).
///
/// This does **not** live-update when [`HtmlTheme`] changes `html[data-theme]`.
/// For clickable palettes, style with `html[data-theme="night"] { --bg: … }` and
/// put [`theme_switch`] / `data-r-theme` on the buttons. Selected state is
/// `aria-pressed` plus class `r-theme-on` (do not bake a selected class in SSR).
pub fn theme_css_vars(theme: &Theme) -> String {
    format!(
        "--resuma-primary:{p};--resuma-bg:{bg};--resuma-fg:{fg};",
        p = theme.primary,
        bg = theme.background,
        fg = theme.foreground,
    )
}

/// Live document theme: `<html data-theme>` + cookie + `localStorage`.
///
/// Buttons with `data-r-theme="slate"` (or [`theme_switch`]) restyle the whole
/// page. Layout `onClick` handlers cannot do this reliably — they live in a
/// lazy chunk. Enable with [`crate::ResumaApp::with_html_theme`].
#[derive(Debug, Clone)]
pub struct HtmlTheme {
    pub ids: Vec<String>,
    pub dark: Vec<String>,
    pub cookie: String,
    pub storage_key: String,
    pub default_light: String,
    pub default_dark: String,
}

impl Default for HtmlTheme {
    fn default() -> Self {
        Self::new(["light", "dark"]).dark(["dark"])
    }
}

impl HtmlTheme {
    /// `ids[0]` is the default light palette when nothing is stored.
    pub fn new(ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let ids: Vec<String> = ids
            .into_iter()
            .map(Into::into)
            .filter(|s| is_theme_token(s))
            .collect();
        let default_light = ids.first().cloned().unwrap_or_else(|| "light".into());
        let dark: Vec<String> = ids.iter().filter(|id| looks_dark(id)).cloned().collect();
        let default_dark = dark
            .first()
            .cloned()
            .unwrap_or_else(|| default_light.clone());
        Self {
            ids,
            dark,
            cookie: "resuma_theme".into(),
            storage_key: "resuma-theme".into(),
            default_light,
            default_dark,
        }
    }

    pub fn dark(mut self, ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.dark = ids
            .into_iter()
            .map(Into::into)
            .filter(|s| is_theme_token(s))
            .collect();
        if self.default_dark.is_empty() || !self.allows(&self.default_dark) {
            if let Some(d) = self.dark.first() {
                self.default_dark = d.clone();
            }
        }
        self
    }

    pub fn cookie(mut self, name: impl Into<String>) -> Self {
        let name = theme_token(&name.into());
        if !name.is_empty() {
            self.cookie = name;
        }
        self
    }

    pub fn storage_key(mut self, key: impl Into<String>) -> Self {
        let key = theme_token(&key.into());
        if !key.is_empty() {
            self.storage_key = key;
        }
        self
    }

    pub fn allows(&self, id: &str) -> bool {
        self.ids.iter().any(|x| x == id)
    }

    /// Cookie / staged id, else [`Self::default_light`].
    pub fn resolve(&self, cookie: Option<&str>) -> String {
        cookie
            .map(str::trim)
            .filter(|id| self.allows(id))
            .map(str::to_string)
            .unwrap_or_else(|| self.default_light.clone())
    }

    /// Blocking `<head>` boot — classic script, no `type=module`.
    pub fn boot_script(&self) -> String {
        let ids = js_id_map(&self.ids);
        let dark = js_id_map(&self.dark);
        let cookie = self.cookie.as_str();
        let key = self.storage_key.as_str();
        let light = self.default_light.as_str();
        let night = if self.allows(&self.default_dark) {
            self.default_dark.as_str()
        } else {
            light
        };
        format!(
            r##"<meta name="color-scheme" content="light dark">
<script>
(function(){{
  var C="{cookie}",K="{key}",I={ids},D={dark},L="{light}",N="{night}";
  function ok(id){{return I[id]?id:L}}
  function scheme(id){{return D[id]?"dark":"light"}}
  function hideFrom(el){{
    if(!el||!el.closest)return;
    var p=el.closest("[popover]");
    if(p&&p.hidePopover)try{{p.hidePopover()}}catch(e){{}}
  }}
  function stored(){{
    var id=null;
    try{{id=localStorage.getItem(K)}}catch(e){{}}
    if(!id||!I[id]){{var m=document.cookie.match(new RegExp("(?:^|; )"+C+"=([^;]*)"));id=m?decodeURIComponent(m[1]):""}}
    return (id&&I[id])?id:"";
  }}
  function apply(id,persist,from){{
    id=ok(id);
    var r=document.documentElement;
    r.setAttribute("data-theme",id);
    r.style.colorScheme=scheme(id);
    var m=document.querySelector('meta[name="color-scheme"]');
    if(m)m.content=scheme(id);
    var tc=document.querySelector('meta[name="theme-color"]');
    if(tc){{try{{var bg=(getComputedStyle(r).getPropertyValue("--bg")||"").trim();if(bg)tc.content=bg}}catch(e){{}}}}
    document.querySelectorAll("[data-r-theme]").forEach(function(b){{
      var on=b.getAttribute("data-r-theme")===id;
      b.setAttribute("aria-pressed",on?"true":"false");
      b.classList.toggle("r-theme-on",on);
    }});
    if(!persist)return;
    try{{localStorage.setItem(K,id)}}catch(e){{}}
    document.cookie=C+"="+encodeURIComponent(id)+";Path=/;Max-Age=31536000;SameSite=Lax"+(location.protocol==="https:"?";Secure":"");
    hideFrom(from);
    try{{if(window.__resuma&&window.__resuma.announce)window.__resuma.announce("Theme "+id)}}catch(e){{}}
  }}
  var id=stored();
  if(!id)id=(window.matchMedia&&window.matchMedia("(prefers-color-scheme: dark)").matches)?N:L;
  apply(id,false);
  window.__resumaSetTheme=function(n){{apply(n,true)}};
  document.addEventListener("click",function(ev){{
    var t=ev.target;if(!t||!t.closest)return;
    var b=t.closest("[data-r-theme]");if(!b)return;
    var n=b.getAttribute("data-r-theme");if(n)apply(n,true,b);
  }},true);
  function sync(){{apply(document.documentElement.getAttribute("data-theme")||L,false)}}
  if(document.readyState==="loading")document.addEventListener("DOMContentLoaded",sync);
  else sync();
  document.addEventListener("resuma:navigate",sync);
  try{{
    window.matchMedia&&window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change",function(e){{
      if(stored())return;
      apply(e.matches?N:L,false);
    }});
  }}catch(e){{}}
}})();
</script>"##
        )
    }
}

impl<const N: usize> From<[&str; N]> for HtmlTheme {
    fn from(ids: [&str; N]) -> Self {
        Self::new(ids)
    }
}

impl From<Vec<&str>> for HtmlTheme {
    fn from(ids: Vec<&str>) -> Self {
        Self::new(ids)
    }
}

impl From<Vec<String>> for HtmlTheme {
    fn from(ids: Vec<String>) -> Self {
        Self::new(ids)
    }
}

/// Button that sets the document theme (`data-r-theme`). Works from a layout
/// without waiting on a handler chunk.
pub fn theme_switch(id: impl Into<String>, children: Vec<Child>) -> View {
    let id = theme_token(&id.into());
    let id = if id.is_empty() { "theme".into() } else { id };
    let extra = vec![
        Attr {
            name: "type".into(),
            value: AttrValue::Static("button".into()),
        },
        Attr {
            name: "data-r-theme".into(),
            value: AttrValue::Static(id),
        },
        Attr {
            name: "aria-pressed".into(),
            value: AttrValue::Static("false".into()),
        },
    ];
    match children.len() {
        1 => match children.into_iter().next() {
            Some(Child::View(View::Element(mut el))) if el.tag.eq_ignore_ascii_case("button") => {
                el.merge_attrs(extra);
                View::Element(el)
            }
            Some(other) => View::Element(Element {
                tag: "button".into(),
                attrs: extra,
                children: vec![other],
                dom_id: None,
            }),
            None => View::Element(Element {
                tag: "button".into(),
                attrs: extra,
                children: vec![],
                dom_id: None,
            }),
        },
        _ => View::Element(Element {
            tag: "button".into(),
            attrs: extra,
            children,
            dom_id: None,
        }),
    }
}

fn looks_dark(id: &str) -> bool {
    let l = id.to_ascii_lowercase();
    l == "dark" || l.contains("night") || l.contains("midnight")
}

pub(crate) fn is_theme_token(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 48
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn theme_token(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(48)
        .collect()
}

fn js_id_map(ids: &[String]) -> String {
    let mut out = String::from("{");
    for (i, id) in ids.iter().filter(|s| is_theme_token(s)).enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(id);
        out.push_str(":1");
    }
    out.push('}');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssr::render_view;

    #[test]
    fn boot_script_has_no_view_transition_and_binds_data_r_theme() {
        let html = HtmlTheme::new(["paper", "slate", "midnight"])
            .dark(["midnight"])
            .boot_script();
        assert!(html.contains("data-r-theme"), "{html}");
        assert!(html.contains("__resumaSetTheme"), "{html}");
        assert!(!html.contains("startViewTransition"), "{html}");
        assert!(!html.contains("pointerdown"), "{html}");
        assert!(html.contains("r-theme-on"), "{html}");
        assert!(html.contains("paper:1"), "{html}");
        assert!(html.contains("addEventListener(\"click\""), "{html}");
    }

    #[test]
    fn new_guesses_dark_ids() {
        let t = HtmlTheme::new(["light", "dark"]);
        assert_eq!(t.dark, vec!["dark"]);
        assert_eq!(t.default_dark, "dark");
        let t = HtmlTheme::new(["paper", "slate", "midnight"]);
        assert_eq!(t.dark, vec!["midnight"]);
    }

    #[test]
    fn rejects_unsafe_ids() {
        let t = HtmlTheme::new(["ok", "bad id", "x;alert(1)"]);
        assert_eq!(t.ids, vec!["ok"]);
        assert!(!t.allows("bad id"));
    }

    #[test]
    fn theme_switch_emits_data_r_theme() {
        let html = render_view(&theme_switch("slate", vec![Child::Text("Slate".into())]));
        assert!(html.contains("data-r-theme=\"slate\""), "{html}");
        assert!(html.contains("type=\"button\""), "{html}");
        assert!(html.contains("Slate"), "{html}");
    }

    #[test]
    fn theme_switch_forces_type_button_on_submit() {
        let btn = View::Element(Element {
            tag: "button".into(),
            attrs: vec![Attr {
                name: "type".into(),
                value: AttrValue::Static("submit".into()),
            }],
            children: vec![Child::Text("Go".into())],
            dom_id: None,
        });
        let html = render_view(&theme_switch("slate", vec![Child::View(btn)]));
        assert!(html.contains("type=\"button\""), "{html}");
        assert!(!html.contains("type=\"submit\""), "{html}");
        assert!(html.contains("data-r-theme=\"slate\""), "{html}");
    }
}
