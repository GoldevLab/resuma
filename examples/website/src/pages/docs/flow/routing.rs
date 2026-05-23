use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Routing"</h1>
            <p class="lead">"Resuma Flow discovers routes from files under your pages directory — no manual route tables."</p>

            <h2>"File conventions"</h2>
            {code_block(r##"index.rs           → /
about.rs           → /about
blog/index.rs      → /blog
users/[id].rs      → /users/:id
blog/[...slug].rs  → /blog/*slug
_layout.rs         → layout marker (not a route)"##)}

            <h2>"Dynamic segments"</h2>
            <p>"Single-bracket " <code>"[id]"</code> " captures one path segment. Rest-bracket " <code>"[...slug]"</code> " captures the remainder."</p>
            {code_block(r##"src/pages/
├── users/
│   └── [id].rs        → /users/:id
└── docs/
    └── [...slug].rs   → /docs/*slug"##)}

            <h2>"Access params in pages"</h2>
            {code_block(r#"pub fn page(req: FlowRequest) -> View {
    let id = req.param("id").unwrap_or("?");
    view! {
        <h1>"User " {id.to_string()}</h1>
    }
}"#)}

            <h2>"Generate registry"</h2>
            {code_block("resuma routes --generate --path src/pages")}

            <h2>"Inspect routes"</h2>
            {code_block("resuma routes --path src/pages")}
        </>
    }
}
