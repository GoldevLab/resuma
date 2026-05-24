//! SQLite/Postgres pool — init before `FlowApp::serve()`.

use sqlx::sqlite::SqlitePoolOptions;
use std::sync::OnceLock;

static POOL: OnceLock<sqlx::SqlitePool> = OnceLock::new();

pub async fn init_db() -> anyhow::Result<()> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:local.db".into());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    POOL.set(pool)
        .map_err(|_| anyhow::anyhow!("database pool already initialized"))?;
    Ok(())
}

pub fn pool() -> &'static sqlx::SqlitePool {
    POOL.get().expect("call db::init_db() before FlowApp::serve()")
}
