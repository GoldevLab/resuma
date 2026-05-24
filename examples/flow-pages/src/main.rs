//! Resuma Flow file-based pages — auto-wired via `resuma routes --generate`.

mod pages;

use pages::PagesRegistry;
use resuma::prelude::*;

#[middleware]
async fn log_requests(req: FlowRequest) -> resuma::Result<FlowRequest> {
    println!("[flow-pages] {} {}", req.method, req.path);
    Ok(req)
}

#[layout("/")]
fn RootLayout() -> View {
    let theme = Theme::default();

    view! {
        <div class="shell" style={theme_css_vars(&theme)}>
            <nav class="nav">
                <NavLink href="/" activeClass="active">"Home"</NavLink>
                <NavLink href="/about" activeClass="active">"About"</NavLink>
            </nav>
            <Slot />
            <div id="modals" data-r-portal-target="modals"></div>
        </div>
    }
}

const INLINE_CSS: &str = r#"<style>
* { box-sizing: border-box; }
body { font-family: ui-sans-serif, system-ui, sans-serif; background: var(--resuma-bg, #0b1020); color: var(--resuma-fg, #e6e8ee); margin: 0; min-height: 100vh; }
.shell { max-width: 42rem; margin: 0 auto; padding: 2rem 1rem; }
.nav { display: flex; gap: 1rem; margin-bottom: 1.5rem; }
.nav a { color: #b9bfd2; text-decoration: none; }
.nav a.active { color: var(--resuma-primary, #818cf8); font-weight: 600; }
.card { background: #14182b; border: 1px solid #2a2f4a; padding: 1.5rem; border-radius: 12px; }
.resuma-error { border: 1px solid #7f1d1d; background: #450a0a; padding: 1rem; border-radius: 8px; }
</style>"#;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    FlowApp::new()
        .with_title("Resuma · File Pages")
        .with_head(INLINE_CSS)
        .streaming(true)
        .not_found(|| not_found_page())
        .auto_pages(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pages"),
            PagesRegistry,
        )
        .serve(FlowServeOptions::default())
        .await
}
