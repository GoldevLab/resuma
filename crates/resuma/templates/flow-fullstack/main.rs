//! Flow + SQLx full-stack template.

mod db;
mod pages;

use pages::PagesRegistry;
use resuma::prelude::*;

#[derive(sqlx::FromRow)]
#[data]
struct UserRow {
    id: i64,
    name: String,
    email: String,
}

#[load]
async fn users(_req: &FlowRequest) -> Vec<UserRow> {
    sqlx::query_as::<_, UserRow>("SELECT id, name, email FROM users ORDER BY id")
        .fetch_all(db::pool())
        .await
        .unwrap_or_default()
}

#[data]
struct CreateUser {
    name: String,
    email: String,
}

#[submit]
async fn create_user(
    form: CreateUser,
    _req: &FlowRequest,
) -> std::result::Result<(), SubmitError> {
    if form.name.trim().is_empty() {
        return Err(SubmitError::new("Fix errors").field("name", "Required"));
    }
    sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
        .bind(form.name)
        .bind(form.email)
        .execute(db::pool())
        .await
        .map_err(|_| SubmitError::new("Could not save user"))?;
    Ok(())
}

#[layout("/")]
fn RootLayout() -> View {
    view! {
        <div class="shell">
            <nav>
                <NavLink href="/" activeClass="active">"Home"</NavLink>
                <NavLink href="/users" activeClass="active">"Users"</NavLink>
            </nav>
            <Slot />
        </div>
    }
}

const CSS: &str = r#"<style>
body { font-family: system-ui, sans-serif; background: #0b1020; color: #e6e8ee; margin: 0; }
.shell { max-width: 42rem; margin: 0 auto; padding: 2rem 1rem; }
nav { display: flex; gap: 1rem; margin-bottom: 1.5rem; }
nav a { color: #b9bfd2; text-decoration: none; }
nav a.active { color: #818cf8; font-weight: 600; }
.card { background: #14182b; border: 1px solid #2a2f4a; padding: 1.5rem; border-radius: 12px; }
input { display: block; width: 100%; margin: 0.25rem 0 0.75rem; padding: 0.4rem; }
button { background: #6366f1; color: #fff; border: 0; padding: 0.5rem 1rem; border-radius: 8px; cursor: pointer; }
</style>"#;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    db::init_db()
        .await
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    FlowApp::new()
        .with_title("%NAME%")
        .with_head(CSS)
        .with_extension("db", "ready")
        .not_found(|| not_found_page())
        .auto_pages(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pages"),
            PagesRegistry,
        )
        .serve(FlowServeOptions::default())
        .await
}
