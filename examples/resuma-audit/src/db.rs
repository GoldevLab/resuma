//! SQLite pool + migrations for the audit todo showcase.

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::sync::{OnceLock, RwLock};

static POOL: OnceLock<RwLock<Option<SqlitePool>>> = OnceLock::new();
static META: OnceLock<DbMeta> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct DbMeta {
    pub url_display: String,
    pub todo_count: i64,
}

fn storage() -> &'static RwLock<Option<SqlitePool>> {
    POOL.get_or_init(|| RwLock::new(None))
}

pub fn default_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("audit.db");
        format!("sqlite://{}", path.display())
    })
}

pub async fn init_db() -> anyhow::Result<()> {
    init_db_with_url(&default_database_url()).await
}

pub async fn init_db_with_url(url: &str) -> anyhow::Result<()> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    seed(&pool).await?;
    let count = count_todos(&pool).await?;
    let _ = META.set(DbMeta {
        url_display: mask_database_url(url),
        todo_count: count,
    });
    *storage().write().expect("db lock") = Some(pool);
    Ok(())
}

pub fn pool() -> SqlitePool {
    storage()
        .read()
        .expect("db lock")
        .clone()
        .expect("call db::init_db() before using the database")
}

pub fn meta() -> Option<&'static DbMeta> {
    META.get()
}

pub fn mask_database_url(url: &str) -> String {
    if let Some((scheme, rest)) = url.split_once("://") {
        if rest.contains('@') {
            return format!("{scheme}://***@***");
        }
    }
    url.to_string()
}

async fn count_todos(pool: &SqlitePool) -> anyhow::Result<i64> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM todos")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

async fn seed(pool: &SqlitePool) -> anyhow::Result<()> {
    let count = count_todos(pool).await?;
    if count > 0 {
        return Ok(());
    }
    let seeds = [
        (
            1_i64,
            "alice",
            "Read how resumability works (no hydration)",
            1,
        ),
        (2, "guest", "Add a task — server action via #[server]", 0),
        (
            3,
            "bob",
            "Toggle, edit, or filter — island chunk loads on first click",
            0,
        ),
    ];
    for (id, owner, title, done) in seeds {
        sqlx::query("INSERT INTO todos (id, owner_id, title, done) VALUES (?, ?, ?, ?)")
            .bind(id)
            .bind(owner)
            .bind(title)
            .bind(done)
            .execute(pool)
            .await?;
    }
    Ok(())
}

#[cfg(test)]
pub async fn reset_test_db() -> anyhow::Result<()> {
    init_db_with_url("sqlite::memory:").await
}
