use crate::site::code_block;
use resuma::prelude::*;

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
            {code_block(r#"resuma new my-app
resuma new my-app --template basic          # static SSR (default)
resuma new my-app --template todo           # full showcase
resuma new my-app --template flow           # file-based pages
resuma new my-app --template flow-fullstack # Flow + SQLx sample"#)}

            <h2>"resuma add"</h2>
            {code_block(r#"resuma add sqlx    # src/db.rs, migrations/, deps
resuma add turso  # src/turso.rs, .env.example"#)}

            <h2>"resuma dev"</h2>
            {code_block(r#"resuma dev
resuma dev --open
resuma dev --addr 0.0.0.0:8080
resuma dev --skip-runtime"#)}

            <h2>"resuma build"</h2>
            {code_block("resuma build")}

            <h2>"resuma routes"</h2>
            {code_block(r#"resuma routes --path src/pages
resuma routes --generate --path src/pages"#)}
        </>
    }
}
