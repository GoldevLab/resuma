//! Turso / libSQL client — file DB locally, remote URL + token in production.

use libsql::{Builder, Connection};

pub async fn connect() -> anyhow::Result<Connection> {
    let url = std::env::var("TURSO_DATABASE_URL").unwrap_or_else(|_| "file:local.db".into());
    let db = if url.starts_with("file:") {
        Builder::new_local(url.strip_prefix("file:").unwrap())
            .build()
            .await?
    } else {
        let token = std::env::var("TURSO_AUTH_TOKEN")?;
        Builder::new_remote(url, token).build().await?
    };
    Ok(db.connect()?)
}
