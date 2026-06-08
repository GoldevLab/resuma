use crate::audit_shell::{audit_page, AuditStatus};
use resuma::prelude::*;

pub fn architecture(_req: FlowRequest) -> View {
    audit_page(
        "Architecture",
        AuditStatus::Info,
        "https://resuma-docs.fly.dev/docs/architecture",
        vec![Child::View(view! {
            <>
                <p>"Resumability: components run once on server. SSR embeds signals + handler refs in HTML."</p>
                <p>"907 B gzip loader resumes interactivity on first click — no hydration."</p>
            </>
        })],
    )
}

pub fn reactivity(_req: FlowRequest) -> View {
    audit_page(
        "Reactivity Internals",
        AuditStatus::Info,
        "https://resuma-docs.fly.dev/docs/reactivity",
        vec![Child::View(view! {
            <p>"Signals serialized in resuma/state script tag. Client runtime replays subscriptions."</p>
        })],
    )
}

pub fn package(_req: FlowRequest) -> View {
    audit_page(
        "Package",
        AuditStatus::Info,
        "https://resuma-docs.fly.dev/docs/package",
        vec![Child::View(view! {
            <>
                <p>"Single crate: " <code>"resuma = \"0.4.7\""</code></p>
                <p>"Features: cli, default = full stack"</p>
            </>
        })],
    )
}

pub fn cli(_req: FlowRequest) -> View {
    audit_page(
        "CLI",
        AuditStatus::Info,
        "https://resuma-docs.fly.dev/docs/cli",
        vec![Child::View(view! {
            <>
                <ul>
                    <li><code>"resuma new"</code>" — scaffold"</li>
                    <li><code>"resuma dev"</code>" — hot reload"</li>
                    <li><code>"resuma build"</code>" — release"</li>
                    <li><code>"resuma routes --generate"</code>" — page registry"</li>
                    <li><code>"resuma doctor"</code>" — diagnostics"</li>
                </ul>
            </>
        })],
    )
}

pub fn api(_req: FlowRequest) -> View {
    audit_page(
        "API Reference",
        AuditStatus::Info,
        "https://resuma-docs.fly.dev/docs/api",
        vec![Child::View(view! {
            <>
                <p><a href="https://docs.rs/resuma" target="_blank">"docs.rs/resuma"</a></p>
                <p><a href="https://docs.rs/resuma-macros" target="_blank">"docs.rs/resuma-macros"</a></p>
            </>
        })],
    )
}

pub fn registry(req: FlowRequest) -> View {
    crate::test_registry::registry(req)
}
