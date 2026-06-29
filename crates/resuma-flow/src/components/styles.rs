//! Shared Flow dashboard styles (include once per page via [`flow_styles`]).

use resuma::core::view::{Attr, AttrValue, Child, Element, View};

/// Inline stylesheet for Flow execution UI widgets.
pub const FLOW_CSS: &str = r#"
.r-flow-dash{font-family:system-ui,-apple-system,sans-serif;color:#e8eaed;background:linear-gradient(145deg,#0f1419 0%,#1a2332 100%);border:1px solid #2d3a4f;border-radius:12px;padding:1.25rem;margin:1rem 0}
.r-flow-dash__header{display:flex;flex-wrap:wrap;align-items:center;justify-content:space-between;gap:.75rem;margin-bottom:1.25rem}
.r-flow-dash__title{margin:0;font-size:1.1rem;font-weight:600;letter-spacing:-.02em}
.r-flow-dash__badge{font-size:.7rem;padding:.2rem .55rem;border-radius:999px;background:#1e3a2f;color:#6ee7b7;border:1px solid #065f46}
.r-flow-dash__badge--warn{background:#3d2e12;color:#fcd34d;border-color:#92400e}
.r-flow-dash__badge--err{background:#3f1d1d;color:#fca5a5;border-color:#991b1b}
.r-flow-dash__grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(140px,1fr));gap:.75rem;margin-bottom:1.25rem}
.r-flow-dash__stat{background:#0b1020;border:1px solid #243044;border-radius:8px;padding:.85rem}
.r-flow-dash__stat-label{font-size:.68rem;text-transform:uppercase;letter-spacing:.06em;color:#8b9cb3;margin:0 0 .35rem}
.r-flow-dash__stat-value{font-size:1.5rem;font-weight:700;margin:0;line-height:1.1}
.r-flow-dash__section{margin-top:1rem}
.r-flow-dash__section h3{margin:0 0 .6rem;font-size:.8rem;text-transform:uppercase;letter-spacing:.05em;color:#8b9cb3}
.r-flow-dash__chips{display:flex;flex-wrap:wrap;gap:.4rem}
.r-flow-dash__chip{font-size:.75rem;padding:.25rem .55rem;border-radius:6px;background:#1e293b;border:1px solid #334155;color:#cbd5e1}
.r-flow-dash__table{width:100%;border-collapse:collapse;font-size:.8rem}
.r-flow-dash__table th,.r-flow-dash__table td{padding:.45rem .5rem;text-align:left;border-bottom:1px solid #243044}
.r-flow-dash__table th{color:#8b9cb3;font-weight:500}
.r-flow-dash__bar{height:6px;background:#1e293b;border-radius:3px;overflow:hidden;margin-top:.25rem}
.r-flow-dash__bar span{display:block;height:100%;background:linear-gradient(90deg,#3b82f6,#6366f1);border-radius:3px}
.r-flow-exec{display:grid;grid-template-columns:1fr;gap:1rem;margin:1rem 0}
@media(min-width:900px){.r-flow-exec{grid-template-columns:1.2fr .8fr}}
.r-flow-exec__panel{background:#0f1419;border:1px solid #2d3a4f;border-radius:12px;padding:1rem}
.r-flow-exec__panel h3{margin:0 0 .75rem;font-size:.85rem;color:#8b9cb3;text-transform:uppercase;letter-spacing:.05em}
.r-flow-graph__track{display:flex;flex-wrap:wrap;align-items:center;gap:.5rem;min-height:2.5rem}
.r-flow-graph__node{font-size:.78rem;padding:.35rem .65rem;border-radius:8px;border:1px solid #334155;background:#1e293b;white-space:nowrap}
.r-flow-graph__node--running{border-color:#3b82f6;background:#172554;color:#93c5fd;animation:r-flow-pulse 1.5s ease-in-out infinite}
.r-flow-graph__node--done{border-color:#059669;background:#052e16;color:#6ee7b7}
.r-flow-graph__node--failed{border-color:#dc2626;background:#450a0a;color:#fca5a5}
.r-flow-graph__node--paused{border-color:#d97706;background:#451a03;color:#fcd34d}
.r-flow-graph__arrow{color:#475569;font-size:.7rem}
.r-flow-graph__status{margin:.5rem 0 0;font-size:.72rem;color:#64748b}
.r-event-stream-list{list-style:none;margin:0;padding:0;max-height:280px;overflow-y:auto;font-family:ui-monospace,monospace;font-size:.72rem}
.r-event-stream-list li{padding:.35rem .5rem;border-bottom:1px solid #1e293b;color:#94a3b8}
.r-event-stream-list li:nth-child(odd){background:#0b1020}
.r-worker-panel{display:flex;flex-wrap:wrap;gap:.5rem;margin:.75rem 0}
.r-worker-panel button{font-size:.78rem;padding:.4rem .85rem;border-radius:6px;border:1px solid #334155;background:#1e293b;color:#e2e8f0;cursor:pointer}
.r-worker-panel button:hover{background:#334155}
@keyframes r-flow-pulse{0%,100%{opacity:1}50%{opacity:.65}}
"#;

/// Emit Flow widget CSS once per page (place in layout or page head).
pub fn flow_styles() -> View {
    View::Element(Element {
        tag: "style".into(),
        attrs: vec![Attr {
            name: "data-r-flow-styles".into(),
            value: AttrValue::Static("true".into()),
        }],
        children: vec![Child::Text(FLOW_CSS.into())],
        dom_id: None,
    })
}
