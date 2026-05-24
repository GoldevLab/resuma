//! Resuma official documentation site — landing + docs (Resuma¹ + Flow²).

mod pages;
mod site;

use pages::PagesRegistry;
use resuma::current_request;
use resuma::prelude::*;

#[layout("/")]
fn SiteLayout() -> View {
    view! {
        <header class="site-header">
            <div class="header-inner">
                <a href="/" class="logo">
                    <span class="logo-mark">"🌊"</span>
                    "Resuma"
                </a>
                <nav class="site-nav">
                    <NavLink href="/docs" activeClass="active">"Docs"</NavLink>
                    <NavLink href="/docs/getting_started" activeClass="active">"Tutorial"</NavLink>
                    <NavLink href="/docs/flow" activeClass="active">"Flow"</NavLink>
                </nav>
                <div class="header-actions">
                    <a href="/docs/getting_started" class="btn btn-ghost">"Get Started"</a>
                    <a href="https://docs.rs/resuma/0.3.1" class="btn btn-ghost" target="_blank">"docs.rs"</a>
                    <a href="https://crates.io/crates/resuma" class="btn btn-ghost" target="_blank">"crates.io"</a>
                    <a href="https://github.com/GolfredoPerezFernandez/resuma" class="btn btn-primary">"GitHub"</a>
                </div>
            </div>
        </header>
        <Slot />
        <footer class="site-footer">
            <p>"Made with ❤️ by the Resuma team · MIT License"</p>
            <div class="site-footer-links">
                <a href="https://crates.io/crates/resuma" target="_blank">"crates.io"</a>
                <a href="https://docs.rs/resuma/0.3.1" target="_blank">"docs.rs"</a>
                <a href="/docs/package">"Install guide"</a>
                <a href="/docs/architecture">"Architecture"</a>
                <a href="/docs/benchmark">"Benchmarks"</a>
                <a href="https://github.com/GolfredoPerezFernandez/resuma" target="_blank">"GitHub"</a>
            </div>
        </footer>
    }
}

#[layout("/docs")]
fn DocsLayout() -> View {
    let path = current_request()
        .map(|r| r.path)
        .unwrap_or_else(|| "/docs".into());

    view! {
        <div class="docs-shell">
            {site::doc_sidebar(&path)}
            <main class="docs-main">
                <Slot />
            </main>
        </div>
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let pages_root = std::env::var("RESUMA_PAGES_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pages"));

    let site_url = site::site_url();
    let json_ld = site::json_ld(&site_url);

    FlowApp::new()
        .with_title(site::site_title())
        .with_description(site::site_description())
        .with_site_url(site_url)
        .with_og_image("/og.svg")
        .with_json_ld(json_ld)
        .with_pwa(site::pwa_config())
        .with_head(site::SITE_CSS)
        .streaming(false)
        .not_found(|| not_found_page())
        .auto_pages(pages_root, PagesRegistry)
        .serve(FlowServeOptions::default())
        .await
}
