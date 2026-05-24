use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Supabase"</h1>
            <p class="lead">"Hosted PostgreSQL + auth — use Supabase as your Postgres backend with SQLx in Resuma Flow."</p>

            <h2>"Setup"</h2>
            {code_block(r#"# .env
DATABASE_URL=postgres://postgres.[ref]:[password]@aws-0-[region].pooler.supabase.com:6543/postgres

# Cargo.toml
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "macros", "migrate"] }"#)}

            <h2>"Loader with Supabase Postgres"</h2>
            {code_block(r#"#[load]
async fn posts(_req: &FlowRequest) -> Vec<Post> {
    sqlx::query_as!(Post, "SELECT id, title FROM posts ORDER BY id DESC LIMIT 20")
        .fetch_all(db::pool())
        .await
        .unwrap_or_default()
}"#)}

            <h2>"Supabase Auth (optional)"</h2>
            <p>
                "Verify JWT from " <code>"Authorization: Bearer"</code> " in middleware using Supabase JWT secret, "
                "or use Supabase only as managed Postgres and handle auth with "
                <a href="/docs/integrations/auth">"Flow auth"</a>"."
            </p>
            <p>"See also " <a href="/docs/integrations/sqlx">"SQLx integration"</a> " for loaders, submits, and migrations."</p>
        </>
    }
}
