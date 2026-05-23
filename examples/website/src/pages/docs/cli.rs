use resuma::prelude::*;
use crate::site::code_block;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"CLI Reference"</h1>
            <p class="lead">"The resuma command scaffolds projects, runs dev servers, builds releases, and generates route registries."</p>

            <h2>"Install"</h2>
            {code_block(r#"cargo install resuma

# From monorepo source:
cargo install --path crates/resuma --features cli"#)}

            <h2>"resuma new / resuma create"</h2>
            <p>"Scaffold a new project from a template. " <code>"create"</code> " is an alias for " <code>"new"</code>"."</p>
            {code_block(r#"resuma new my-app
resuma new my-app --template counter   # single-page ResumaApp
resuma new my-app --template flow      # multi-page FlowApp"#)}

            <h2>"resuma dev"</h2>
            <p>"Run the app with hot reload. Binds to 127.0.0.1:3000 by default. Rebuilds the JS runtime unless " <code>"--skip-runtime"</code> " is passed."</p>
            {code_block(r#"resuma dev
resuma dev --addr 0.0.0.0:8080
resuma dev --skip-runtime"#)}

            <h2>"resuma build"</h2>
            <p>"Build a production release binary and JS bundles (" <code>"cargo build --release"</code> " + runtime npm build)."</p>
            {code_block("resuma build")}

            <h2>"resuma routes"</h2>
            <p>"Discover file-based routes under a pages directory. With " <code>"--generate"</code> ", writes " <code>"mod.rs"</code> " and " <code>"_registry.rs"</code>"."</p>
            {code_block(r#"resuma routes --path src/pages
resuma routes --generate --path src/pages"#)}

            <h2>"Without the CLI"</h2>
            <p>"All commands map to plain Cargo workflows:"</p>
            {code_block(r#"cargo run                              # dev server
cargo build --release                  # production build
cargo run -p resuma-cli -- routes --generate --path src/pages"#)}
        </>
    }
}
