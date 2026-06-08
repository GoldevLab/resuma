use crate::actions::audit_prg;
use crate::audit_shell::{audit_page, demo_box, AuditStatus};
use resuma::prelude::*;

#[component]
fn DebounceDemo() -> View {
    let results_text = signal(String::new());
    view! {
        <>
            <input type="search" placeholder="Type 2+ chars…" onInput={
                js! {
                    const q = event.target.value;
                    clearTimeout(window.__auditDebounce);
                    window.__auditDebounce = setTimeout(() => {
                        state.results_text.set(
                            q.length >= 2 ? "Debounced result for '" + q + "'" : ""
                        );
                    }, 300);
                }
            } />
            <p>{results_text}</p>
        </>
    }
}

#[component]
fn ModalDemo() -> View {
    let open = signal(false);
    view! {
        <>
            <button class="btn" onClick={open.set(true)}>"Open modal"</button>
            <Show when={open}>
                {portal("modals", vec![Child::View(view! {
                    <div class="modal-backdrop" onClick={open.set(false)}>
                        <div class="modal" onClick={js! { event.stopPropagation(); }}>
                            <h3>"Portal Modal"</h3>
                            <p>"Rendered via portal into #modals"</p>
                            <button class="btn" onClick={open.set(false)}>"Close"</button>
                        </div>
                    </div>
                })])}
            </Show>
        </>
    }
}

#[component]
fn ThemeDemo() -> View {
    let dark = Theme {
        mode: "dark".into(),
        primary: "#818cf8".into(),
        background: "#0b1020".into(),
        foreground: "#e6e8ee".into(),
    };
    provide_theme(dark);

    view! {
        <div class="theme-dark" data-audit-theme-panel="true">
            <p data-audit-theme-mode="true">
                "Theme via provide_theme / theme_css_vars — mode: dark"
            </p>
            <button class="btn btn-themed" data-audit-theme-toggle="true">
                "Toggle theme"
            </button>
        </div>
    }
}

pub fn index(_req: FlowRequest) -> View {
    audit_page(
        "Cookbook Overview",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/cookbook",
        vec![Child::View(view! { <p>"Recipes for common patterns."</p> })],
    )
}

pub fn debouncer(_req: FlowRequest) -> View {
    audit_page(
        "Debouncer",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/cookbook/debouncer",
        vec![demo_box(
            "use_debounce",
            vec![Child::View(DebounceDemo::render(
                DebounceDemoProps::default(),
            ))],
        )],
    )
}

pub fn portals(_req: FlowRequest) -> View {
    audit_page(
        "Portals",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/cookbook/portals",
        vec![demo_box(
            "portal()",
            vec![Child::View(ModalDemo::render(ModalDemoProps::default()))],
        )],
    )
}

pub fn view_transitions(_req: FlowRequest) -> View {
    audit_page(
        "View Transitions",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/cookbook/view_transitions",
        vec![Child::View(with_view_transition(
            "audit-vt",
            vec![Child::View(view! {
                <>
                    <p>"with_view_transition wraps page content for CSS view transitions."</p>
                    <NavLink href="/audit/cookbook/theme" activeClass="active">"Navigate →"</NavLink>
                </>
            })],
        ))],
    )
}

pub fn theme(_req: FlowRequest) -> View {
    use_visible_task(
        r#"(async (state, __resuma) => {
            const panel = document.querySelector('[data-audit-theme-panel]');
            const label = document.querySelector('[data-audit-theme-mode]');
            const btn = document.querySelector('[data-audit-theme-toggle]');
            const darkVars = '--resuma-primary:#818cf8;--resuma-bg:#0b1020;--resuma-fg:#e6e8ee;';
            const lightVars = '--resuma-primary:#4f46e5;--resuma-bg:#f8fafc;--resuma-fg:#0f172a;';
            const apply = (mode) => {
                if (!panel || !label) return;
                panel.style.cssText = mode === 'light' ? lightVars : darkVars;
                panel.classList.toggle('theme-light', mode === 'light');
                panel.classList.toggle('theme-dark', mode !== 'light');
                label.textContent = 'Theme via provide_theme / theme_css_vars — mode: ' + mode;
            };
            btn?.addEventListener('click', () => {
                const next = panel?.classList.contains('theme-light') ? 'dark' : 'light';
                apply(next);
            });
        })"#,
    );
    audit_page(
        "Theme",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/cookbook/theme",
        vec![demo_box(
            "Theme",
            vec![Child::View(ThemeDemo::render(ThemeDemoProps::default()))],
        )],
    )
}

pub fn streaming_loaders(_req: FlowRequest) -> View {
    audit_page(
        "Streaming Loaders",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/cookbook/streaming_loaders",
        vec![Child::View(view! {
            <NavLink href="/audit/flow/streaming" activeClass="active">"See streaming demo →"</NavLink>
        })],
    )
}

pub fn prg(req: FlowRequest) -> View {
    let added = req.query_param("added").unwrap_or("");
    audit_page(
        "PRG Pattern",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/cookbook/prg",
        vec![
            Child::View(view! {
                <Form submit={audit_prg}>
                    <label>"Item" <input name="item" type="text" /></label>
                    <button type="submit" class="btn">"Create (PRG redirect)"</button>
                </Form>
            }),
            if !added.is_empty() {
                Child::View(view! { <p class="pill">{"Added: "}{added.to_string()}</p> })
            } else {
                Child::View(view! { <></> })
            },
        ],
    )
}

pub fn loader_invalidation(_req: FlowRequest) -> View {
    audit_page(
        "Loader Invalidation",
        AuditStatus::Demo,
        "https://resuma-docs.fly.dev/docs/cookbook/loader_invalidation",
        vec![Child::View(view! {
            <>
                <p>"After submit, navigate with new query to re-run loaders. PRG demo shows redirect pattern."</p>
                <NavLink href="/audit/cookbook/prg" activeClass="active">"PRG demo →"</NavLink>
            </>
        })],
    )
}

pub fn virtual_list(_req: FlowRequest) -> View {
    use_visible_task(
        r#"(async (state, __resuma) => {
            const TOTAL = 500;
            const ROW_H = 32;
            const items = Array.from({ length: TOTAL }, (_, i) => 'Row ' + (i + 1));
            const root = document.querySelector('[data-audit-vlist]');
            const viewport = document.querySelector('[data-audit-vlist-viewport]');
            const inner = document.querySelector('[data-audit-vlist-inner]');
            const meta = document.querySelector('[data-audit-vlist-meta]');
            if (!root || !viewport || !inner || !meta) return;
            inner.style.height = (TOTAL * ROW_H) + 'px';
            inner.style.position = 'relative';
            inner.style.width = '100%';
            const render = () => {
                const scrollTop = viewport.scrollTop;
                const viewH = viewport.clientHeight || 240;
                const start = Math.max(0, Math.floor(scrollTop / ROW_H) - 2);
                const visible = Math.ceil(viewH / ROW_H) + 4;
                const end = Math.min(TOTAL, start + visible);
                inner.replaceChildren();
                for (let i = start; i < end; i++) {
                    const row = document.createElement('div');
                    row.className = 'vlist-row';
                    row.textContent = items[i];
                    row.style.position = 'absolute';
                    row.style.left = '0';
                    row.style.right = '0';
                    row.style.width = '100%';
                    row.style.height = ROW_H + 'px';
                    row.style.top = (i * ROW_H) + 'px';
                    row.style.display = 'flex';
                    row.style.alignItems = 'center';
                    row.setAttribute('role', 'listitem');
                    inner.appendChild(row);
                }
                meta.textContent = 'Showing rows ' + (start + 1) + '–' + end + ' of ' + TOTAL + ' · scroll ' + Math.round(scrollTop) + 'px';
            };
            viewport.addEventListener('scroll', render, { passive: true });
            render();
        })"#,
    );
    audit_page(
        "Virtual List",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/cookbook/virtual_list",
        vec![demo_box(
            "Windowed rendering (500 rows)",
            vec![Child::View(view! {
                <>
                    <div class="vlist" data-audit-vlist="true">
                        <div class="vlist-viewport" data-audit-vlist-viewport="true" tabindex="0" role="list" aria-label="Virtual list demo">
                            <div class="vlist-inner" data-audit-vlist-inner="true"></div>
                        </div>
                    </div>
                    <p class="pill vlist-meta" data-audit-vlist-meta="true">"Scroll to window rows"</p>
                </>
            })],
        )],
    )
}

pub fn docker(_req: FlowRequest) -> View {
    audit_page(
        "Docker Deploy",
        AuditStatus::Info,
        "https://resuma-docs.fly.dev/docs/cookbook/docker",
        vec![Child::View(view! {
            <>
                <pre style="background:#0b1020;padding:1rem;border-radius:8px;font-size:.85rem">{"docker build -t resuma-audit .\ndocker run -p 3000:3000 -e RESUMA_ENV=production resuma-audit"}</pre>
            </>
        })],
    )
}

pub fn animations(_req: FlowRequest) -> View {
    use_visible_task(
        r#"(async (state, __resuma) => {
            const box = document.querySelector('[data-audit-anim-box]');
            const btn = document.querySelector('[data-audit-anim-toggle]');
            const label = document.querySelector('[data-audit-anim-status]');
            btn?.addEventListener('click', () => {
                const on = box?.classList.toggle('anim-pulse');
                label.textContent = on ? 'Animation: running' : 'Animation: paused';
            });
        })"#,
    );
    audit_page(
        "CSS Animations",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/cookbook/view_transitions",
        vec![demo_box(
            "@keyframes + View Transitions",
            vec![Child::View(view! {
                <>
                    <div class="anim-box" data-audit-anim-box="true">"Resuma"</div>
                    <div class="row">
                        <button type="button" class="btn" data-audit-anim-toggle="true">"Toggle pulse"</button>
                        <NavLink href="/audit/cookbook/view_transitions" activeClass="active">"View transitions →"</NavLink>
                    </div>
                    <p class="pill" data-audit-anim-status="true">"Animation: paused"</p>
                </>
            })],
        )],
    )
}

pub fn drag_drop(_req: FlowRequest) -> View {
    use_visible_task(
        r#"(async (state, __resuma) => {
            const list = document.querySelector('[data-audit-dnd-list]');
            const status = document.querySelector('[data-audit-dnd-status]');
            if (!list) return;
            let dragEl = null;
            const sync = () => {
                const order = [...list.querySelectorAll('[data-dnd-item]')].map(el => el.textContent.trim());
                status.textContent = 'Order: ' + order.join(' → ');
            };
            list.querySelectorAll('[data-dnd-item]').forEach(el => {
                el.draggable = true;
                el.addEventListener('dragstart', (e) => {
                    dragEl = el;
                    e.dataTransfer.effectAllowed = 'move';
                    el.classList.add('dragging');
                });
                el.addEventListener('dragend', () => {
                    el.classList.remove('dragging');
                    dragEl = null;
                    sync();
                });
                el.addEventListener('dragover', (e) => {
                    e.preventDefault();
                    if (!dragEl || dragEl === el) return;
                    const rect = el.getBoundingClientRect();
                    const after = e.clientY > rect.top + rect.height / 2;
                    list.insertBefore(dragEl, after ? el.nextSibling : el);
                });
            });
            sync();
        })"#,
    );
    audit_page(
        "Drag & Drop",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/cookbook",
        vec![demo_box(
            "HTML5 reorder list",
            vec![Child::View(view! {
                <>
                    <ul class="dnd-list" data-audit-dnd-list="true">
                        <li data-dnd-item="true">"Design API"</li>
                        <li data-dnd-item="true">"Ship SSR"</li>
                        <li data-dnd-item="true">"Resume on click"</li>
                    </ul>
                    <p class="pill" data-audit-dnd-status="true">"Drag rows to reorder"</p>
                </>
            })],
        )],
    )
}

pub fn image_list(req: FlowRequest) -> View {
    let data = match try_use_load::<crate::image_service::ImageListData>("audit_image_list") {
        Ok(d) => d,
        Err(e) => return error_page(&FlowError::Loader(e)),
    };
    let q = req.query_param("q").unwrap_or("");
    let items_json = serde_json::to_string(&data.items).unwrap_or_else(|_| "[]".into());

    use_visible_task(format!(
        r#"(async (state, __resuma) => {{
            const ITEMS = {items_json};
            const ROW_H = 88;
            const viewport = document.querySelector('[data-audit-img-viewport]');
            const inner = document.querySelector('[data-audit-img-inner]');
            const meta = document.querySelector('[data-audit-img-meta]');
            if (!viewport || !inner) return;
            inner.style.height = (ITEMS.length * ROW_H) + 'px';
            inner.style.position = 'relative';
            const render = () => {{
                const scrollTop = viewport.scrollTop;
                const viewH = viewport.clientHeight || 280;
                const start = Math.max(0, Math.floor(scrollTop / ROW_H) - 1);
                const end = Math.min(ITEMS.length, start + Math.ceil(viewH / ROW_H) + 3);
                inner.replaceChildren();
                for (let i = start; i < end; i++) {{
                    const item = ITEMS[i];
                    const row = document.createElement('div');
                    row.className = 'img-row';
                    row.style.top = (i * ROW_H) + 'px';
                    row.innerHTML = '<img loading="lazy" decoding="async" width="160" height="120" alt="' + item.title + '" src="' + item.thumb_url + '" /><span>' + item.title + '</span>';
                    inner.appendChild(row);
                }}
                meta.textContent = 'Showing ' + (start + 1) + '–' + end + ' of ' + ITEMS.length + ' (SSR loader + lazy images)';
            }};
            viewport.addEventListener('scroll', render, {{ passive: true }});
            render();
        }})"#
    ));

    audit_page(
        "Image List",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/cookbook/virtual_list",
        vec![
            Child::View(view! {
                <>
                    <p class="muted">"Architecture: " <code>"image_service.rs"</code> " (service) → " <code>"#[load]"</code> " → virtual viewport + lazy " <code>"<img>"</code></p>
                    <form method="get" action="/audit/cookbook/image_list" class="row">
                        <input type="search" name="q" value={q.to_string()} placeholder="Filter images…" />
                        <button type="submit" class="btn">"Search"</button>
                    </form>
                </>
            }),
            demo_box(
                &format!("{} images loaded via #[load]", data.items.len()),
                vec![Child::View(view! {
                    <>
                        <div class="vlist img-vlist" data-audit-img-list="true">
                            <div class="vlist-viewport img-viewport" data-audit-img-viewport="true" tabindex="0">
                                <div class="vlist-inner" data-audit-img-inner="true"></div>
                            </div>
                        </div>
                        <p class="pill vlist-meta" data-audit-img-meta="true">"Scroll to window images"</p>
                    </>
                })],
            ),
        ],
    )
}
