use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"SQLx"</h1>
            <p class="lead">
                "SQLx is the Rust data layer closest to "
                <a href="https://qwik.dev/docs/integrations/drizzle/" target="_blank">"Qwik's Drizzle"</a>
                ": async, SQL-native, and type-safe. "
                "Use it inside " <code>"#[load]"</code> ", " <code>"#[submit]"</code> ", and " <code>"#[server]"</code> " — all server-only."
            </p>

            <h2>"Why SQLx for Resuma"</h2>
            <ul>
                <li>"Async-first — matches Tokio + Axum under the hood"</li>
                <li>"Compile-time query checking with " <code>"sqlx::query!"</code> " / " <code>"sqlx::query_as!"</code></li>
                <li>"Works with PostgreSQL, SQLite, MySQL — same Flow patterns everywhere"</li>
                <li>"Migrations via " <code>"sqlx migrate"</code> " (similar role to Drizzle Kit)"</li>
            </ul>

            <h2>"Install"</h2>
            {code_block(r#"# Cargo.toml — pick one driver feature
[dependencies]
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "macros", "migrate"] }
# or for Turso/local SQLite:
# sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "macros", "migrate"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }"#)}

            <h2>"Connection pool (app startup)"</h2>
            {code_block(r#"// src/db.rs
use sqlx::postgres::PgPoolOptions;
use std::sync::OnceLock;

static POOL: OnceLock<sqlx::PgPool> = OnceLock::new();

pub async fn init_db() -> anyhow::Result<()> {
    let url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    POOL.set(pool).map_err(|_| anyhow::anyhow!("pool already set"))?;
    Ok(())
}

pub fn pool() -> &'static sqlx::PgPool {
    POOL.get().expect("call init_db() before serve()")
}"#)}

            <h2>"Listing users - " <code>"#[load]"</code> " (routeLoader$)"</h2>
            {code_block(r#"#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
struct User {
    id: i64,
    name: String,
    email: String,
}

#[load]
async fn users(_req: &FlowRequest) -> Vec<User> {
    sqlx::query_as!(User, "SELECT id, name, email FROM users ORDER BY id")
        .fetch_all(crate::db::pool())
        .await
        .unwrap_or_default()
}

// pages/users/index.rs
pub fn page(_req: FlowRequest) -> View {
    let users = use_users_load();
    view! {
        <section>
            <h1>"User directory"</h1>
            <ul>
                {users.iter().map(|u| view! {
                    <li key={u.id.to_string()}>
                        <a href={format!("/users/{}", u.id)}>
                            {format!("{} ({})", u.name, u.email)}
                        </a>
                    </li>
                }).collect::<Vec<_>>()}
            </ul>
        </section>
    }
}"#)}

            <h2>"User detail - param from " <code>"FlowRequest"</code></h2>
            {code_block(r#"#[load]
async fn user_detail(req: &FlowRequest) -> Option<User> {
    let id: i64 = req.param("id")?.parse().ok()?;
    sqlx::query_as!(User, "SELECT id, name, email FROM users WHERE id = $1", id)
        .fetch_optional(crate::db::pool())
        .await
        .ok()
        .flatten()
}"#)}

            <h2>"Adding a user - " <code>"#[submit]"</code> " (routeAction$)"</h2>
            {code_block(r#"#[derive(Deserialize)]
struct CreateUserForm {
    name: String,
    email: String,
}

#[submit]
async fn create_user(form: CreateUserForm, _req: &FlowRequest)
    -> Result<(), SubmitError>
{
    if form.email.is_empty() {
        return Err(SubmitError::new("Fix errors").field("email", "Required"));
    }
    sqlx::query!(
        "INSERT INTO users (name, email) VALUES ($1, $2)",
        form.name,
        form.email,
    )
    .execute(crate::db::pool())
    .await
    .map_err(|_| SubmitError::new("Could not create user"))?;
    Ok(())
}

// In the page:
view! {
    <Form submit={create_user}>
        <label>"Name" <input name="name" type="text" /></label>
        <label>"Email" <input name="email" type="email" /></label>
        <button type="submit">"Create"</button>
    </Form>
}"#)}

            <h2>"Server RPC - " <code>"#[server]"</code></h2>
            {code_block(r#"#[server]
async fn toggle_todo(id: i64, done: bool) -> Result<bool, String> {
    sqlx::query!("UPDATE todos SET done = $1 WHERE id = $2", done, id)
        .execute(crate::db::pool())
        .await
        .map_err(|e| e.to_string())?;
    Ok(done)
}"#)}

            <h2>"Migrations"</h2>
            {code_block(r#"# migrations/001_users.sql
CREATE TABLE users (
    id   BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE
);

# CLI
sqlx migrate run
# offline check in CI: cargo sqlx prepare --check"#)}

            <h2>"Global extensions"</h2>
            {code_block(r#"db::init_db().await?;

FlowApp::new()
    .with_extension("db", "ready")
    .auto_pages(pages_root, PagesRegistry)
    .serve(FlowServeOptions::default())
    .await

// In #[load]:
async fn users(req: &FlowRequest) -> Vec<User> {
    assert_eq!(req.extension("db").and_then(|v| v.as_str()), Some("ready"));
    // ...
}"#)}

            <h2>"Deploy on Fly.io"</h2>
            <p>
                "Attach " <a href="https://fly.io/docs/postgres/" target="_blank">"Fly Postgres"</a> " and set "
                <code>"DATABASE_URL"</code> " as a secret. Call " <code>"init_db().await"</code> " before "
                <code>"FlowApp::serve()"</code> ". See " <a href="/docs/cookbook/docker">"Docker deploy"</a> "."
            </p>

            <p>
                "For edge SQLite instead of Postgres, use "
                <a href="/docs/integrations/turso">"Turso + libSQL"</a> "."
            </p>
        </>
    }
}
