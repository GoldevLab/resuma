use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Turso"</h1>
            <p class="lead">
                "Turso is an edge database built on "
                <a href="https://github.com/tursodatabase/libsql" target="_blank">"libSQL"</a>
                " (open-source SQLite fork). "
                "Same model as "
                <a href="https://qwik.dev/docs/integrations/turso/" target="_blank">"Qwik's Turso integration"</a>
                ": " <code>"file:"</code> " locally, remote URL + auth token in production."
            </p>

            <h2>"Why Turso with Resuma"</h2>
            <ul>
                <li>"SQLite semantics — zero server to manage for small/medium apps"</li>
                <li>"Edge replicas — data close to Fly regions (Paris, Ashburn, etc.)"</li>
                <li>"Identical SQL in dev and prod when you use libSQL file mode locally"</li>
                <li>"Pairs with " <a href="/docs/integrations/sqlx">"SQLx"</a> " (SQLite driver) or the official " <code>"libsql"</code> " crate"</li>
            </ul>

            <h2>"Install"</h2>
            {code_block(r#"[dependencies]
libsql = "0.6"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }

# Optional: SQLx with SQLite for compile-time queries against Turso
# sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "macros"] }"#)}

            <h2>"Client helper"</h2>
            {code_block(r#"// src/turso.rs
use libsql::{Builder, Connection};

pub async fn connect() -> anyhow::Result<Connection> {
    let url = std::env::var("TURSO_DATABASE_URL")
        .unwrap_or_else(|_| "file:local.db".into());

    let db = if url.starts_with("file:") {
        Builder::new_local(url.strip_prefix("file:").unwrap()).build().await?
    } else {
        let token = std::env::var("TURSO_AUTH_TOKEN")?;
        Builder::new_remote(url, token).build().await?
    };

    Ok(db.connect()?)
}"#)}

            <h2>"Local file database (dev / CI)"</h2>
            {code_block(r#"# Create schema
sqlite3 local.db
# sqlite> CREATE TABLE todo (id INTEGER PRIMARY KEY, task TEXT NOT NULL, done INTEGER DEFAULT 0);
# sqlite> .quit

# .env (never commit)
TURSO_DATABASE_URL=file:local.db
# No auth token needed for file: URLs"#)}

            <h2>"Listing todos - " <code>"#[load]"</code></h2>
            {code_block(r#"#[derive(Clone, Serialize, Deserialize)]
struct Todo {
    id: i64,
    task: String,
    done: bool,
}

#[load]
async fn todos(_req: &FlowRequest) -> Vec<Todo> {
    let conn = crate::turso::connect().await.ok()?;
    let mut rows = conn
        .query("SELECT id, task, done FROM todo ORDER BY id", ())
        .await
        .ok()?;

    let mut out = Vec::new();
    while let Ok(Some(row)) = rows.next().await {
        out.push(Todo {
            id: row.get::<i64>(0).unwrap_or(0),
            task: row.get::<String>(1).unwrap_or_default(),
            done: row.get::<i64>(2).unwrap_or(0) != 0,
        });
    }
    out
}

pub fn page(_req: FlowRequest) -> View {
    let items = use_todos_load();
    view! {
        <ul>
            {items.iter().map(|t| view! {
                <li key={t.id.to_string()}>{t.task.clone()}</li>
            }).collect::<Vec<_>>()}
        </ul>
    }
}"#)}

            <h2>"Adding a todo - " <code>"#[submit]"</code></h2>
            {code_block(r#"#[derive(Deserialize)]
struct NewTodo {
    task: String,
}

#[submit]
async fn add_todo(form: NewTodo, _req: &FlowRequest) -> Result<(), SubmitError> {
    if form.task.trim().is_empty() {
        return Err(SubmitError::new("Fix errors").field("task", "Required"));
    }
    let conn = crate::turso::connect()
        .await
        .map_err(|_| SubmitError::new("Database unavailable"))?;
    conn.execute("INSERT INTO todo (task) VALUES (?)", [form.task.as_str()])
        .await
        .map_err(|_| SubmitError::new("Insert failed"))?;
    Ok(())
}"#)}

            <h2>"Production Turso database"</h2>
            {code_block(r#"# Turso CLI
turso db create my-app
turso db show my-app --url
turso db tokens create my-app

# Fly secrets
fly secrets set \
  TURSO_DATABASE_URL="libsql://my-app-....turso.io" \
  TURSO_AUTH_TOKEN="eyJ..." \
  --app my-app"#)}

            <h2>"Deploy on Fly.io"</h2>
            <p>
                "Turso complements Fly's global edge: your Resuma app on Fly + Turso replica in the same region "
                "keeps TTFB low for data-heavy loaders. No Postgres VM required."
            </p>
            <p>
                "For relational workloads that outgrow SQLite, migrate to "
                <a href="/docs/integrations/sqlx">"SQLx + PostgreSQL"</a> " — the Flow loader/submit code shape stays the same."
            </p>

            <h2>"SQLx vs libsql client"</h2>
            <table class="docs-table">
                <thead>
                    <tr>
                        <th>"Approach"</th>
                        <th>"Best for"</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td><code>"libsql"</code> " crate"</td>
                        <td>"Turso-native features, embedded replicas, simplest Turso docs parity"</td>
                    </tr>
                    <tr>
                        <td><code>"sqlx"</code> " + SQLite"</td>
                        <td>"Compile-time query macros, shared pool code with Postgres builds"</td>
                    </tr>
                </tbody>
            </table>
        </>
    }
}
