//! Resuma Audit — comprehensive feature verification app.
//!
//! One route per docs section at https://resuma-docs.fly.dev/docs
//! Run: `cargo run -p example-resuma-audit`

#![allow(
    dead_code,
    unused_imports,
    non_snake_case,
    clippy::needless_borrow,
    clippy::redundant_static_lifetimes,
    clippy::needless_update
)]
mod actions;
mod audit_shell;
mod db;
mod extensions_catalog;
mod image_service;
mod page_impls;
mod platform;
mod security;
mod test_registry;
mod todo_store;

mod pages;

use audit_shell::CSS;
use pages::PagesRegistry;
use resuma::prelude::*;

#[middleware]
async fn audit_log(req: FlowRequest) -> resuma::Result<FlowRequest> {
    println!("[audit] {} {}", req.method, req.path);
    Ok(req)
}

#[layout("/")]
fn RootLayout() -> View {
    let theme = Theme::default();
    view! {
        <div class="shell" style={theme_css_vars(&theme)}>
            <nav class="nav">
                <NavLink href="/" activeClass="active">"🏠 Audit Home"</NavLink>
                <NavLink href="/audit/intro/getting_started" activeClass="active">"Intro"</NavLink>
                <NavLink href="/audit/components/signals" activeClass="active">"Components"</NavLink>
                <NavLink href="/audit/flow/loaders" activeClass="active">"Flow"</NavLink>
                <NavLink href="/audit/security/todo" activeClass="active">"Security"</NavLink>
                <NavLink href="/audit/cookbook/debouncer" activeClass="active">"Cookbook"</NavLink>
                <NavLink href="/audit/reference/registry" activeClass="active">"Registry"</NavLink>
                <NavLink href="/audit/reference/matrix" activeClass="active">"Matrix"</NavLink>
                <NavLink href="/audit/integrations" activeClass="active">"Integrations"</NavLink>
            </nav>
            <Slot />
            <div id="modals" data-r-portal-target="modals"></div>
        </div>
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    security::install();

    db::init_db()
        .await
        .map_err(|e| std::io::Error::other(format!("database init failed: {e}")))?;

    if let Some(meta) = db::meta() {
        println!(
            "[audit:db] connected ({}) — {} seed todos",
            meta.url_display, meta.todo_count
        );
    }

    let site_url = std::env::var("SITE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".into());

    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    FlowApp::new()
        .with_title("Resuma Audit — Full Feature Verification")
        .with_description("Interactive audit of every Resuma docs section")
        .with_site_url(site_url)
        .with_head(CSS)
        .with_public_dir(manifest.join("public"))
        .streaming(true)
        .not_found(not_found_page)
        .auto_pages(manifest.join("src/pages"), PagesRegistry)
        .serve(FlowServeOptions {
            addr: security::bind_addr(),
            security: security::security_config(),
            ..Default::default()
        })
        .await
}
