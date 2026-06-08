use crate::audit_shell::{audit_page, demo_box, AuditStatus};
use crate::security;
use crate::todo_store::{self, Todo};
use resuma::prelude::*;

#[server]
async fn list_todos(req: &FlowRequest) -> Result<Vec<Todo>> {
    todo_store::list_for(req).await
}

#[server]
async fn add_todo(title: String, req: &FlowRequest) -> Result<Vec<Todo>> {
    todo_store::add(title, req).await
}

#[server]
async fn toggle_todo(id: u64, req: &FlowRequest) -> Result<Vec<Todo>> {
    todo_store::toggle(id, req).await
}

#[server]
async fn rename_todo(id: u64, title: String, req: &FlowRequest) -> Result<Vec<Todo>> {
    todo_store::rename(id, title, req).await
}

#[server]
async fn remove_todo(id: u64, req: &FlowRequest) -> Result<Vec<Todo>> {
    todo_store::remove(id, req).await
}

#[server]
async fn clear_done(req: &FlowRequest) -> Result<Vec<Todo>> {
    todo_store::clear_done(req).await
}

#[component]
fn TodoMini() -> View {
    let _todos = use_signal(Vec::<Todo>::new());
    let _title = use_signal(String::new());
    let _status = use_signal(String::new());

    use_visible_task(
        r#"(async (state, __resuma) => {
            let filter = 'all';
            let search = '';
            let listCache = [];
            const status = document.querySelector('[data-audit-todo-status]');
            const ul = document.querySelector('[data-audit-todos]');
            const searchEl = document.querySelector('[data-audit-todo-search]');
            const filterBtns = document.querySelectorAll('[data-audit-filter]');
            const render = (list) => {
                listCache = list;
                if (!ul) return;
                const q = search.trim().toLowerCase();
                const visible = list.filter(t => {
                    if (filter === 'active' && t.done) return false;
                    if (filter === 'done' && !t.done) return false;
                    if (q && !t.title.toLowerCase().includes(q)) return false;
                    return true;
                });
                if (!visible.length) {
                    ul.innerHTML = '<li class="muted">No tasks match</li>';
                    return;
                }
                ul.innerHTML = visible.map(t =>
                    '<li class="todo-row" data-id="' + t.id + '">' +
                    '<label><input type="checkbox" data-toggle="' + t.id + '" ' + (t.done ? 'checked' : '') + '/> ' +
                    '<span class="todo-title" data-title="' + t.id + '">' + t.title + '</span></label>' +
                    '<button type="button" class="btn btn-ghost btn-xs" data-rename="' + t.id + '">edit</button>' +
                    '<button type="button" class="btn btn-ghost btn-xs" data-remove="' + t.id + '">×</button></li>'
                ).join('');
                ul.querySelectorAll('[data-toggle]').forEach(el => {
                    el.addEventListener('change', async () => {
                        try {
                            const next = await __resuma.action('toggle_todo', [Number(el.dataset.toggle)]);
                            render(next);
                            status.textContent = 'Toggled';
                        } catch (e) { status.textContent = 'Toggle failed'; }
                    });
                });
                ul.querySelectorAll('[data-rename]').forEach(btn => {
                    btn.addEventListener('click', async () => {
                        const id = Number(btn.dataset.rename);
                        const span = ul.querySelector('[data-title="' + id + '"]');
                        const next = prompt('Rename task', span?.textContent || '');
                        if (!next?.trim()) return;
                        try {
                            const list = await __resuma.action('rename_todo', [id, next.trim()]);
                            render(list);
                            status.textContent = 'Renamed';
                        } catch (e) { status.textContent = 'Rename failed'; }
                    });
                });
                ul.querySelectorAll('[data-remove]').forEach(btn => {
                    btn.addEventListener('click', async () => {
                        try {
                            const next = await __resuma.action('remove_todo', [Number(btn.dataset.remove)]);
                            render(next);
                            status.textContent = 'Removed';
                        } catch (e) { status.textContent = 'Remove failed'; }
                    });
                });
            };
            filterBtns.forEach(btn => {
                btn.addEventListener('click', () => {
                    filter = btn.dataset.auditFilter || 'all';
                    filterBtns.forEach(b => b.classList.toggle('active', b === btn));
                    render(listCache);
                });
            });
            searchEl?.addEventListener('input', (e) => {
                search = e.target.value;
                render(listCache);
            });
            document.querySelector('[data-audit-clear-done]')?.addEventListener('click', async () => {
                try {
                    const next = await __resuma.action('clear_done', []);
                    render(next);
                    status.textContent = 'Cleared done';
                } catch (e) { status.textContent = 'Clear failed'; }
            });
            try {
                const list = await __resuma.action('list_todos', []);
                render(list);
                status.textContent = list.length + ' loaded';
            } catch (e) {
                status.textContent = 'Load failed';
            }
            const input = document.querySelector('[data-audit-title]');
            const addBtn = document.querySelector('[data-audit-add]');
            addBtn?.addEventListener('click', async () => {
                const t = String(input?.value || '').trim();
                if (!t) return;
                try {
                    const next = await __resuma.action('add_todo', [t]);
                    input.value = '';
                    render(next);
                    status.textContent = 'Added (' + next.length + ' total)';
                } catch (e) {
                    status.textContent = 'Add failed: ' + (e.message || e);
                }
            });
        })"#,
    );

    view! {
        <>
            <div class="row">
                <input placeholder="New task" data-audit-title="true" />
                <button class="btn" data-audit-add="true">"Add"</button>
            </div>
            <div class="row filters-mini">
                <button type="button" class="btn btn-ghost active" data-audit-filter="all">"All"</button>
                <button type="button" class="btn btn-ghost" data-audit-filter="active">"Active"</button>
                <button type="button" class="btn btn-ghost" data-audit-filter="done">"Done"</button>
                <button type="button" class="btn btn-ghost" data-audit-clear-done="true">"Clear done"</button>
            </div>
            <input type="search" placeholder="Search tasks…" data-audit-todo-search="true" />
            <p class="pill" data-audit-todo-status="true">"Loading…"</p>
            <ul class="todo-list" data-audit-todos="true"></ul>
        </>
    }
}

pub fn index(_req: FlowRequest) -> View {
    audit_page(
        "Security Overview",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/security",
        vec![Child::View(view! {
            <ul>
                <li>"CSRF tokens on actions/submits"</li>
                <li>"Rate limiting"</li>
                <li>"Security headers + CSP"</li>
                <li>"Session middleware"</li>
                <li>"Authorization guards"</li>
            </ul>
        })],
    )
}

pub fn configure(_req: FlowRequest) -> View {
    audit_page(
        "Configure Server",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/security/configure",
        vec![Child::View(view! {
            <>
                <p>"Active SecurityConfig:"</p>
                <ul>
                    <li>"csrf: true"</li>
                    <li>"origin_check: true"</li>
                    <li>"actions_per_minute: 90"</li>
                    <li>"submits_per_minute: 45"</li>
                    <li>"body_limit_bytes: 256KB"</li>
                </ul>
            </>
        })],
    )
}

pub fn server_actions(_req: FlowRequest) -> View {
    audit_page(
        "Server Actions Security",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/security/server_actions",
        vec![Child::View(view! {
            <p>"All #[server] calls go through action middleware with CSRF + rate limits. See security.rs."</p>
        })],
    )
}

pub fn middleware(_req: FlowRequest) -> View {
    use_visible_task(
        r#"(async (state, __resuma) => {
            document.querySelectorAll('[data-audit-user-btn]').forEach((btn) => {
                btn.addEventListener('click', () => {
                    const user = btn.getAttribute('data-user');
                    if (!user) return;
                    document.cookie = 'resuma_demo_user=' + user + '; path=/; SameSite=Strict';
                    location.reload();
                });
            });
        })"#,
    );
    audit_page(
        "Auth Middleware",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/security/middleware",
        vec![Child::View(view! {
            <>
                <p>"Demo users (cookie resuma_demo_user):"</p>
                <div class="row">
                    {security::demo_users().iter().map(|u| {
                        let user = (*u).to_string();
                        view! {
                            <button class="btn btn-ghost" data-audit-user-btn="true" data-user={user.clone()}>
                                {user}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
                <p>"Current session user affects todo visibility."</p>
            </>
        })],
    )
}

pub fn authorization(_req: FlowRequest) -> View {
    audit_page(
        "Authorization & RLS",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/security/authorization",
        vec![Child::View(view! {
            <>
                <p>"Row-level: todos filtered by owner_id. Admin (alice) sees all."</p>
                <p>"assert_owner() in todo_store enforces ownership on mutations."</p>
            </>
        })],
    )
}

pub fn backend_patterns(_req: FlowRequest) -> View {
    audit_page(
        "Backend Patterns",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/security/backend_patterns",
        vec![Child::View(view! {
            <>
                <pre class="pattern-diagram" aria-label="Request flow diagram">{"HTTP Request
    │
    ▼
┌─────────────┐     attach_session() · CSRF · rate limit
│  Middleware │ ──► security.rs
└──────┬──────┘
       │
       ▼
┌─────────────┐     thin handlers — no business logic
│ #[server]   │ ──► page_impls/security.rs · actions.rs
│ Controller  │
└──────┬──────┘
       │
       ▼
┌─────────────┐     validation DTOs · assert_owner · RLS
│   Service   │ ──► todo_store.rs · image_service.rs
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  SQLx pool  │ ──► db.rs · migrations/
└─────────────┘"}</pre>
                <ul>
                    <li><strong>"Guard"</strong>" — " <code>"attach_session()"</code> " in middleware"</li>
                    <li><strong>"ValidationPipe"</strong>" — " <code>"AddTodoInput"</code> " / " <code>"RenameTodoInput"</code></li>
                    <li><strong>"Service"</strong>" — " <code>"todo_store.rs"</code> ", " <code>"image_service.rs"</code></li>
                    <li><strong>"Controller"</strong>" — " <code>"#[server]"</code> " + " <code>"#[load]"</code> " in pages"</li>
                    <li><strong>"Store (client)"</strong>" — " <code>"use_store"</code> " / visible tasks for UI state"</li>
                </ul>
                <NavLink href="/audit/cookbook/image_list" activeClass="active">"Image list (service + loader) →"</NavLink>
            </>
        })],
    )
}

pub fn todo(_req: FlowRequest) -> View {
    audit_page(
        "Todo Showcase",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/security/todo",
        vec![
            demo_box(
                "Mini todo with #[server] + visible task",
                vec![Child::View(TodoMini::render(TodoMiniProps::default()))],
            ),
            Child::View(view! {
                <p>"Full demo: " <code>"cargo run -p example-todo"</code></p>
            }),
        ],
    )
}
