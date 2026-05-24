use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Resuma + Flow"</h1>
            <p class="lead">"One crate to install — core, server, Flow, and CLI together."</p>

            <h2>"The model"</h2>
            <table class="docs-table">
                <thead>
                    <tr><th>"Layer"</th><th>"Module (internal)"</th><th>"You import"</th><th>"Purpose"</th></tr>
                </thead>
                <tbody>
                    <tr><td><strong>"Resuma¹"</strong></td><td>"resuma::core, ssr, server"</td><td>"resuma::prelude::*"</td><td>"Components, signals, SSR, resumability"</td></tr>
                    <tr><td><strong>"Flow²"</strong></td><td>"resuma::flow, router"</td><td>"FlowApp, #[load], #[submit]"</td><td>"Pages, routing, data, forms"</td></tr>
                </tbody>
            </table>

            <h2>"Install"</h2>
            <p>"Users depend on a single crate:"</p>
            {code_block(r#"[dependencies]
resuma = "0.3"
tokio  = { version = "1", features = ["full"] }"#)}

            <p>"Everything re-exports through " <code>"resuma::prelude"</code>":"</p>
            {code_block(r#"use resuma::prelude::*;
// ResumaApp, view!, #[component], #[server]
// FlowApp, #[load], #[submit], #[layout], #[middleware]"#)}

            <h2>"When to use what"</h2>
            <ul>
                <li><strong>"ResumaApp"</strong>" — single-page or manually registered routes. Perfect for widgets, islands, demos."</li>
                <li><strong>"FlowApp"</strong>" — multi-page apps with " <code>"src/pages/"</code>", layouts, server data, forms."</li>
            </ul>

            <h2>"Project structure (Flow)"</h2>
            {code_block(r#"my-app/
  src/
    main.rs           # FlowApp bootstrap
    pages/
      index.rs        # GET /
      about.rs        # GET /about
      users/
        [id].rs       # GET /users/:id
        layout.rs     # layout for /users/*
  Cargo.toml          # resuma + tokio only"#)}

            <h2>"CLI commands"</h2>
            {code_block(r#"cargo install resuma
resuma new my-app                    # static SSR (default)
resuma new my-app --template todo    # full showcase
resuma new my-app --template flow-fullstack  # Flow + SQLx + users CRUD
resuma add sqlx                      # add SQLx to existing project
resuma add turso                     # add Turso/libSQL client
resuma dev
resuma dev --open                    # open browser
resuma build
resuma routes --generate --path src/pages   # Flow apps only"#)}

            <h2>"Published crates"</h2>
            <p>"Only two crates ship on crates.io: " <code>"resuma"</code> " (runtime) and " <code>"resuma-macros"</code> " (proc-macros — required by Rust)."</p>
            <table class="docs-table">
                <thead>
                    <tr><th>"Crate"</th><th>"crates.io"</th><th>"docs.rs"</th></tr>
                </thead>
                <tbody>
                    <tr>
                        <td><code>"resuma"</code></td>
                        <td><a href="https://crates.io/crates/resuma" target="_blank">"crates.io/crates/resuma"</a></td>
                        <td><a href="https://docs.rs/resuma" target="_blank">"docs.rs/resuma"</a></td>
                    </tr>
                    <tr>
                        <td><code>"resuma-macros"</code></td>
                        <td><a href="https://crates.io/crates/resuma-macros" target="_blank">"crates.io/crates/resuma-macros"</a></td>
                        <td><a href="https://docs.rs/resuma-macros" target="_blank">"docs.rs/resuma-macros"</a></td>
                    </tr>
                </tbody>
            </table>

            <h2>"Full-stack API map"</h2>
            <table class="docs-table">
                <thead>
                    <tr><th>"Concept"</th><th>"Resuma Flow"</th></tr>
                </thead>
                <tbody>
                    <tr><td>"Component"</td><td>"#[component] + view!"</td></tr>
                    <tr><td>"Server data loader"</td><td>"#[load]"</td></tr>
                    <tr><td>"Form mutation"</td><td>"#[submit]"</td></tr>
                    <tr><td>"Server RPC"</td><td>"#[server]"</td></tr>
                    <tr><td>"Request middleware"</td><td>"#[middleware]"</td></tr>
                    <tr><td>"File-based pages"</td><td>"src/pages/"</td></tr>
                </tbody>
            </table>
        </>
    }
}
