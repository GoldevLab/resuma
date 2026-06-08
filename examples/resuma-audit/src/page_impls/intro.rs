use crate::audit_shell::{audit_page, demo_box, AuditStatus};
use resuma::prelude::*;

pub fn getting_started(_req: FlowRequest) -> View {
    audit_page(
        "Getting Started",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/getting_started",
        vec![
            demo_box(
                "Hello Resuma — signal + handler",
                vec![Child::View(GettingStartedDemo::render(
                    GettingStartedDemoProps::default(),
                ))],
            ),
            Child::View(view! {
                <p>"CLI: " <code>"cargo install resuma"</code> " · Templates: basic, todo, flow, flow-booking, flow-fullstack"</p>
            }),
        ],
    )
}

#[component]
fn GettingStartedDemo() -> View {
    let msg = signal("Click to resume interactivity.".to_string());
    view! {
        <>
            <h1>"Hello Resuma Audit"</h1>
            <p>{msg}</p>
            <button class="btn" onClick={msg.set("Resumed! Interactivity works.".into())}>"Click me"</button>
        </>
    }
}

pub fn benchmark(_req: FlowRequest) -> View {
    audit_page(
        "Benchmark",
        AuditStatus::Info,
        "https://resuma-docs.fly.dev/docs/benchmark",
        vec![Child::View(view! {
            <>
                <p>"Measured bundle sizes (from docs landing page):"</p>
                <ul>
                    <li><strong>"907 B"</strong>" initial JS (gzip)"</li>
                    <li><strong>"5.08 KiB"</strong>" first interaction"</li>
                    <li><strong>"0 B"</strong>" static pages"</li>
                    <li><strong>"1 crate"</strong>" — core + Flow + CLI unified"</li>
                </ul>
                <p>"Compare vs Qwik, Leptos, Next.js, React, Astro in the benchmark repo."</p>
            </>
        })],
    )
}

pub fn examples(_req: FlowRequest) -> View {
    audit_page(
        "Examples",
        AuditStatus::Info,
        "https://resuma-docs.fly.dev/docs/examples",
        vec![Child::View(view! {
            <>
                <p>"Runnable examples in the Resuma monorepo:"</p>
                <ul>
                    <li><code>"cargo run -p example-counter"</code>" — signals + view!"</li>
                    <li><code>"cargo run -p example-todo"</code>" — full showcase (server, island, security)"</li>
                    <li><code>"cargo run -p example-flow-demo"</code>" — loads, submits, streaming"</li>
                    <li><code>"cargo run -p example-flow-pages"</code>" — file-based pages"</li>
                    <li><code>"cargo run -p example-resuma-audit"</code>" — this audit app"</li>
                </ul>
            </>
        })],
    )
}

pub fn project_structure(_req: FlowRequest) -> View {
    audit_page(
        "Project Structure",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/project_structure",
        vec![Child::View(view! {
            <>
                <p>"This audit app uses Flow layout:"</p>
                <pre style="background:#0b1020;padding:1rem;border-radius:8px;font-size:.85rem">{"examples/resuma-audit/\n├── Cargo.toml\n├── public/          → static assets\n└── src/\n    ├── main.rs      → FlowApp + layout\n    ├── actions.rs   → #[load], #[submit], #[server]\n    ├── security.rs  → CSRF, auth middleware\n    └── pages/       → file-based routes\n        ├── index.rs\n        └── audit/..."}</pre>
            </>
        })],
    )
}

pub fn faq(_req: FlowRequest) -> View {
    audit_page(
        "FAQ",
        AuditStatus::Info,
        "https://resuma-docs.fly.dev/docs/faq",
        vec![Child::View(view! {
            <>
                <h3>"Is this a resume builder?"</h3>
                <p>"No — Resuma is a resumable SSR Rust web framework."</p>
                <h3>"Do I need WASM?"</h3>
                <p>"No — default runtime is ~907 B JS loader, not WASM."</p>
                <h3>"Hydration?"</h3>
                <p>"No hydration — components run once on server; client resumes on first interaction."</p>
            </>
        })],
    )
}

pub fn index(_req: FlowRequest) -> View {
    audit_page(
        "Introduction",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/getting_started",
        vec![Child::View(view! {
            <>
                <p>"Getting started guides and project overview."</p>
                <ul>
                    <li><NavLink href="/audit/intro/getting_started" activeClass="active">"Getting Started"</NavLink></li>
                    <li><NavLink href="/audit/intro/benchmark" activeClass="active">"Benchmark"</NavLink></li>
                    <li><NavLink href="/audit/intro/examples" activeClass="active">"Examples"</NavLink></li>
                </ul>
            </>
        })],
    )
}
