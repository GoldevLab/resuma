use crate::audit_shell::{audit_page, demo_box, AuditStatus};
use crate::db;
use crate::extensions_catalog::EXTENSIONS;
use resuma::prelude::*;

pub fn index(_req: FlowRequest) -> View {
    audit_page(
        "Integrations Overview",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/integrations",
        vec![
            Child::View(view! {
                <>
                    <p>"Resuma core stays lean. Optional capabilities ship as CLI extensions:"</p>
                    <p><code>"resuma add sqlx|turso|tailwind|…"</code></p>
                </>
            }),
            extensions_table(),
            Child::View(view! {
                <p>
                    <NavLink href="/audit/reference/matrix" activeClass="active">"Full audit matrix →"</NavLink>
                </p>
            }),
        ],
    )
}

fn extensions_table() -> Child {
    Child::View(view! {
        <div class="matrix-wrap">
            <table class="matrix-table">
                <thead>
                    <tr>
                        <th>"Extension"</th>
                        <th>"CLI"</th>
                        <th>"Env"</th>
                        <th>"Status"</th>
                    </tr>
                </thead>
                <tbody>
                    {EXTENSIONS.iter().map(|ext| {
                        view! {
                            <tr>
                                <td>
                                    <NavLink href={ext.audit_href.to_string()} activeClass="active">
                                        {ext.name.to_string()}
                                    </NavLink>
                                    <br />
                                    <span class="muted">{ext.summary.to_string()}</span>
                                </td>
                                <td><code>{ext.cli.to_string()}</code></td>
                                <td><code>{ext.env.unwrap_or("—").to_string()}</code></td>
                                <td>
                                    <span class={format!("badge {}", ext.status.class())}>
                                        {ext.status.label()}
                                    </span>
                                </td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
    })
}

pub fn sqlx(_req: FlowRequest) -> View {
    let (status, body) = match db::meta() {
        Some(meta) => (
            AuditStatus::Pass,
            view! {
                <>
                    <p>"SQLx SQLite pool active in this audit app."</p>
                    <ul>
                        <li><code>{meta.url_display.clone()}</code></li>
                        <li>{"Todos in DB: "}{meta.todo_count.to_string()}</li>
                        <li>"Migrations: " <code>"migrations/001_todos.sql"</code></li>
                    </ul>
                    <NavLink href="/audit/security/todo" activeClass="active">"Todo DB demo →"</NavLink>
                </>
            },
        ),
        None => (
            AuditStatus::Info,
            view! {
                <p class="pill">"Database not initialized — set DATABASE_URL"</p>
            },
        ),
    };
    audit_page(
        "SQLx",
        status,
        "https://resuma-docs.fly.dev/docs/integrations/sqlx",
        vec![Child::View(body)],
    )
}

pub fn turso(_req: FlowRequest) -> View {
    audit_page(
        "Turso",
        AuditStatus::Info,
        "https://resuma-docs.fly.dev/docs/integrations/turso",
        vec![Child::View(view! {
            <p>"CLI: " <code>"resuma add turso"</code> " · " <code>"TURSO_DATABASE_URL=file:local.db"</code></p>
        })],
    )
}

pub fn supabase(_req: FlowRequest) -> View {
    audit_page(
        "Supabase",
        AuditStatus::Info,
        "https://resuma-docs.fly.dev/docs/integrations/supabase",
        vec![Child::View(view! {
            <p>"Hosted Postgres + auth patterns documented. Requires Supabase project."</p>
        })],
    )
}

pub fn auth(_req: FlowRequest) -> View {
    audit_page(
        "Auth Integration",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/integrations/auth",
        vec![Child::View(view! {
            <NavLink href="/audit/security/middleware" activeClass="active">"Auth middleware demo →"</NavLink>
        })],
    )
}

pub fn validator(_req: FlowRequest) -> View {
    audit_page(
        "Validation",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/integrations/validator",
        vec![Child::View(view! {
            <NavLink href="/audit/components/form" activeClass="active">"Form validation demo →"</NavLink>
        })],
    )
}

pub fn i18n(_req: FlowRequest) -> View {
    audit_page(
        "i18n",
        AuditStatus::Info,
        "https://resuma-docs.fly.dev/docs/integrations/i18n",
        vec![Child::View(view! {
            <NavLink href="/audit/components/context" activeClass="active">"Context locale demo →"</NavLink>
        })],
    )
}

pub fn tailwind(_req: FlowRequest) -> View {
    audit_page(
        "Tailwind",
        AuditStatus::Info,
        "https://resuma-docs.fly.dev/docs/integrations/tailwind",
        vec![Child::View(view! {
            <p>"CLI: " <code>"resuma add tailwind"</code> " · this audit uses inline CSS."</p>
        })],
    )
}

pub fn og_image(_req: FlowRequest) -> View {
    audit_page(
        "OG Image",
        AuditStatus::Info,
        "https://resuma-docs.fly.dev/docs/integrations/og_image",
        vec![Child::View(view! {
            <p>"FlowApp.with_description() sets meta tags — view page source for og: tags."</p>
        })],
    )
}

pub fn e2e(_req: FlowRequest) -> View {
    audit_page(
        "E2E Testing",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/integrations/e2e",
        vec![Child::View(view! {
            <>
                <p>"Audit scripts: " <code>"scripts/audit_interactive.py"</code></p>
                <p>"CI manifest: " <code>"public/audit-results.json"</code></p>
                <p>"Example app: " <code>"cargo run -p example-e2e"</code></p>
                <NavLink href="/audit/reference/registry" activeClass="active">"Test registry →"</NavLink>
            </>
        })],
    )
}

pub fn network(_req: FlowRequest) -> View {
    use_visible_task(
        r#"(async (state, __resuma) => {
            const el = document.querySelector('[data-audit-network]');
            const sync = () => {
                el.textContent = navigator.onLine ? 'Online' : 'Offline';
                el.className = 'pill ' + (navigator.onLine ? 'net-online' : 'net-offline');
            };
            window.addEventListener('online', sync);
            window.addEventListener('offline', sync);
            sync();
        })"#,
    );
    audit_page(
        "Network Status",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/integrations",
        vec![demo_box(
            "navigator.onLine",
            vec![Child::View(view! {
                <>
                    <p class="pill net-online" data-audit-network="true">"Online"</p>
                    <p class="muted">"Toggle offline in DevTools → Network to test."</p>
                </>
            })],
        )],
    )
}

pub fn storage(_req: FlowRequest) -> View {
    use_visible_task(
        r#"(async (state, __resuma) => {
            const KEY = 'resuma_audit_store';
            const input = document.querySelector('[data-audit-store-input]');
            const save = document.querySelector('[data-audit-store-save]');
            const load = document.querySelector('[data-audit-store-load]');
            const out = document.querySelector('[data-audit-store-out]');
            const saved = localStorage.getItem(KEY);
            if (saved && input) input.value = saved;
            out.textContent = saved ? 'Stored: ' + saved : 'No value yet';
            save?.addEventListener('click', () => {
                const v = input?.value || '';
                localStorage.setItem(KEY, v);
                out.textContent = 'Stored: ' + v;
            });
            load?.addEventListener('click', () => {
                const v = localStorage.getItem(KEY) || '';
                if (input) input.value = v;
                out.textContent = 'Loaded: ' + (v || '(empty)');
            });
        })"#,
    );
    audit_page(
        "localStorage",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/store",
        vec![demo_box(
            "localStorage + use_store pattern",
            vec![Child::View(view! {
                <>
                    <input type="text" placeholder="Persist me…" data-audit-store-input="true" />
                    <div class="row">
                        <button type="button" class="btn" data-audit-store-save="true">"Save"</button>
                        <button type="button" class="btn btn-ghost" data-audit-store-load="true">"Load"</button>
                    </div>
                    <p class="pill" data-audit-store-out="true">"No value yet"</p>
                </>
            })],
        )],
    )
}
