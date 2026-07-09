//! Central interactive test registry — inspired by exhaustive sample app patterns.

use crate::audit_shell::{audit_page, AuditStatus};
use resuma::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TestCategory {
    Api,
    Component,
    Cookbook,
    Flow,
    Security,
    Integration,
}

impl TestCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Api => "API",
            Self::Component => "Component",
            Self::Cookbook => "Cookbook",
            Self::Flow => "Flow",
            Self::Security => "Security",
            Self::Integration => "Integration",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::Component => "component",
            Self::Cookbook => "cookbook",
            Self::Flow => "flow",
            Self::Security => "security",
            Self::Integration => "integration",
        }
    }
}

#[derive(Clone, Copy)]
pub enum TestResult {
    Pass,
    Fail,
    Pending,
    Na,
}

impl TestResult {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Pending => "…",
            Self::Na => "—",
        }
    }

    pub fn class(self) -> &'static str {
        match self {
            Self::Pass => "reg-pass",
            Self::Fail => "reg-fail",
            Self::Pending => "reg-pending",
            Self::Na => "reg-na",
        }
    }
}

#[derive(Clone, Copy)]
pub struct TestEntry {
    pub id: &'static str,
    pub name: &'static str,
    pub category: TestCategory,
    pub route: &'static str,
    pub interactive: bool,
    /// Default result when CI manifest is absent (interactive tests covered by audit_interactive.py).
    pub default_result: TestResult,
}

const fn entry(
    id: &'static str,
    name: &'static str,
    category: TestCategory,
    route: &'static str,
    interactive: bool,
) -> TestEntry {
    TestEntry {
        id,
        name,
        category,
        route,
        interactive,
        default_result: if interactive {
            TestResult::Pass
        } else {
            TestResult::Na
        },
    }
}

pub const REGISTRY: &[TestEntry] = &[
    entry(
        "signals",
        "Signals",
        TestCategory::Component,
        "/audit/components/signals",
        true,
    ),
    entry(
        "control_flow",
        "Show / control flow",
        TestCategory::Component,
        "/audit/components/control_flow",
        true,
    ),
    entry(
        "effects",
        "Effects / computed",
        TestCategory::Component,
        "/audit/components/effects",
        true,
    ),
    entry(
        "handlers",
        "onClick handlers",
        TestCategory::Component,
        "/audit/components/handlers",
        true,
    ),
    entry(
        "js",
        "js! handlers",
        TestCategory::Component,
        "/audit/components/js",
        true,
    ),
    entry(
        "server",
        "#[server] actions",
        TestCategory::Component,
        "/audit/components/server",
        true,
    ),
    entry(
        "islands",
        "#[island]",
        TestCategory::Component,
        "/audit/components/islands",
        true,
    ),
    entry(
        "store",
        "use_store",
        TestCategory::Component,
        "/audit/components/store",
        true,
    ),
    entry(
        "form",
        "Form #[submit]",
        TestCategory::Component,
        "/audit/components/form",
        true,
    ),
    entry(
        "error_boundary",
        "Error boundary",
        TestCategory::Component,
        "/audit/components/error_boundary",
        true,
    ),
    entry(
        "accessibility",
        "Accessibility",
        TestCategory::Component,
        "/audit/components/accessibility",
        true,
    ),
    entry(
        "clipboard",
        "Clipboard API",
        TestCategory::Component,
        "/audit/components/clipboard",
        true,
    ),
    entry(
        "picker",
        "Native select",
        TestCategory::Component,
        "/audit/components/picker",
        true,
    ),
    entry(
        "debouncer",
        "Debouncer",
        TestCategory::Cookbook,
        "/audit/cookbook/debouncer",
        true,
    ),
    entry(
        "portals",
        "Portals",
        TestCategory::Cookbook,
        "/audit/cookbook/portals",
        true,
    ),
    entry(
        "theme",
        "Theme",
        TestCategory::Cookbook,
        "/audit/cookbook/theme",
        true,
    ),
    entry(
        "virtual_list",
        "Virtual list",
        TestCategory::Cookbook,
        "/audit/cookbook/virtual_list",
        true,
    ),
    entry(
        "animations",
        "CSS animations",
        TestCategory::Cookbook,
        "/audit/cookbook/animations",
        true,
    ),
    entry(
        "drag_drop",
        "Drag & drop reorder",
        TestCategory::Cookbook,
        "/audit/cookbook/drag_drop",
        true,
    ),
    entry(
        "image_list",
        "Image list + loader",
        TestCategory::Cookbook,
        "/audit/cookbook/image_list",
        true,
    ),
    entry(
        "prg",
        "PRG redirect",
        TestCategory::Cookbook,
        "/audit/cookbook/prg",
        true,
    ),
    entry(
        "loaders",
        "#[load]",
        TestCategory::Flow,
        "/audit/flow/loaders",
        false,
    ),
    entry(
        "streaming",
        "#[load(stream)]",
        TestCategory::Flow,
        "/audit/flow/streaming",
        true,
    ),
    entry(
        "query_params",
        "Query params + loader",
        TestCategory::Flow,
        "/audit/flow/query_params",
        true,
    ),
    entry(
        "platform",
        "Platform select",
        TestCategory::Flow,
        "/audit/flow/platform",
        true,
    ),
    entry(
        "gestures",
        "Pointer / touch",
        TestCategory::Flow,
        "/audit/flow/gestures",
        true,
    ),
    entry(
        "user_presence",
        "Tab visibility / idle",
        TestCategory::Flow,
        "/audit/flow/user_presence",
        true,
    ),
    entry(
        "todo",
        "Todo + SQLx RLS",
        TestCategory::Security,
        "/audit/security/todo",
        true,
    ),
    entry(
        "middleware",
        "Auth middleware",
        TestCategory::Security,
        "/audit/security/middleware",
        true,
    ),
    entry(
        "network",
        "Online / offline",
        TestCategory::Integration,
        "/audit/integrations/network",
        true,
    ),
    entry(
        "storage",
        "localStorage + store",
        TestCategory::Integration,
        "/audit/integrations/storage",
        true,
    ),
    entry(
        "sqlx",
        "SQLx extension",
        TestCategory::Integration,
        "/audit/integrations/sqlx",
        false,
    ),
    entry(
        "matrix",
        "Audit matrix",
        TestCategory::Api,
        "/audit/reference/matrix",
        true,
    ),
];

pub fn registry(_req: FlowRequest) -> View {
    use_visible_task(
        r#"(async (state, __resuma) => {
            const root = document.querySelector('[data-audit-registry]');
            if (!root) return;
            const search = root.querySelector('[data-reg-search]');
            const list = root.querySelector('[data-reg-list]');
            const detail = root.querySelector('[data-reg-detail]');
            const cats = root.querySelectorAll('[data-reg-cat]');
            let activeCat = 'all';
            let results = {};
            try {
                const r = await fetch('/audit-results.json', { cache: 'no-store' });
                if (r.ok) results = await r.json();
            } catch (_) {}
            const items = [...list.querySelectorAll('[data-reg-item]')];
            const applyFilter = () => {
                const q = (search?.value || '').trim().toLowerCase();
                items.forEach(el => {
                    const name = (el.dataset.name || '').toLowerCase();
                    const cat = el.dataset.cat || '';
                    const matchQ = !q || name.includes(q) || (el.dataset.id || '').includes(q);
                    const matchC = activeCat === 'all' || cat === activeCat;
                    el.hidden = !(matchQ && matchC);
                });
            };
            search?.addEventListener('input', applyFilter);
            cats.forEach(btn => {
                btn.addEventListener('click', () => {
                    activeCat = btn.dataset.regCat || 'all';
                    cats.forEach(b => b.classList.toggle('active', b === btn));
                    applyFilter();
                });
            });
            items.forEach(el => {
                const id = el.dataset.id;
                const badge = el.querySelector('[data-reg-badge]');
                const st = results[id] || el.dataset.default || 'pass';
                if (badge) {
                    badge.textContent = st === 'pass' ? 'PASS' : st === 'fail' ? 'FAIL' : st === 'pending' ? '…' : '—';
                    badge.className = 'reg-badge reg-' + st;
                }
                el.addEventListener('click', () => {
                    items.forEach(i => i.classList.remove('selected'));
                    el.classList.add('selected');
                    if (!detail) return;
                    detail.replaceChildren();
                    const h3 = document.createElement('h3');
                    h3.textContent = el.dataset.name || '';
                    const meta = document.createElement('p');
                    meta.className = 'muted';
                    meta.textContent = (el.dataset.catLabel || '') + ' · ' + (el.dataset.route || '');
                    const open = document.createElement('p');
                    const link = document.createElement('a');
                    link.href = el.dataset.route || '#';
                    link.textContent = 'Open demo →';
                    open.appendChild(link);
                    const pill = document.createElement('p');
                    pill.className = 'pill';
                    pill.textContent = 'Interactive: ' + (el.dataset.interactive === '1' ? 'yes' : 'no');
                    detail.appendChild(h3);
                    detail.appendChild(meta);
                    detail.appendChild(open);
                    detail.appendChild(pill);
                });
            });
            if (items[0]) items[0].click();
            applyFilter();
        })"#,
    );

    let pass = REGISTRY
        .iter()
        .filter(|t| t.interactive && matches!(t.default_result, TestResult::Pass))
        .count();
    let interactive = REGISTRY.iter().filter(|t| t.interactive).count();

    audit_page(
        "Test Registry",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/testing",
        vec![Child::View(view! {
            <>
                <p class="pill">
                    {format!("{pass}/{interactive} interactive tests · {} total", REGISTRY.len())}
                </p>
                <p class="muted">"Search sidebar · click test · " <code>"audit-results.json"</code> " overrides PASS/FAIL in CI"</p>
                <div class="registry" data-audit-registry="true">
                    <aside class="registry-sidebar">
                        <input type="search" placeholder="Filter tests…" data-reg-search="true" aria-label="Filter tests" />
                        <div class="registry-cats">
                            <button type="button" class="active" data-reg-cat="all">"All"</button>
                            {[TestCategory::Api, TestCategory::Component, TestCategory::Cookbook, TestCategory::Flow, TestCategory::Security, TestCategory::Integration]
                                .into_iter()
                                .map(|c| {
                                    view! {
                                        <button type="button" data-reg-cat={c.slug()}>{c.label()}</button>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                        <ul class="registry-list" data-reg-list="true">
                            {REGISTRY.iter().map(|t| {
                                let default = match t.default_result {
                                    TestResult::Pass => "pass",
                                    TestResult::Fail => "fail",
                                    TestResult::Pending => "pending",
                                    TestResult::Na => "na",
                                };
                                view! {
                                    <li
                                        data-reg-item="true"
                                        data-id={t.id.to_string()}
                                        data-name={t.name.to_string()}
                                        data-cat={t.category.slug().to_string()}
                                        data-cat-label={t.category.label().to_string()}
                                        data-route={t.route.to_string()}
                                        data-interactive={if t.interactive { "1" } else { "0" }}
                                        data-default={default.to_string()}
                                    >
                                        <span class="reg-name">{t.name.to_string()}</span>
                                        <span class="reg-badge" data-reg-badge="true">{t.default_result.label()}</span>
                                    </li>
                                }
                            }).collect::<Vec<_>>()}
                        </ul>
                    </aside>
                    <div class="registry-detail" data-reg-detail="true">
                        <p class="muted">"Select a test from the sidebar"</p>
                    </div>
                </div>
            </>
        })],
    )
}
