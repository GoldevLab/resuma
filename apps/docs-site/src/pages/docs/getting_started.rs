use resuma::prelude::*;

use crate::site::{code_block, playground_card};

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Getting Started Resumably"</h1>
            <p class="lead">
                "Resuma is a resumable Rust web framework — no hydration, no eager JS execution. "
                "Components run on the server; a tiny loader resumes interactivity on demand. "
                "Resuma Flow adds file-based pages, loads, and submits in one crate — core and full-stack unified."
            </p>

            <h2>"Examples in this repo"</h2>
            <p>"See " <a href="/docs/examples">"Examples"</a> " for all runnable crates and when to use each."</p>
            <p>
                "Rust apps run on the server — clone the repo and launch a live example in one command:"
            </p>
            <div class="playground-grid">
                {playground_card(
                    "Todo (full showcase)",
                    "Signals, #[server], #[island], js!, theme — every Resuma feature in one app.",
                    "cargo run -p example-todo",
                )}
                {playground_card(
                    "Flow demo (full-stack)",
                    "Loads, submits, streaming SSR, and deferred chunks.",
                    "cargo run -p example-flow-demo",
                )}
            </div>
            <p>
                "Docs are live at "
                <a href="https://resuma-docs.fly.dev/" target="_blank">"resuma-docs.fly.dev"</a>
                ". To hack on the docs site locally: "
                <code>"cargo run -p example-website"</code> " (source in "
                <code>"apps/docs-site"</code>", not under "
                <code>"examples/"</code>")."
            </p>

            <h2>"Prerequisites"</h2>
            <p>"To build Resuma apps locally, you need:"</p>
            <ul>
                <li><a href="https://rustup.rs">"Rust 1.91+"</a>" (stable channel via rustup)"</li>
                <li><a href="https://nodejs.org">"Node.js 18+"</a>" (optional — only to rebuild the JS runtime)"</li>
                <li>"Your favorite editor ("<a href="https://code.visualstudio.com/">"VS Code"</a>" + rust-analyzer recommended)"</li>
            </ul>
            <p>
                "Optionally, read "
                <a href="/docs/architecture">"How resumability works"</a>
                " before scaffolding."
            </p>

            <h2>"Install the CLI"</h2>
            <p>
                "From "
                <a href="https://crates.io/crates/resuma" target="_blank">"crates.io"</a>
                " (recommended):"
            </p>
            {code_block("cargo install resuma")}
            <p>
                "API reference: "
                <a href="https://docs.rs/resuma" target="_blank">"docs.rs/resuma"</a>
                " · "
                <a href="https://docs.rs/resuma-macros" target="_blank">"docs.rs/resuma-macros"</a>
            </p>
            <p>"From source while developing the monorepo:"</p>
            {code_block(r#"git clone https://github.com/GolfredoPerezFernandez/resuma
cd resuma
cargo install --path crates/resuma --features cli

resuma --help"#)}

            <h2>"Create an app using the CLI"</h2>
            <p>
                "Use " <code>"resuma new"</code> " or " <code>"resuma create"</code> " to scaffold a starter. "
                "Pick a template:"
            </p>
            <div class="template-grid">
                <div class="template-pill">
                    <strong>"basic"</strong>
                    <span>"Static SSR page · zero client JS · clean starting point"</span>
                </div>
                <div class="template-pill">
                    <strong>"todo"</strong>
                    <span>"Signals · #[server] · #[island] · js! — all Resuma features"</span>
                </div>
            </div>
            {code_block(r#"# Static page (default)
resuma new my-app
resuma new my-app --template basic

# Full feature showcase
resuma new my-app --template todo

cd my-app"#)}

            <p>"The CLI generates " <code>"Cargo.toml"</code> " and " <code>"src/main.rs"</code> "."</p>

            <h2>"Start the development server"</h2>
            <p>
                "Inside your project directory. "
                <code>"resuma dev"</code> " installs " <code>"cargo-watch"</code> " if needed, rebuilds on save, and refreshes the browser automatically."
            </p>
            {code_block(r#"resuma dev
resuma dev --open   # open http://127.0.0.1:3000"#)}
            <p>"Without the CLI, plain Cargo works too:"</p>
            {code_block("cargo run")}

            <h2>"Hello, Resuma"</h2>
            <p>"A minimal component with resumable state:"</p>
            {code_block(r#"use resuma::prelude::*;

#[component]
fn Hello() -> View {
    let excited = use_signal(false);
    view! {
        <main>
            <h1>"Hello Resuma"</h1>
            <button onClick={ move |_| excited.set(true) }>
                "Click me"
            </button>
        </main>
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    ResumaApp::new()
        .page("/", || Hello::render(HelloProps::default()))
        .serve(ServeOptions::default())
        .await
}"#)}

            <h2>"Add a server action"</h2>
            {code_block(r#"#[server]
async fn greet(name: String) -> String {
    format!("Hello, {name}!")
}"#)}
            <p>"From a handler, call " <code>"__resuma.action('greet', [name])"</code> " — RPC at " <code>"POST /_resuma/action/:name"</code>"."</p>

            <h2>"Project structure"</h2>
            <p><strong>"basic / todo"</strong>" — single " <code>"main.rs"</code> " (+ security modules for todo). "<strong>"Flow"</strong>" — add " <code>"src/pages/"</code> " (see " <a href="/docs/project_structure">"Project structure"</a> ")."</p>
            {code_block(r##"my-app/                  # resuma new --template todo
├── Cargo.toml
└── src/
    ├── main.rs
    ├── security.rs
    └── todo_store.rs"##)}

            <h2>"Next steps"</h2>
            <ul>
                <li><a href="/docs/security/todo">"Todo example — full backend reference"</a></li>
                <li><a href="/docs/flow">"Resuma Flow — multi-page apps"</a></li>
                <li><a href="/docs/package">"Package map"</a></li>
                <li><a href="/docs/architecture">"Architecture"</a></li>
                <li><a href="/docs/cookbook/docker">"Docker deploy"</a></li>
            </ul>
        </>
    }
}
