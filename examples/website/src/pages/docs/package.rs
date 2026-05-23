use resuma::prelude::*;
use crate::site::code_block;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Resuma + Flow"</h1>
            <p class="lead">"One package to install, two layers to learn — like Qwik + Qwik City, but unified for Rust."</p>

            <h2>"The model"</h2>
            <table class="docs-table">
                <thead>
                    <tr><th>"Layer"</th><th>"Crate (internal)"</th><th>"You import"</th><th>"Purpose"</th></tr>
                </thead>
                <tbody>
                    <tr><td><strong>"Resuma¹"</strong></td><td>"resuma-core, resuma-ssr, resuma-server"</td><td>"resuma::prelude::*"</td><td>"Components, signals, SSR, resumability"</td></tr>
                    <tr><td><strong>"Flow²"</strong></td><td>"resuma-flow, resuma-router"</td><td>"FlowApp, #[load], #[submit]"</td><td>"Pages, routing, data, forms"</td></tr>
                </tbody>
            </table>

            <h2>"Install"</h2>
            <p>"Users depend on a single crate:"</p>
            {code_block(r#"[dependencies]
resuma = "0.1"
tokio  = { version = "1", features = ["full"] }"#)}

            <p>"Everything re-exports through " <code>"resuma::prelude"</code>":"</p>
            {code_block(r#"use resuma::prelude::*;
// ResumaApp, ResumaApp, view!, #[component], #[server]
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
            {code_block(r#"resuma new my-app                    # static SSR (default)
resuma new my-app --template todo    # full showcase
resuma dev
resuma build
resuma routes --generate --path src/pages   # Flow apps only"#)}

            <h2>"API map (Qwik → Resuma)"</h2>
            <table class="docs-table">
                <thead>
                    <tr><th>"Qwik / Qwik City"</th><th>"Resuma Flow"</th></tr>
                </thead>
                <tbody>
                    <tr><td>"component$"</td><td>"#[component] + view!"</td></tr>
                    <tr><td>"routeLoader$"</td><td>"#[load]"</td></tr>
                    <tr><td>"routeAction$"</td><td>"#[submit]"</td></tr>
                    <tr><td>"server$"</td><td>"#[server]"</td></tr>
                    <tr><td>"plugin.ts middleware"</td><td>"#[middleware]"</td></tr>
                    <tr><td>"src/routes/"</td><td>"src/pages/"</td></tr>
                </tbody>
            </table>
        </>
    }
}
