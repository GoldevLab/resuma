//! Shared audit UI — nav, status badges, page wrapper.

use resuma::prelude::*;

#[derive(Clone, Copy)]
pub enum AuditStatus {
    Pass,
    Demo,
    Info,
    Skip,
}

impl AuditStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pass => "✓ PASS",
            Self::Demo => "◉ DEMO",
            Self::Info => "ℹ INFO",
            Self::Skip => "○ SKIP",
        }
    }

    pub fn class(self) -> &'static str {
        match self {
            Self::Pass => "badge-pass",
            Self::Demo => "badge-demo",
            Self::Info => "badge-info",
            Self::Skip => "badge-skip",
        }
    }
}

pub struct AuditSection {
    pub title: &'static str,
    pub href: &'static str,
}

pub const SECTIONS: &[(&str, &[AuditSection])] = &[
    (
        "Introduction",
        &[
            AuditSection {
                title: "Getting Started",
                href: "/audit/intro/getting_started",
            },
            AuditSection {
                title: "Benchmark",
                href: "/audit/intro/benchmark",
            },
            AuditSection {
                title: "Examples",
                href: "/audit/intro/examples",
            },
            AuditSection {
                title: "Project structure",
                href: "/audit/intro/project_structure",
            },
            AuditSection {
                title: "FAQ",
                href: "/audit/intro/faq",
            },
        ],
    ),
    (
        "Security",
        &[
            AuditSection {
                title: "Overview",
                href: "/audit/security",
            },
            AuditSection {
                title: "Configure server",
                href: "/audit/security/configure",
            },
            AuditSection {
                title: "Server actions",
                href: "/audit/security/server_actions",
            },
            AuditSection {
                title: "Auth middleware",
                href: "/audit/security/middleware",
            },
            AuditSection {
                title: "Authorization & RLS",
                href: "/audit/security/authorization",
            },
            AuditSection {
                title: "Backend patterns",
                href: "/audit/security/backend_patterns",
            },
            AuditSection {
                title: "Todo showcase",
                href: "/audit/security/todo",
            },
        ],
    ),
    (
        "Components",
        &[
            AuditSection {
                title: "Overview",
                href: "/audit/components",
            },
            AuditSection {
                title: "view!",
                href: "/audit/components/view",
            },
            AuditSection {
                title: "Control flow",
                href: "/audit/components/control_flow",
            },
            AuditSection {
                title: "Signals",
                href: "/audit/components/signals",
            },
            AuditSection {
                title: "Effects",
                href: "/audit/components/effects",
            },
            AuditSection {
                title: "Error boundaries",
                href: "/audit/components/error_boundary",
            },
            AuditSection {
                title: "Handlers",
                href: "/audit/components/handlers",
            },
            AuditSection {
                title: "Islands",
                href: "/audit/components/islands",
            },
            AuditSection {
                title: "Client (TypeScript)",
                href: "/audit/components/client",
            },
            AuditSection {
                title: "Server actions",
                href: "/audit/components/server",
            },
            AuditSection {
                title: "js!",
                href: "/audit/components/js",
            },
            AuditSection {
                title: "Slots",
                href: "/audit/components/slots",
            },
            AuditSection {
                title: "NavLink",
                href: "/audit/components/nav_link",
            },
            AuditSection {
                title: "Form",
                href: "/audit/components/form",
            },
            AuditSection {
                title: "Store",
                href: "/audit/components/store",
            },
            AuditSection {
                title: "Context",
                href: "/audit/components/context",
            },
            AuditSection {
                title: "Accessibility",
                href: "/audit/components/accessibility",
            },
            AuditSection {
                title: "Clipboard",
                href: "/audit/components/clipboard",
            },
            AuditSection {
                title: "Native select",
                href: "/audit/components/picker",
            },
            AuditSection {
                title: "Tasks",
                href: "/audit/components/tasks",
            },
            AuditSection {
                title: "Testing",
                href: "/audit/components/testing",
            },
        ],
    ),
    (
        "Resuma Flow",
        &[
            AuditSection {
                title: "Overview",
                href: "/audit/flow",
            },
            AuditSection {
                title: "Routing",
                href: "/audit/flow/routing",
            },
            AuditSection {
                title: "Query params",
                href: "/audit/flow/query_params",
            },
            AuditSection {
                title: "Pages",
                href: "/audit/flow/pages",
            },
            AuditSection {
                title: "Layouts",
                href: "/audit/flow/layouts",
            },
            AuditSection {
                title: "Loaders",
                href: "/audit/flow/loaders",
            },
            AuditSection {
                title: "Actions",
                href: "/audit/flow/actions",
            },
            AuditSection {
                title: "Middleware",
                href: "/audit/flow/middleware",
            },
            AuditSection {
                title: "Endpoints",
                href: "/audit/flow/endpoints",
            },
            AuditSection {
                title: "Error handling",
                href: "/audit/flow/errors",
            },
            AuditSection {
                title: "Caching",
                href: "/audit/flow/caching",
            },
            AuditSection {
                title: "Streaming",
                href: "/audit/flow/streaming",
            },
            AuditSection {
                title: "Prefetch",
                href: "/audit/flow/prefetch",
            },
            AuditSection {
                title: "Platform select",
                href: "/audit/flow/platform",
            },
            AuditSection {
                title: "Pointer / touch",
                href: "/audit/flow/gestures",
            },
            AuditSection {
                title: "User presence",
                href: "/audit/flow/user_presence",
            },
            AuditSection {
                title: "PWA & public/",
                href: "/audit/flow/pwa",
            },
        ],
    ),
    (
        "Cookbook",
        &[
            AuditSection {
                title: "Overview",
                href: "/audit/cookbook",
            },
            AuditSection {
                title: "Debouncer",
                href: "/audit/cookbook/debouncer",
            },
            AuditSection {
                title: "Portals",
                href: "/audit/cookbook/portals",
            },
            AuditSection {
                title: "View transitions",
                href: "/audit/cookbook/view_transitions",
            },
            AuditSection {
                title: "Theme",
                href: "/audit/cookbook/theme",
            },
            AuditSection {
                title: "Streaming loaders",
                href: "/audit/cookbook/streaming_loaders",
            },
            AuditSection {
                title: "PRG pattern",
                href: "/audit/cookbook/prg",
            },
            AuditSection {
                title: "Loader invalidation",
                href: "/audit/cookbook/loader_invalidation",
            },
            AuditSection {
                title: "Virtual list",
                href: "/audit/cookbook/virtual_list",
            },
            AuditSection {
                title: "Image list",
                href: "/audit/cookbook/image_list",
            },
            AuditSection {
                title: "Animations",
                href: "/audit/cookbook/animations",
            },
            AuditSection {
                title: "Drag & drop",
                href: "/audit/cookbook/drag_drop",
            },
            AuditSection {
                title: "Docker deploy",
                href: "/audit/cookbook/docker",
            },
        ],
    ),
    (
        "Integrations",
        &[
            AuditSection {
                title: "Overview",
                href: "/audit/integrations",
            },
            AuditSection {
                title: "SQLx",
                href: "/audit/integrations/sqlx",
            },
            AuditSection {
                title: "Turso",
                href: "/audit/integrations/turso",
            },
            AuditSection {
                title: "Supabase",
                href: "/audit/integrations/supabase",
            },
            AuditSection {
                title: "Auth",
                href: "/audit/integrations/auth",
            },
            AuditSection {
                title: "Validation",
                href: "/audit/integrations/validator",
            },
            AuditSection {
                title: "i18n",
                href: "/audit/integrations/i18n",
            },
            AuditSection {
                title: "Tailwind",
                href: "/audit/integrations/tailwind",
            },
            AuditSection {
                title: "OG Image",
                href: "/audit/integrations/og_image",
            },
            AuditSection {
                title: "E2E testing",
                href: "/audit/integrations/e2e",
            },
            AuditSection {
                title: "Network status",
                href: "/audit/integrations/network",
            },
            AuditSection {
                title: "localStorage",
                href: "/audit/integrations/storage",
            },
        ],
    ),
    (
        "Reference",
        &[
            AuditSection {
                title: "Architecture",
                href: "/audit/reference/architecture",
            },
            AuditSection {
                title: "Reactivity internals",
                href: "/audit/reference/reactivity",
            },
            AuditSection {
                title: "Package",
                href: "/audit/reference/package",
            },
            AuditSection {
                title: "CLI",
                href: "/audit/reference/cli",
            },
            AuditSection {
                title: "Audit matrix",
                href: "/audit/reference/matrix",
            },
            AuditSection {
                title: "Test registry",
                href: "/audit/reference/registry",
            },
            AuditSection {
                title: "API reference",
                href: "/audit/reference/api",
            },
        ],
    ),
];

pub fn audit_page(title: &str, status: AuditStatus, doc_href: &str, body: Vec<Child>) -> View {
    view! {
        <article class="audit-card">
            <header class="audit-header">
                <div>
                    <h1>{title.to_string()}</h1>
                    <a class="doc-link" href={doc_href.to_string()} target="_blank">"📄 Docs →"</a>
                </div>
                <span class={format!("badge {}", status.class())}>{status.label()}</span>
            </header>
            <section class="audit-body">
                {View::fragment(body)}
            </section>
        </article>
    }
}

pub fn demo_box(title: &str, children: Vec<Child>) -> Child {
    Child::View(view! {
        <div class="demo-box">
            <h3>{title.to_string()}</h3>
            {View::fragment(children)}
        </div>
    })
}

pub fn status_list(items: &[(&str, AuditStatus)]) -> Child {
    Child::View(view! {
        <ul class="status-list">
            {items.iter().map(|(label, st)| {
                view! {
                    <li>
                        <span class={format!("badge {}", st.class())}>{st.label()}</span>
                        " " {(*label).to_string()}
                    </li>
                }
            }).collect::<Vec<_>>()}
        </ul>
    })
}

pub const CSS: &str = r#"<style>
* { box-sizing: border-box; }
body { font-family: ui-sans-serif, system-ui, sans-serif; background: #0b1020; color: #e6e8ee; margin: 0; min-height: 100vh; line-height: 1.5; }
.shell { max-width: 56rem; margin: 0 auto; padding: 1.5rem 1rem 3rem; }
.nav { display: flex; flex-wrap: wrap; gap: .75rem; margin-bottom: 1.5rem; padding-bottom: 1rem; border-bottom: 1px solid #2a2f4a; }
.nav a { color: #b9bfd2; text-decoration: none; font-size: .9rem; }
.nav a.active, .nav a:hover { color: #818cf8; }
.hero { margin-bottom: 2rem; }
.hero h1 { margin: 0 0 .5rem; font-size: 1.75rem; }
.hero p { color: #b9bfd2; margin: 0; max-width: 52ch; }
.grid { display: grid; gap: 1rem; grid-template-columns: repeat(auto-fill, minmax(14rem, 1fr)); }
.section-card { background: #14182b; border: 1px solid #2a2f4a; border-radius: 12px; padding: 1rem 1.1rem; }
.section-card h2 { margin: 0 0 .65rem; font-size: .95rem; color: #818cf8; }
.section-card ul { list-style: none; padding: 0; margin: 0; }
.section-card li { margin: .3rem 0; }
.section-card a { color: #b9bfd2; text-decoration: none; font-size: .85rem; }
.section-card a:hover { color: #e6e8ee; }
.audit-card { background: #14182b; border: 1px solid #2a2f4a; border-radius: 12px; padding: 1.5rem; }
.audit-header { display: flex; justify-content: space-between; align-items: flex-start; gap: 1rem; margin-bottom: 1.25rem; flex-wrap: wrap; }
.audit-header h1 { margin: 0; font-size: 1.35rem; }
.doc-link { font-size: .85rem; color: #818cf8; text-decoration: none; }
.doc-link:hover { text-decoration: underline; }
.badge { display: inline-block; font-size: .72rem; font-weight: 700; padding: .2rem .5rem; border-radius: 999px; letter-spacing: .03em; }
.badge-pass { background: #14532d; color: #86efac; }
.badge-demo { background: #1e3a5f; color: #93c5fd; }
.badge-info { background: #422006; color: #fcd34d; }
.badge-skip { background: #374151; color: #9ca3af; }
.demo-box { background: #0b1020; border: 1px dashed #2a2f4a; border-radius: 8px; padding: 1rem; margin: 1rem 0; }
.demo-box h3 { margin: 0 0 .75rem; font-size: .9rem; color: #818cf8; }
.audit-body p { color: #b9bfd2; }
.audit-body code { background: #0b1020; padding: .1rem .35rem; border-radius: 4px; font-size: .88em; }
.btn { background: #6366f1; color: white; border: 0; border-radius: 8px; padding: .45rem .85rem; font-weight: 600; cursor: pointer; font-size: .88rem; }
.btn-themed { background: var(--resuma-primary, #6366f1); color: var(--resuma-fg, #fff); border: 1px solid color-mix(in srgb, var(--resuma-primary, #6366f1) 60%, transparent); }
.btn-themed:hover { filter: brightness(1.08); }
[data-audit-theme-panel] { border-radius: 8px; padding: 1rem; transition: background .2s, color .2s; }
[data-audit-theme-panel].theme-dark { background: #0b1020; color: #e6e8ee; }
[data-audit-theme-panel].theme-light { background: #f8fafc; color: #0f172a; }
.btn:hover { background: #818cf8; }
.btn-ghost { background: transparent; border: 1px solid #2a2f4a; color: inherit; }
.row { display: flex; gap: .5rem; flex-wrap: wrap; align-items: center; }
input, select, textarea { background: #0b1020; border: 1px solid #2a2f4a; color: inherit; border-radius: 8px; padding: .45rem .65rem; font: inherit; }
.status-list { list-style: none; padding: 0; }
.status-list li { margin: .4rem 0; display: flex; align-items: center; gap: .5rem; }
.pill { display: inline-block; background: #1e293b; padding: .15rem .5rem; border-radius: 999px; font-size: .82rem; }
#modals { position: fixed; inset: 0; pointer-events: none; z-index: 50; }
.modal-backdrop { pointer-events: auto; position: fixed; inset: 0; background: rgba(0,0,0,.6); display: grid; place-items: center; }
.modal { background: #14182b; border: 1px solid #2a2f4a; border-radius: 12px; padding: 1.5rem; min-width: 16rem; }
.resuma-error { border: 1px solid #7f1d1d; background: #450a0a; padding: 1rem; border-radius: 8px; }
.resuma-field-error { color: #f87171; font-size: .85rem; display: block; margin-top: .25rem; }
.resuma-stream-loading { color: #b9bfd2; font-style: italic; padding: 1rem 0; }
.todo-list { list-style: none; padding: 0; margin: .75rem 0; }
.todo-list li { padding: .35rem 0; border-bottom: 1px solid #2a2f4a; }
.matrix-wrap { overflow-x: auto; margin: 1rem 0; }
.matrix-table { width: 100%; border-collapse: collapse; font-size: .85rem; }
.matrix-table th, .matrix-table td { border: 1px solid #2a2f4a; padding: .45rem .6rem; text-align: left; vertical-align: top; }
.matrix-table th { background: #0b1020; color: #818cf8; }
.matrix-table .tick.yes { color: #86efac; }
.matrix-table .tick.no { color: #6b7280; }
.muted { color: #9ca3af; font-size: .82rem; }
.platform-tabs a { padding: .25rem .55rem; border: 1px solid #2a2f4a; border-radius: 6px; }
.platform-panel { border-radius: 8px; padding: 1rem; margin-top: .5rem; }
.platform-desktop { background: #1e293b; }
.platform-mobile { background: #312e81; max-width: 20rem; }
.platform-pwa { background: #14532d; border: 1px dashed #86efac; }
.vlist { border: 1px solid #2a2f4a; border-radius: 8px; background: #0b1020; }
.vlist-viewport { height: 240px; overflow-y: auto; overflow-x: hidden; position: relative; }
.vlist-inner { position: relative; width: 100%; min-height: 1px; }
.vlist-row {
  position: absolute; left: 0; right: 0; width: 100%; height: 32px;
  display: flex; align-items: center; padding: 0 .75rem;
  border-bottom: 1px solid #1e293b; box-sizing: border-box;
  font-variant-numeric: tabular-nums;
}
.vlist-row:nth-child(even) { background: rgba(99,102,241,.06); }
.vlist-meta { margin-top: .65rem; }
.registry { display: grid; grid-template-columns: minmax(14rem, 22rem) 1fr; gap: 1rem; min-height: 22rem; }
.registry-sidebar { background: #0b1020; border: 1px solid #2a2f4a; border-radius: 8px; padding: .75rem; display: flex; flex-direction: column; gap: .5rem; }
.registry-sidebar input[type=search] { width: 100%; box-sizing: border-box; }
.registry-cats { display: flex; flex-wrap: wrap; gap: .25rem; }
.registry-cats button { font-size: .72rem; padding: .2rem .45rem; border-radius: 6px; border: 1px solid #2a2f4a; background: transparent; color: inherit; cursor: pointer; }
.registry-cats button.active { background: #312e81; border-color: #6366f1; }
.registry-list { list-style: none; padding: 0; margin: 0; overflow-y: auto; max-height: 28rem; }
.registry-list li { display: flex; justify-content: space-between; align-items: center; gap: .5rem; padding: .4rem .5rem; border-radius: 6px; cursor: pointer; font-size: .85rem; }
.registry-list li:hover, .registry-list li.selected { background: #1e293b; }
.reg-badge { font-size: .65rem; font-weight: 700; padding: .1rem .35rem; border-radius: 999px; }
.reg-pass { background: #14532d; color: #86efac; }
.reg-fail { background: #7f1d1d; color: #fca5a5; }
.reg-pending { background: #422006; color: #fcd34d; }
.reg-na { background: #374151; color: #9ca3af; }
.registry-detail { background: #0b1020; border: 1px dashed #2a2f4a; border-radius: 8px; padding: 1rem; }
.registry-detail h3 { margin: 0 0 .5rem; }
.anim-box { width: 6rem; height: 6rem; display: grid; place-items: center; background: #312e81; border-radius: 12px; font-weight: 700; margin-bottom: .75rem; }
.anim-box.anim-pulse { animation: audit-pulse 1s ease-in-out infinite alternate; }
@keyframes audit-pulse { from { transform: scale(1); opacity: 1; } to { transform: scale(1.06); opacity: .85; } }
.dnd-list { list-style: none; padding: 0; margin: 0; }
.dnd-list li { padding: .55rem .75rem; margin: .35rem 0; background: #1e293b; border: 1px solid #2a2f4a; border-radius: 8px; cursor: grab; }
.dnd-list li.dragging { opacity: .5; }
.img-vlist .img-viewport { height: 280px; }
.img-row { position: absolute; left: 0; right: 0; height: 88px; display: flex; align-items: center; gap: .75rem; padding: .35rem .75rem; border-bottom: 1px solid #1e293b; box-sizing: border-box; }
.img-row img { border-radius: 6px; object-fit: cover; background: #1e293b; }
.gesture-pad { min-height: 8rem; border: 2px dashed #6366f1; border-radius: 12px; display: grid; place-items: center; touch-action: none; user-select: none; background: #1e293b; }
.net-online { background: #14532d; color: #86efac; }
.net-offline { background: #7f1d1d; color: #fca5a5; }
.pattern-diagram { background: #0b1020; padding: 1rem; border-radius: 8px; font-size: .78rem; line-height: 1.35; overflow-x: auto; }
.filters-mini .btn.active { background: #312e81; border-color: #6366f1; }
.btn-xs { font-size: .75rem; padding: .15rem .4rem; }
.todo-row { display: flex; align-items: center; gap: .35rem; flex-wrap: wrap; }
:focus-visible { outline: 2px solid #818cf8; outline-offset: 2px; }
</style>"#;
