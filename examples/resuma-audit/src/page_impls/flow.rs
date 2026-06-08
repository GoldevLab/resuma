use crate::actions::use_audit_delayed_load;
use crate::audit_shell::{audit_page, demo_box, AuditStatus};
use crate::page_impls::components;
use crate::platform;
use resuma::prelude::*;

pub fn index(_req: FlowRequest) -> View {
    audit_page(
        "Resuma Flow Overview",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/flow",
        vec![Child::View(view! {
            <p>"File-based routing, #[load], #[submit], layouts, middleware — all in one crate."</p>
        })],
    )
}

pub fn routing(_req: FlowRequest) -> View {
    audit_page(
        "Routing",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/flow/routing",
        vec![Child::View(view! {
            <>
                <p>"This app uses " <code>"src/pages/"</code> " file-based routes:"</p>
                <ul>
                    <li><code>"index.rs"</code>" → /"</li>
                    <li><code>"audit/flow/loaders.rs"</code>" → /audit/flow/loaders"</li>
                    <li><code>"audit/flow/users/[id].rs"</code>" → /audit/flow/users/:id"</li>
                </ul>
                <NavLink href="/audit/flow/users/42" activeClass="active">"Try dynamic route →"</NavLink>
            </>
        })],
    )
}

pub fn query_params(req: FlowRequest) -> View {
    let q = req.query_param("q").unwrap_or("");
    let data = match try_use_load::<crate::actions::SearchData>("audit_search") {
        Ok(d) => d,
        Err(e) => return error_page(&FlowError::Loader(e)),
    };
    audit_page(
        "Query Params",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/flow/query_params",
        vec![
            Child::View(view! {
                <form method="get" action="/audit/flow/query_params" class="row">
                    <input type="search" name="q" value={q.to_string()} placeholder="Search (min 2 chars)" />
                    <button type="submit" class="btn">"Search"</button>
                </form>
            }),
            demo_box(
                "Loader results",
                vec![Child::View(view! {
                    <>
                        <p>"Query: " {data.query.clone()}</p>
                        <ul class="todo-list">
                            {data.results.iter().map(|r| view! { <li>{r.clone()}</li> }).collect::<Vec<_>>()}
                        </ul>
                    </>
                })],
            ),
        ],
    )
}

pub fn pages(_req: FlowRequest) -> View {
    audit_page(
        "Pages",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/flow/pages",
        vec![Child::View(view! {
            <p>"Each " <code>"pub fn page(req: FlowRequest) -> View"</code> " in " <code>"src/pages/"</code> " is auto-wired via PagesRegistry."</p>
        })],
    )
}

pub fn layouts(_req: FlowRequest) -> View {
    audit_page(
        "Layouts",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/flow/layouts",
        vec![Child::View(view! {
            <>
                <p>"Root layout in main.rs uses " <code>"#[layout(\"/\")]"</code> " with " <code>"<Slot />"</code>". You see the nav on every page."</p>
            </>
        })],
    )
}

pub fn loaders(_req: FlowRequest) -> View {
    let cached = match try_use_load::<crate::actions::CachedData>("audit_cached") {
        Ok(d) => d,
        Err(e) => return error_page(&FlowError::Loader(e)),
    };
    audit_page(
        "Loaders",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/flow/loaders",
        vec![demo_box(
            "#[load] with cache",
            vec![Child::View(view! {
                <>
                    <p>{cached.value.clone()}</p>
                    <p class="pill">{"Loaded at: "}{cached.timestamp.clone()}</p>
                </>
            })],
        )],
    )
}

pub fn actions(_req: FlowRequest) -> View {
    audit_page(
        "Actions (Submit)",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/flow/submits",
        vec![demo_box("#[submit]", vec![components::greet_form_demo()])],
    )
}

pub fn middleware(_req: FlowRequest) -> View {
    audit_page(
        "Middleware",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/flow/middleware",
        vec![Child::View(view! {
            <p>"Check server logs — " <code>"#[middleware] audit_log"</code> " prints every request."</p>
        })],
    )
}

pub fn endpoints(_req: FlowRequest) -> View {
    audit_page(
        "Endpoints",
        AuditStatus::Info,
        "https://resuma-docs.fly.dev/docs/flow/endpoints",
        vec![Child::View(view! {
            <>
                <ul>
                    <li><code>"POST /_resuma/action/:name"</code>" — #[server]"</li>
                    <li><code>"POST /_resuma/submit/:name"</code>" — #[submit]"</li>
                    <li><code>"GET /_resuma/handler/:name.js"</code>" — lazy handlers"</li>
                </ul>
            </>
        })],
    )
}

pub fn errors(_req: FlowRequest) -> View {
    audit_page(
        "Error Handling",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/flow/errors",
        vec![Child::View(view! {
            <>
                <p>"Try loading a missing user:"</p>
                <NavLink href="/audit/flow/users/404" activeClass="active">"/audit/flow/users/404"</NavLink>
            </>
        })],
    )
}

pub fn caching(_req: FlowRequest) -> View {
    let data = match try_use_load::<crate::actions::CachedData>("audit_cached") {
        Ok(d) => d,
        Err(e) => return error_page(&FlowError::Loader(e)),
    };
    audit_page(
        "Caching",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/flow/caching",
        vec![Child::View(view! {
            <>
                <p>"Loader cached with " <code>"cache = \"public, max-age=60\""</code>"."</p>
                <p class="pill">{data.timestamp.clone()}</p>
                <p>"Refresh within 60s — timestamp should stay the same (browser cache)."</p>
            </>
        })],
    )
}

pub fn streaming(_req: FlowRequest) -> View {
    let body = match use_audit_delayed_load() {
        LoadValue::Pending => view! {
            {stream_slot("audit_delayed")}
        },
        LoadValue::Ok(msg) => view! { <p>{msg.clone()}</p> },
        LoadValue::Err(e) => return error_page(&FlowError::Loader(e)),
    };
    audit_page(
        "Streaming",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/flow/streaming",
        vec![demo_box("#[load(stream)]", vec![Child::View(body)])],
    )
}

pub fn prefetch(_req: FlowRequest) -> View {
    audit_page(
        "Prefetch",
        AuditStatus::Demo,
        "https://resuma-docs.fly.dev/docs/flow/prefetch",
        vec![Child::View(view! {
            <>
                <p>"NavLink prefetches on viewport — hover links in nav to trigger prefetch."</p>
                <NavLink href="/audit/flow/loaders" activeClass="active">"Prefetch target →"</NavLink>
            </>
        })],
    )
}

pub fn pwa(_req: FlowRequest) -> View {
    audit_page(
        "PWA & public/",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/flow/pwa",
        vec![Child::View(view! {
            <>
                <p>"FlowApp enables PWA by default. Static files from " <code>"public/"</code>":"</p>
                <ul>
                    <li><a href="/audit-badge.svg">"/audit-badge.svg"</a></li>
                </ul>
            </>
        })],
    )
}

pub fn platform(req: FlowRequest) -> View {
    let target = platform::detect(&req);
    let layout = platform::platform_select(
        target,
        "Multi-column desktop layout".to_string(),
        Some("Single-column mobile layout".to_string()),
        Some("PWA standalone shell layout".to_string()),
        "Default layout".to_string(),
    );
    let touch_hint = platform::platform_select(
        target,
        "Mouse / keyboard primary".to_string(),
        Some("Touch-first targets (44px)".to_string()),
        Some("Installable — offline-ready shell".to_string()),
        "Standard web".to_string(),
    );
    audit_page(
        "Platform Select",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/flow/pwa",
        vec![
            Child::View(view! {
                <>
                    <p>"Active target: " <code>{target.label().to_string()}</code></p>
                    <div class="row platform-tabs">
                        <NavLink href="/audit/flow/platform" activeClass="active">"desktop"</NavLink>
                        <NavLink href="/audit/flow/platform?platform=mobile" activeClass="active">"mobile"</NavLink>
                        <NavLink href="/audit/flow/platform?platform=pwa" activeClass="active">"pwa"</NavLink>
                    </div>
                </>
            }),
            demo_box(
                "platform_select()",
                vec![Child::View(view! {
                    <div class={format!("platform-panel {}", target.css_class())} data-audit-platform-panel="true">
                        <p><strong>{layout}</strong></p>
                        <p>{touch_hint}</p>
                    </div>
                })],
            ),
        ],
    )
}

pub fn dynamic_user(req: FlowRequest) -> View {
    let profile = match try_use_load::<std::result::Result<crate::actions::UserProfile, LoaderError>>(
        "audit_user",
    ) {
        Ok(Ok(p)) => p,
        Ok(Err(e)) | Err(e) => return error_page(&FlowError::Loader(e)),
    };
    let id = req.param("id").unwrap_or("?");
    audit_page(
        &format!("User :{id}"),
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/flow/routing",
        vec![Child::View(view! {
            <>
                <p>"Dynamic param " <code>{format!("id={id}")}</code></p>
                <p>"Name: " {profile.name.clone()}</p>
            </>
        })],
    )
}

pub fn gestures(_req: FlowRequest) -> View {
    use_visible_task(
        r#"(async (state, __resuma) => {
            const pad = document.querySelector('[data-audit-gesture-pad]');
            const out = document.querySelector('[data-audit-gesture-out]');
            if (!pad) return;
            const show = (type, x, y) => {
                out.textContent = type + ' @ ' + Math.round(x) + ',' + Math.round(y);
            };
            pad.addEventListener('pointerdown', (e) => show('pointerdown', e.offsetX, e.offsetY));
            pad.addEventListener('pointerup', (e) => show('pointerup', e.offsetX, e.offsetY));
            pad.addEventListener('touchstart', (e) => {
                const t = e.touches[0];
                const r = pad.getBoundingClientRect();
                show('touch', t.clientX - r.left, t.clientY - r.top);
            }, { passive: true });
        })"#,
    );
    audit_page(
        "Pointer & Touch",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/flow/pwa",
        vec![
            Child::View(view! {
                <NavLink href="/audit/flow/platform?platform=mobile" activeClass="active">"Mobile platform layout →"</NavLink>
            }),
            demo_box(
                "Gesture pad",
                vec![Child::View(view! {
                    <>
                        <div class="gesture-pad" data-audit-gesture-pad="true" tabindex="0">
                            "Tap or click here"
                        </div>
                        <p class="pill" data-audit-gesture-out="true">"Waiting…"</p>
                    </>
                })],
            ),
        ],
    )
}

pub fn user_presence(_req: FlowRequest) -> View {
    use_visible_task(
        r#"(async (state, __resuma) => {
            const vis = document.querySelector('[data-audit-presence-vis]');
            const idle = document.querySelector('[data-audit-presence-idle]');
            let timer = null;
            let idleSec = 0;
            const resetIdle = () => {
                idleSec = 0;
                idle.textContent = 'Idle: 0s';
            };
            ['mousemove', 'keydown', 'click', 'scroll'].forEach(ev => {
                document.addEventListener(ev, resetIdle, { passive: true });
            });
            document.addEventListener('visibilitychange', () => {
                vis.textContent = document.hidden ? 'Tab: hidden' : 'Tab: visible';
            });
            vis.textContent = document.hidden ? 'Tab: hidden' : 'Tab: visible';
            timer = setInterval(() => {
                idleSec += 1;
                idle.textContent = 'Idle: ' + idleSec + 's';
            }, 1000);
        })"#,
    );
    audit_page(
        "User Presence",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/flow/pwa",
        vec![demo_box(
            "visibilitychange + idle timer",
            vec![Child::View(view! {
                <>
                    <p class="pill" data-audit-presence-vis="true">"Tab: visible"</p>
                    <p class="pill" data-audit-presence-idle="true">"Idle: 0s"</p>
                    <p class="muted">"Switch tabs or wait without input to observe changes."</p>
                </>
            })],
        )],
    )
}
