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
.r-flow-dash__bar svg{display:block;width:100%;height:6px}
.r-flow-dash__bar-fill{fill:#6366f1;height:100%}
.r-flow-exec{display:grid;grid-template-columns:1fr;gap:1.15rem;margin:1rem 0}
@media(min-width:900px){.r-flow-exec{grid-template-columns:1.15fr .85fr;align-items:start}}
.r-flow-exec__side{display:flex;flex-direction:column;gap:1rem;min-width:0}
.r-flow-exec__panel{background:#0f1419;border:1px solid #2d3a4f;border-radius:14px;padding:1.15rem 1.2rem}
.r-flow-exec__panel h3{margin:0 0 .85rem;font-size:.8rem;color:#8b9cb3;text-transform:uppercase;letter-spacing:.06em;font-weight:700}
.r-flow-graph__track{display:flex;flex-wrap:wrap;align-items:center;gap:.55rem;min-height:2.75rem;padding:.15rem 0}
.r-flow-graph__node{font-size:.78rem;padding:.4rem .7rem;border-radius:999px;border:1px solid #334155;background:#1e293b;white-space:nowrap}
.r-flow-graph__node--running{border-color:#3b82f6;background:#172554;color:#93c5fd;animation:r-flow-pulse 1.5s ease-in-out infinite}
.r-flow-graph__node--done{border-color:#059669;background:#052e16;color:#6ee7b7}
.r-flow-graph__node--failed{border-color:#dc2626;background:#450a0a;color:#fca5a5}
.r-flow-graph__node--paused{border-color:#d97706;background:#451a03;color:#fcd34d}
.r-flow-graph__arrow{color:#475569;font-size:.7rem}
.r-flow-graph__status{margin:.65rem 0 0;font-size:.74rem;color:#64748b;line-height:1.45}
.r-event-stream{display:flex;flex-direction:column;min-height:0;gap:.5rem}
.r-event-stream__viewport{
  min-height:12rem;max-height:min(42vh,22rem);overflow-x:hidden;overflow-y:auto;overflow-anchor:none;
  padding:.65rem .75rem;border-radius:12px;border:1px solid #243044;
  background:rgba(11,16,32,.55);scroll-behavior:smooth;
  -webkit-overflow-scrolling:touch;
}
.r-event-stream__viewport:empty::before,
.r-event-stream__viewport:has(.r-event-stream-list:empty)::before{
  content:"Waiting for events…";display:block;padding:.5rem .25rem;color:#64748b;font-size:.75rem;font-style:italic;
}
.r-event-stream-list{list-style:none;margin:0;padding:0;font-family:ui-monospace,monospace;font-size:.72rem;line-height:1.45}
.r-event-stream-list li{
  padding:.42rem .55rem .42rem .7rem;margin:0 0 .28rem;border-radius:8px;
  border:1px solid #1e293b;color:#94a3b8;background:rgba(15,23,42,.35);
}
.r-event-stream-list li:last-child{margin-bottom:0;overflow-anchor:auto}
.r-worker-panel{display:flex;flex-direction:column;gap:.65rem;margin:.5rem 0 0}
.r-worker-panel__actions{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:.55rem}
@media(min-width:520px){.r-worker-panel__actions{grid-template-columns:repeat(4,minmax(0,1fr))}}
.r-worker-panel__status{margin:0;font-size:.75rem;color:#64748b;line-height:1.4;min-height:1.1rem}
.r-flow-control{
  display:inline-flex;align-items:center;justify-content:center;width:100%;
  font:inherit;font-size:.78rem;font-weight:600;line-height:1.2;
  padding:.52rem .85rem;border-radius:999px;cursor:pointer;
  border:1px solid #475569;
  background:linear-gradient(145deg,#1e293b 0%,#334155 100%);
  color:#f8fafc;
  box-shadow:0 4px 16px rgba(15,23,42,.18),inset 0 1px 0 rgba(255,255,255,.12);
  transition:transform .15s ease,box-shadow .15s ease,background .15s ease;
}
.r-flow-control:hover:not(:disabled){transform:translateY(-1px);box-shadow:0 8px 22px rgba(15,23,42,.22),inset 0 1px 0 rgba(255,255,255,.18)}
.r-flow-control:active:not(:disabled){transform:translateY(0)}
.r-flow-control:disabled{opacity:.42;cursor:not-allowed;transform:none}
.r-flow-control--ghost{
  background:rgba(30,41,59,.72);
  border-color:rgba(148,163,184,.45);
  color:#e2e8f0;
}
.r-flow-control--pause{border-color:rgba(96,165,250,.45)}
.r-flow-control--resume{border-color:rgba(52,211,153,.45)}
.r-flow-control--danger{
  border-color:rgba(248,113,113,.55);
  background:linear-gradient(145deg,#7f1d1d 0%,#991b1b 100%);
  color:#fff;
}
.r-flow-control--replay{border-color:rgba(196,181,253,.4)}
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
