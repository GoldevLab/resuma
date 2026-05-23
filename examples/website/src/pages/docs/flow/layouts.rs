use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Layouts"</h1>
            <p class="lead">"Layouts wrap pages with shared chrome — nav, sidebars, and nested shells via #[layout] and Slot."</p>

            <h2>"Define a layout"</h2>
            {code_block(r#"#[layout("/")]
fn SiteLayout() -> View {
    view! {
        <header>
            <nav>
                <NavLink href="/" activeClass="active">"Home"</NavLink>
            </nav>
        </header>
        <Slot />
        <footer>"© 2026"</footer>
    }
}"#)}

            <h2>"Nested layouts"</h2>
            {code_block(r#"#[layout("/docs")]
fn DocsLayout() -> View {
    view! {
        <div class="docs-shell">
            <aside>{doc_sidebar()}</aside>
            <main><Slot /></main>
        </div>
    }
}"#)}

            <h2>"Layout markers"</h2>
            <p>"Place layout.rs or _layout.rs in a directory to mark layout scope. Layouts in main.rs use " <code>"#[layout(\"/prefix\")]"</code> " with a URL prefix."</p>

            <h2>"How nesting works"</h2>
            <p>"The router computes a layout chain root → leaf. Each layout renders its Slot with the next inner layout or the page content."</p>
            {code_block(r##"/docs/getting_started
  → SiteLayout (/)
    → DocsLayout (/docs)
      → getting_started page"##)}
        </>
    }
}
