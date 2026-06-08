use crate::actions::audit_greet;
use crate::audit_shell::{audit_page, demo_box, AuditStatus};
use resuma::prelude::*;
use serde::{Deserialize, Serialize};

#[component]
fn CounterDemo() -> View {
    let count = signal(0_i32);
    view! {
        <>
            <p>"Count: " {count}</p>
            <div class="row">
                <button class="btn" onClick={count.update(|c| *c -= 1)}>"-"</button>
                <button class="btn" onClick={count.update(|c| *c += 1)}>"+"</button>
                <button class="btn btn-ghost" onClick={count.set(0)}>"reset"</button>
            </div>
        </>
    }
}

#[component]
fn ShowDemo() -> View {
    let logged_in = signal(false);
    view! {
        <>
            <Show when={logged_in.get()}>
                <p>"Welcome back!"</p>
            </Show>
            <Show when={!logged_in.get()} fallback={view! { <span class="pill">"Sign in to continue"</span> }}>
                <span></span>
            </Show>
            <button class="btn" onClick={logged_in.update(|v| *v = !*v)}>
                <Show when={logged_in.get()}>
                    <span>"Logout"</span>
                </Show>
                <Show when={!logged_in.get()}>
                    <span>"Login"</span>
                </Show>
            </button>
        </>
    }
}

#[component]
fn EffectsDemo() -> View {
    let first = signal("Ada".to_string());
    let last = signal("Lovelace".to_string());
    let display = signal("Ada Lovelace".to_string());
    effect!([first, last, display], move || {
        display.set(format!("{} {}", first.get(), last.get()));
    });
    view! {
        <>
            <div class="row">
                <input placeholder="First" onInput={
                    js! { state.first.set(event.target.value); }
                } />
                <input placeholder="Last" onInput={
                    js! { state.last.set(event.target.value); }
                } />
            </div>
            <p>"Full name: " {display}</p>
        </>
    }
}

#[component]
fn StoreDemo() -> View {
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    struct Ui {
        theme: String,
        count: i32,
    }
    let ui = use_store(Ui {
        theme: "dark".into(),
        count: 0,
    });
    let label = signal("Theme: dark · Count: 0".to_string());
    view! {
        <>
            <p>{label}</p>
            <button class="btn" onClick={
                js! {
                    state.ui.update(s => { s.count += 1; });
                    const u = state.ui.value;
                    state.label.set("Theme: " + u.theme + " · Count: " + u.count);
                }
            }>"Increment store"</button>
        </>
    }
}

#[data]
struct Locale {
    lang: String,
}

static LOCALE: resuma::ContextId<Locale> = resuma::ContextId::new();

#[component]
fn LocaleProvider() -> View {
    provide_context(&LOCALE, Locale { lang: "es".into() });
    view! { <LocaleConsumer /> }
}

#[component]
fn LocaleConsumer() -> View {
    let locale = use_context(&LOCALE);
    view! { <p>"Context locale: " {locale.lang.clone()}</p> }
}

#[component]
fn ServerActionDemo() -> View {
    let result = signal(String::new());
    view! {
        <>
            <div class="row">
                <button class="btn" onClick={
                    js! {
                        const r = await __resuma.action("audit_echo", ["Hello from audit"]);
                        state.result.set(r);
                    }
                }>"Call audit_echo"</button>
                <button class="btn btn-ghost" onClick={
                    js! {
                        const sum = await __resuma.action("audit_add", [2, 40]);
                        state.result.set("2 + 40 = " + sum);
                    }
                }>"Call audit_add(2,40)"</button>
            </div>
            <p>{result}</p>
        </>
    }
}

#[component]
fn JsDemo() -> View {
    let msg = signal(String::new());
    view! {
        <>
            <input placeholder="Type here" onInput={
                js! { state.msg.set(event.target.value); }
            } />
            <p>"js! input: " {msg}</p>
        </>
    }
}

#[component]
fn ErrorDemo() -> View {
    let boom = signal(false);
    view! {
        <>
            <Show when={boom.get()} fallback={view! {
                <p>"All good — click to trigger error boundary"</p>
            }}>
                {resuma::error_boundary(Err::<View, &str>("Something broke!"), |e| {
                    view! { <p class="pill">{e}</p> }
                })}
            </Show>
            <button class="btn btn-ghost" onClick={boom.set(true)}>"Trigger error"</button>
        </>
    }
}

#[component]
fn HandlersDemo() -> View {
    let clicked = signal(0);
    view! {
        <>
            <p>"Clicks: " {clicked}</p>
            <button class="btn" onClick={clicked.update(|c| *c += 1)}>"Click me"</button>
        </>
    }
}

#[island]
fn IslandDemo() -> View {
    let n = signal(0);
    view! {
        <>
            <p>"Island counter: " {n}</p>
            <button class="btn" onClick={n.update(|v| *v += 1)}>"+"</button>
        </>
    }
}

#[component]
fn SlotsDemo() -> View {
    view! {
        <div class="demo-box">
            <Slot name="header" />
            <Slot />
        </div>
    }
}

pub fn index(_req: FlowRequest) -> View {
    audit_page(
        "Components Overview",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components",
        vec![Child::View(view! {
            <p>"See sub-routes for each component feature. All demos are interactive."</p>
        })],
    )
}

pub fn view(_req: FlowRequest) -> View {
    audit_page(
        "view!",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/view",
        vec![demo_box(
            "Counter with view!",
            vec![Child::View(
                CounterDemo::render(CounterDemoProps::default()),
            )],
        )],
    )
}

pub fn control_flow(_req: FlowRequest) -> View {
    audit_page(
        "Control Flow",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/control_flow",
        vec![demo_box(
            "Show + toggle",
            vec![Child::View(ShowDemo::render(ShowDemoProps::default()))],
        )],
    )
}

pub fn signals(_req: FlowRequest) -> View {
    audit_page(
        "Signals",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/signals",
        vec![demo_box(
            "signal()",
            vec![Child::View(
                CounterDemo::render(CounterDemoProps::default()),
            )],
        )],
    )
}

pub fn effects(_req: FlowRequest) -> View {
    audit_page(
        "Effects",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/effects",
        vec![demo_box(
            "use_computed()",
            vec![Child::View(
                EffectsDemo::render(EffectsDemoProps::default()),
            )],
        )],
    )
}

pub fn error_boundary(_req: FlowRequest) -> View {
    audit_page(
        "Error Boundaries",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/error_boundary",
        vec![demo_box(
            "error_boundary()",
            vec![Child::View(ErrorDemo::render(ErrorDemoProps::default()))],
        )],
    )
}

pub fn handlers(_req: FlowRequest) -> View {
    audit_page(
        "Handlers",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/handlers",
        vec![demo_box(
            "onClick",
            vec![Child::View(HandlersDemo::render(
                HandlersDemoProps::default(),
            ))],
        )],
    )
}

pub fn islands(_req: FlowRequest) -> View {
    audit_page(
        "Islands",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/islands",
        vec![demo_box("#[island]", vec![Child::View(IslandDemo())])],
    )
}

pub fn client(_req: FlowRequest) -> View {
    audit_page(
        "Client (TypeScript)",
        AuditStatus::Info,
        "https://resuma-docs.fly.dev/docs/components/client",
        vec![Child::View(view! {
            <p>"TypeScript client SDK in " <code>"client-sdk/"</code> ". Handlers compile via rs2js — tested via js! demos."</p>
        })],
    )
}

pub fn server(_req: FlowRequest) -> View {
    audit_page(
        "Server Actions",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/server",
        vec![demo_box(
            "#[server]",
            vec![Child::View(ServerActionDemo::render(
                ServerActionDemoProps::default(),
            ))],
        )],
    )
}

pub fn js(_req: FlowRequest) -> View {
    audit_page(
        "js!",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/js",
        vec![demo_box(
            "js! handlers",
            vec![Child::View(JsDemo::render(JsDemoProps::default()))],
        )],
    )
}

pub fn slots(_req: FlowRequest) -> View {
    audit_page(
        "Slots",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/slots",
        vec![Child::View(view! {
            <SlotsDemo>
                <h4 slot="header">"Header slot"</h4>
                <p>"Default slot body"</p>
            </SlotsDemo>
        })],
    )
}

pub fn nav_link(_req: FlowRequest) -> View {
    audit_page(
        "NavLink",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/nav_link",
        vec![Child::View(view! {
            <div class="row">
                <NavLink href="/audit/components/signals" activeClass="active">"Signals"</NavLink>
                <NavLink href="/audit/components/form" activeClass="active">"Form"</NavLink>
                <NavLink href="/audit/flow/loaders" activeClass="active">"Loaders"</NavLink>
            </div>
        })],
    )
}

#[component]
fn GreetFormDemo() -> View {
    use_visible_task(
        r#"(async (state, __resuma) => {
            const form = document.querySelector('[data-audit-greet-form]');
            const out = document.querySelector('[data-audit-greet-result]');
            if (!form || !out) return;
            const show = (msg) => {
                out.textContent = msg;
                out.style.display = msg ? '' : 'none';
            };
            const orig = window.fetch.bind(window);
            window.fetch = async (...args) => {
                const res = await orig(...args);
                const url = String(args[0]?.url || args[0] || '');
                if (url.includes('/_resuma/submit/audit_greet')) {
                    const data = await res.clone().json().catch(() => null);
                    if (data?.ok && data.value?.message) show(data.value.message);
                    else if (!data?.ok) show(data?.error || 'Submit failed');
                }
                return res;
            };
        })"#,
    );
    view! {
        <>
            <Form submit={audit_greet} data-audit-greet-form="true">
                <label>"Name" <input name="name" type="text" required=true /></label>
                <button type="submit" class="btn">"Greet via #[submit]"</button>
            </Form>
            <p class="pill" data-audit-greet-result="true" style="display:none"></p>
        </>
    }
}

pub fn form(_req: FlowRequest) -> View {
    audit_page(
        "Form",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/form",
        vec![demo_box(
            "#[submit]",
            vec![Child::View(GreetFormDemo::render(
                GreetFormDemoProps::default(),
            ))],
        )],
    )
}

pub fn greet_form_demo() -> Child {
    Child::View(GreetFormDemo::render(GreetFormDemoProps::default()))
}

pub fn store(_req: FlowRequest) -> View {
    audit_page(
        "Store",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/store",
        vec![demo_box(
            "use_store()",
            vec![Child::View(StoreDemo::render(StoreDemoProps::default()))],
        )],
    )
}

pub fn context(_req: FlowRequest) -> View {
    audit_page(
        "Context",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/context",
        vec![demo_box(
            "ContextId",
            vec![Child::View(LocaleProvider::render(
                LocaleProviderProps::default(),
            ))],
        )],
    )
}

pub fn tasks(_req: FlowRequest) -> View {
    audit_page(
        "Tasks",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/tasks",
        vec![Child::View(view! {
            <>
                <p>"use_visible_task runs client-side after mount — full demo on todo + platform pages."</p>
                <div class="row">
                    <NavLink href="/audit/security/todo" activeClass="active">"Todo task →"</NavLink>
                    <NavLink href="/audit/flow/platform" activeClass="active">"Platform task →"</NavLink>
                </div>
            </>
        })],
    )
}

#[component]
fn AccessibilityDemo() -> View {
    let count = signal(0_i32);
    view! {
        <>
            <button
                class="btn"
                type="button"
                aria-label="Increment accessible counter"
                data-audit-a11y-btn="true"
                onClick={count.update(|c| *c += 1)}
            >
                "+1"
            </button>
            <p
                role="status"
                aria-live="polite"
                aria-atomic="true"
                data-audit-a11y-status="true"
            >
                "Count: " {count}
            </p>
            <p id="audit-a11y-help" class="pill">"Use Tab to focus the button; screen readers announce count changes."</p>
        </>
    }
}

pub fn accessibility(_req: FlowRequest) -> View {
    audit_page(
        "Accessibility",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/accessibility",
        vec![demo_box(
            "aria-label · role=status · aria-live",
            vec![Child::View(AccessibilityDemo::render(
                AccessibilityDemoProps::default(),
            ))],
        )],
    )
}

pub fn clipboard(_req: FlowRequest) -> View {
    use_visible_task(
        r#"(async (state, __resuma) => {
            const input = document.querySelector('[data-audit-clip-input]');
            const copyBtn = document.querySelector('[data-audit-clip-copy]');
            const pasteBtn = document.querySelector('[data-audit-clip-paste]');
            const status = document.querySelector('[data-audit-clip-status]');
            copyBtn?.addEventListener('click', async () => {
                const text = input?.value || 'Resuma audit';
                try {
                    await navigator.clipboard.writeText(text);
                    status.textContent = 'Copied: ' + text;
                } catch (e) {
                    status.textContent = 'Clipboard denied';
                }
            });
            pasteBtn?.addEventListener('click', async () => {
                try {
                    const text = await navigator.clipboard.readText();
                    if (input) input.value = text;
                    status.textContent = 'Pasted: ' + text.slice(0, 40);
                } catch (e) {
                    status.textContent = 'Paste denied';
                }
            });
        })"#,
    );
    audit_page(
        "Clipboard",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/js",
        vec![demo_box(
            "navigator.clipboard",
            vec![Child::View(view! {
                <>
                    <input type="text" value="Resuma audit" data-audit-clip-input="true" />
                    <div class="row">
                        <button type="button" class="btn" data-audit-clip-copy="true">"Copy"</button>
                        <button type="button" class="btn btn-ghost" data-audit-clip-paste="true">"Paste"</button>
                    </div>
                    <p class="pill" data-audit-clip-status="true">"Ready"</p>
                </>
            })],
        )],
    )
}

pub fn picker(_req: FlowRequest) -> View {
    use_visible_task(
        r#"(async (state, __resuma) => {
            const sel = document.querySelector('[data-audit-picker]');
            const out = document.querySelector('[data-audit-picker-out]');
            sel?.addEventListener('change', () => {
                out.textContent = 'Selected: ' + sel.value;
            });
        })"#,
    );
    audit_page(
        "Native Select",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/form",
        vec![demo_box(
            "<select> picker",
            vec![Child::View(view! {
                <>
                    <label>
                        "Framework "
                        <select data-audit-picker="true">
                            <option value="resuma">"Resuma"</option>
                            <option value="next">"Next.js"</option>
                            <option value="remix">"Remix"</option>
                        </select>
                    </label>
                    <p class="pill" data-audit-picker-out="true">"Selected: resuma"</p>
                </>
            })],
        )],
    )
}

pub fn testing(_req: FlowRequest) -> View {
    audit_page(
        "Testing",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/testing",
        vec![Child::View(view! {
            <>
                <p>"Rust tests: " <code>"cargo test -p example-resuma-audit"</code></p>
                <p>"Browser audit: " <code>"scripts/audit_interactive.py"</code></p>
                <NavLink href="/audit/reference/registry" activeClass="active">"Test registry →"</NavLink>
                {" · "}
                <NavLink href="/audit/reference/matrix" activeClass="active">"Audit matrix →"</NavLink>
            </>
        })],
    )
}
