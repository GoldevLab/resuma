//! Production template — Flow + security middleware + deploy files.

mod pages;
mod security;

use pages::PagesRegistry;
use resuma::prelude::*;

#[layout("/")]
fn RootLayout() -> View {
    view! {
        <div class="shell">
            <nav>
                <NavLink href="/" activeClass="active">"Home"</NavLink>
            </nav>
            <Slot />
        </div>
    }
}

const CSS: &str = r#"<style>
body { font-family: system-ui, sans-serif; background: #0b1020; color: #e6e8ee; margin: 0; }
.shell { max-width: 42rem; margin: 0 auto; padding: 2rem 1rem; }
nav { display: flex; gap: 1rem; margin-bottom: 1.5rem; }
nav a { color: #b9bfd2; text-decoration: none; }
nav a.active { color: #818cf8; font-weight: 600; }
.card { background: #14182b; border: 1px solid #2a2f4a; padding: 1.5rem; border-radius: 12px; }
</style>"#;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    security::install();

    FlowApp::new()
        .with_title("%NAME%")
        .with_head(CSS)
        .not_found(|| not_found_page())
        .auto_pages(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pages"),
            PagesRegistry,
        )
        .serve(FlowServeOptions {
            security: SecurityConfig::from_env(),
            ..FlowServeOptions::default()
        })
        .await
}
