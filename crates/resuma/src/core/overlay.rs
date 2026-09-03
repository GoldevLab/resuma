//! Native overlay primitives — Popup, Modal, GestureView.
//!
//! These emit HTML the browser already understands (`popover`, `<dialog>`, CSS
//! anchor positioning, Invoker Commands). The lazy `/_resuma/ui.js` chunk is
//! only a fallback for browsers missing those APIs, plus gesture/virtual-list
//! wiring. App code stays in Rust (`view!` or these helpers).

use super::signal::SignalId;
use super::view::{Attr, AttrValue, Child, Element, View};

/// Flip order for anchored popups (CSS `position-try` / JS fallback).
pub const DEFAULT_POPUP_POSITIONS: &str = "bottom right top left";

fn static_attr(name: &str, value: impl Into<String>) -> Attr {
    Attr {
        name: name.into(),
        value: AttrValue::Static(value.into()),
    }
}

/// Keep overlay ids valid in HTML `id` / CSS `anchor-name`.
pub fn overlay_token(raw: &str) -> String {
    let s: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    if s.is_empty() {
        "overlay".into()
    } else {
        s
    }
}

fn position_area(first: &str) -> &'static str {
    match first {
        "top" => "block-start span-inline-end",
        "right" => "inline-end span-block-end",
        "left" => "inline-start span-block-end",
        _ => "block-end span-inline-end",
    }
}

fn first_position(positions: &str) -> &str {
    positions
        .split(|c: char| c.is_ascii_whitespace() || c == ',')
        .find(|s| !s.is_empty())
        .unwrap_or("bottom")
}

fn wrap_button(children: Vec<Child>, extra: Vec<Attr>) -> View {
    let mut el = Element {
        tag: "button".into(),
        attrs: vec![static_attr("type", "button")],
        children,
        dom_id: None,
    };
    el.merge_attrs(extra);
    View::Element(el)
}

fn decorate_view(view: View, extra: Vec<Attr>) -> View {
    match view {
        View::Element(mut el) => {
            el.merge_attrs(extra);
            View::Element(el)
        }
        View::Fragment(frag) if frag.children.len() == 1 => {
            match frag.children.into_iter().next() {
                Some(Child::View(inner)) => decorate_view(inner, extra),
                Some(Child::Text(t)) => wrap_button(vec![Child::Text(t)], extra),
                None => wrap_button(vec![], extra),
            }
        }
        other => wrap_button(vec![Child::View(other)], extra),
    }
}

fn children_to_view(children: Vec<Child>) -> View {
    match children.len() {
        0 => View::empty(),
        1 => match children.into_iter().next() {
            Some(Child::View(v)) => v,
            Some(Child::Text(t)) => View::Text(t),
            None => View::empty(),
        },
        _ => View::fragment(children),
    }
}

/// Options for [`popup`].
#[derive(Debug, Clone)]
pub struct PopupOpts {
    pub id: String,
    /// Space-separated priorities, default [`DEFAULT_POPUP_POSITIONS`].
    pub positions: String,
    /// Toggle closed when shown again (native `popover=auto` already toggles).
    pub dismiss_if_shown: bool,
    /// Keep the panel in the DOM while closed (native popover default).
    pub cacheable: bool,
    /// Optional bool signal synced with open state.
    pub open: Option<SignalId>,
    pub class: String,
}

impl Default for PopupOpts {
    fn default() -> Self {
        Self {
            id: "popup".into(),
            positions: DEFAULT_POPUP_POSITIONS.into(),
            dismiss_if_shown: true,
            cacheable: true,
            open: None,
            class: String::new(),
        }
    }
}

/// Standalone popup trigger (when the panel is declared elsewhere).
pub fn popup_anchor(id: impl Into<String>, children: Vec<Child>) -> View {
    let id = overlay_token(&id.into());
    decorate_view(children_to_view(children), popup_anchor_attrs(&id))
}

fn popup_anchor_attrs(id: &str) -> Vec<Attr> {
    let panel_id = format!("r-popup-{id}");
    let anchor_id = format!("r-popup-anchor-{id}");
    let name = format!("--r-popup-{id}");
    vec![
        static_attr("type", "button"),
        static_attr("id", anchor_id),
        static_attr("popovertarget", panel_id),
        static_attr("data-r-popup-trigger", id),
        static_attr("style", format!("anchor-name: {name}")),
    ]
}

fn popup_panel(opts: &PopupOpts, id: &str, panel: Vec<Child>) -> View {
    let panel_id = format!("r-popup-{id}");
    let name = format!("--r-popup-{id}");
    let positions = if opts.positions.is_empty() {
        DEFAULT_POPUP_POSITIONS
    } else {
        opts.positions.as_str()
    };
    let area = position_area(first_position(positions));
    let class = if opts.class.is_empty() {
        "r-popup".into()
    } else {
        format!("r-popup {}", opts.class)
    };
    let mut attrs = vec![
        static_attr("id", panel_id),
        static_attr("popover", "auto"),
        static_attr("class", class),
        static_attr("data-r-popup", id),
        static_attr("data-r-popup-positions", positions),
        static_attr(
            "style",
            format!("position-anchor: {name}; position-area: {area}"),
        ),
    ];
    if opts.dismiss_if_shown {
        attrs.push(static_attr("data-r-popup-dismiss-if-shown", "true"));
    }
    if opts.cacheable {
        attrs.push(static_attr("data-r-popup-cacheable", "true"));
    }
    if let Some(sig) = opts.open {
        attrs.push(static_attr("data-r-open-signal", sig.to_string()));
    }
    View::Element(Element {
        tag: "div".into(),
        attrs,
        children: panel,
        dom_id: None,
    })
}

/// Anchored popup: trigger + `[popover]` panel. Positioning is CSS (`anchor-name`).
///
/// Do not add `position: relative` or `overflow: hidden` on the panel — `/_resuma/ui.css`
/// forces `position: absolute` so clicks reach buttons inside the top layer.
pub fn popup(opts: PopupOpts, anchor: Vec<Child>, panel: Vec<Child>) -> View {
    let id = overlay_token(&opts.id);
    let trigger = decorate_view(children_to_view(anchor), popup_anchor_attrs(&id));
    View::fragment(vec![
        Child::View(trigger),
        Child::View(popup_panel(&opts, &id, panel)),
    ])
}

/// Options for [`modal`].
#[derive(Debug, Clone)]
pub struct ModalOpts {
    pub id: String,
    /// Native `closedby` (`any` = ESC + backdrop). Fallback JS for Safari.
    pub closed_by: String,
    pub open: Option<SignalId>,
    pub class: String,
}

impl Default for ModalOpts {
    fn default() -> Self {
        Self {
            id: "modal".into(),
            closed_by: "any".into(),
            open: None,
            class: String::new(),
        }
    }
}

/// Button that opens a [`modal`] (`command="show-modal"` + JS fallback).
pub fn modal_trigger(id: impl Into<String>, children: Vec<Child>) -> View {
    let id = overlay_token(&id.into());
    decorate_view(children_to_view(children), modal_trigger_attrs(&id))
}

fn modal_trigger_attrs(id: &str) -> Vec<Attr> {
    let dialog_id = format!("r-modal-{id}");
    vec![
        static_attr("type", "button"),
        static_attr("command", "show-modal"),
        static_attr("commandfor", &dialog_id),
        static_attr("data-r-modal-open", id),
    ]
}

fn modal_dialog(opts: &ModalOpts, id: &str, children: Vec<Child>) -> View {
    let dialog_id = format!("r-modal-{id}");
    let class = if opts.class.is_empty() {
        "r-modal".into()
    } else {
        format!("r-modal {}", opts.class)
    };
    let closed_by = if opts.closed_by.is_empty() {
        "any"
    } else {
        opts.closed_by.as_str()
    };
    let mut attrs = vec![
        static_attr("id", dialog_id),
        static_attr("class", class),
        static_attr("data-r-modal", id),
        static_attr("closedby", closed_by),
    ];
    if let Some(sig) = opts.open {
        attrs.push(static_attr("data-r-open-signal", sig.to_string()));
    }
    View::Element(Element {
        tag: "dialog".into(),
        attrs,
        children,
        dom_id: None,
    })
}

/// Modal dialog with focus restore. Stack + light-dismiss live in `/_resuma/ui.js`
/// only when the native `closedby` / Invoker Commands APIs are missing.
pub fn modal(opts: ModalOpts, trigger: Vec<Child>, body: Vec<Child>) -> View {
    let id = overlay_token(&opts.id);
    let dialog = modal_dialog(&opts, &id, body);
    if trigger.is_empty() {
        return dialog;
    }
    View::fragment(vec![
        Child::View(decorate_view(
            children_to_view(trigger),
            modal_trigger_attrs(&id),
        )),
        Child::View(dialog),
    ])
}

/// Options for [`gesture_view`].
#[derive(Debug, Clone, Default)]
pub struct GestureOpts {
    /// `horizontal` | `vertical` | empty (either).
    pub preferred_pan: String,
    pub pan_threshold: u32,
}

/// Pointer surface: pan / pinch / long-press / double-tap / wheel.
///
/// Handlers (`onPan`, `onLongPress`, …) are normal `view!` events compiled to
/// `data-r-on:*`. Gesture dispatch lives in the lazy UI chunk, not the loader.
pub fn gesture_view(
    opts: GestureOpts,
    extra: Vec<(String, AttrValue)>,
    children: Vec<Child>,
) -> View {
    let mut attrs = vec![static_attr("data-r-gesture", "true")];
    if !opts.preferred_pan.is_empty() {
        attrs.push(static_attr("data-r-preferred-pan", &opts.preferred_pan));
    }
    let threshold = if opts.pan_threshold == 0 {
        10
    } else {
        opts.pan_threshold
    };
    attrs.push(static_attr("data-r-pan-threshold", threshold.to_string()));
    for (name, value) in extra {
        attrs.push(Attr { name, value });
    }
    View::Element(Element {
        tag: "div".into(),
        attrs,
        children,
        dom_id: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::context::{with_context, RenderContext, RenderMode};
    use crate::core::signal::Signal;
    use crate::ssr::render_view;

    #[test]
    fn popup_emits_native_popover_and_anchor() {
        let html = render_view(&popup(
            PopupOpts {
                id: "menu".into(),
                ..Default::default()
            },
            vec![Child::Text("Open".into())],
            vec![Child::Text("Panel".into())],
        ));
        assert!(html.contains("popovertarget=\"r-popup-menu\""), "{html}");
        assert!(html.contains("popover=\"auto\""), "{html}");
        assert!(html.contains("data-r-popup=\"menu\""), "{html}");
        assert!(html.contains("anchor-name: --r-popup-menu"), "{html}");
        assert!(html.contains("position-anchor: --r-popup-menu"), "{html}");
        assert!(html.contains("bottom right top left"), "{html}");
        assert!(html.contains("type=\"button\""), "{html}");
    }

    #[test]
    fn popup_trigger_overwrites_submit_type() {
        let trigger = View::Element(Element {
            tag: "button".into(),
            attrs: vec![static_attr("type", "submit")],
            children: vec![Child::Text("Open".into())],
            dom_id: None,
        });
        let html = render_view(&popup(
            PopupOpts {
                id: "menu".into(),
                ..Default::default()
            },
            vec![Child::View(trigger)],
            vec![Child::Text("Panel".into())],
        ));
        assert!(html.contains("type=\"button\""), "{html}");
        assert!(!html.contains("type=\"submit\""), "{html}");
    }

    #[test]
    fn popup_trigger_keeps_host_style_and_adds_anchor_name() {
        let trigger = View::Element(Element {
            tag: "button".into(),
            attrs: vec![
                static_attr("type", "button"),
                static_attr("style", "color: red"),
            ],
            children: vec![Child::Text("Open".into())],
            dom_id: None,
        });
        let html = render_view(&popup(
            PopupOpts {
                id: "menu".into(),
                ..Default::default()
            },
            vec![Child::View(trigger)],
            vec![Child::Text("Panel".into())],
        ));
        assert!(html.contains("color: red"), "{html}");
        assert!(html.contains("anchor-name: --r-popup-menu"), "{html}");
    }

    #[test]
    fn modal_emits_dialog_closedby_and_invoker() {
        let html = render_view(&modal(
            ModalOpts {
                id: "confirm".into(),
                ..Default::default()
            },
            vec![Child::Text("Open".into())],
            vec![Child::Text("Body".into())],
        ));
        assert!(html.contains("<dialog"), "{html}");
        assert!(html.contains("closedby=\"any\""), "{html}");
        assert!(html.contains("command=\"show-modal\""), "{html}");
        assert!(html.contains("commandfor=\"r-modal-confirm\""), "{html}");
        assert!(html.contains("data-r-modal=\"confirm\""), "{html}");
    }

    #[test]
    fn gesture_view_emits_marker() {
        let html = render_view(&gesture_view(
            GestureOpts {
                preferred_pan: "horizontal".into(),
                pan_threshold: 12,
            },
            vec![],
            vec![Child::Text("pad".into())],
        ));
        assert!(html.contains("data-r-gesture"), "{html}");
        assert!(
            html.contains("data-r-preferred-pan=\"horizontal\""),
            "{html}"
        );
        assert!(html.contains("data-r-pan-threshold=\"12\""), "{html}");
    }

    #[test]
    fn popup_open_signal_attr() {
        let ctx = RenderContext::new(RenderMode::Ssr);
        let html = with_context(ctx, || {
            let open = Signal::new(false);
            render_view(&popup(
                PopupOpts {
                    id: "s".into(),
                    open: Some(open.id()),
                    ..Default::default()
                },
                vec![Child::Text("t".into())],
                vec![Child::Text("p".into())],
            ))
        });
        assert!(html.contains("data-r-open-signal="), "{html}");
    }
}
