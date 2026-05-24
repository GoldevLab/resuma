use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Project Structure"</h1>
            <p class="lead">"ResumaApp (basic/todo templates) and FlowApp (multi-page) project layouts."</p>

            <h2>"Top-level layout"</h2>
            <p>"A flow starter is a standard Rust binary crate. The CLI generates Cargo.toml, src/main.rs, and a pages directory for file-based routing."</p>
            {code_block(r##"my-app/
├── Cargo.toml
├── src/
│   ├── main.rs
│   └── pages/
│       ├── index.rs
│       ├── layout.rs
│       ├── mod.rs
│       └── _registry.rs"##)}

            <h2>"Cargo.toml"</h2>
            <p>"Depend on the umbrella crate — core + Flow in one dependency."</p>
            {code_block(r#"[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[dependencies]
resuma = { version = "0.3", default-features = false }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }"#)}

            <h2>"src/main.rs"</h2>
            <p>"Wire FlowApp, layouts, and auto-discovered pages. Layouts use " <code>"#[layout]"</code> " with a URL prefix; pages live under " <code>"src/pages"</code>"."</p>
            {code_block(r#"use resuma::prelude::*;
mod pages;
use pages::PagesRegistry;

#[layout("/")]
fn AppLayout() -> View {
    view! {
        <header>"My App"</header>
        <Slot />
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    FlowApp::new()
        .with_title("My App")
        .streaming(true)
        .auto_pages("src/pages", PagesRegistry)
        .not_found(|| not_found_page())
        .serve(FlowServeOptions::default())
        .await
}"#)}

            <h2>"Pages directory"</h2>
            <p>"Each .rs file under " <code>"src/pages"</code> " maps to a URL. Run " <code>"resuma routes --generate"</code> " after adding or renaming pages."</p>
            {code_block(r##"src/pages/
├── index.rs          → /
├── about.rs          → /about
├── blog/
│   ├── index.rs      → /blog
│   └── [slug].rs     → /blog/:slug
├── users/
│   └── [id].rs       → /users/:id
├── layout.rs         → layout marker (not a route)
├── mod.rs            → generated module tree
└── _registry.rs      → generated PagesRegistry"##)}

            <h2>"Generated files"</h2>
            <p>"Do not edit " <code>"mod.rs"</code> " or " <code>"_registry.rs"</code> " by hand — regenerate with:"</p>
            {code_block("resuma routes --generate --path src/pages")}

            <h2>"CLI templates"</h2>
            <p>"Scaffold with " <code>"resuma new --template basic"</code> " or " <code>"--template todo"</code>"."</p>

            <h3>"basic / todo (ResumaApp)"</h3>
            {code_block(r##"my-app/
├── Cargo.toml
└── src/
    ├── main.rs          # ResumaApp + routes
    ├── security.rs      # (todo template only)
    └── todo_store.rs    # (todo template only)"##)}

            <h3>"Flow (manual or flow-pages example)"</h3>
            <p>"Multi-page apps use FlowApp and " <code>"src/pages/"</code> ". See " <code>"examples/flow-pages"</code> " in the repo."</p>
        </>
    }
}
