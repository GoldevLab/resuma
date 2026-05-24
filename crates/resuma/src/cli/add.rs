//! `resuma add <integration>` — add SQLx or Turso scaffolding to an existing project.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

const DB_SQLX: &str = include_str!("../../templates/add/sqlx/db.rs");
const ENV_SQLX: &str = include_str!("../../templates/add/sqlx/env.example");
const MIGRATION_SQLX: &str = include_str!("../../templates/add/sqlx/001_users.sql");

const DB_TURSO: &str = include_str!("../../templates/add/turso/turso.rs");
const ENV_TURSO: &str = include_str!("../../templates/add/turso/env.example");

const CARGO_SQLX: &str = r#"sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "macros", "migrate"] }
anyhow = "1"
"#;

const CARGO_TURSO: &str = r#"libsql = "0.6"
anyhow = "1"
"#;

pub fn add_integration(name: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    if !cwd.join("Cargo.toml").exists() {
        return Err(anyhow!(
            "run `resuma add` from a Rust project root (Cargo.toml not found)"
        ));
    }

    match name {
        "sqlx" => add_sqlx(&cwd),
        "turso" => add_turso(&cwd),
        other => Err(anyhow!(
            "unknown integration `{other}` (try: sqlx, turso)"
        )),
    }
}

fn add_sqlx(root: &Path) -> Result<()> {
    write_if_missing(root.join("src/db.rs"), DB_SQLX)?;
    write_if_missing(root.join(".env.example"), ENV_SQLX)?;
    let mig_dir = root.join("migrations");
    fs::create_dir_all(&mig_dir)?;
    write_if_missing(mig_dir.join("001_users.sql"), MIGRATION_SQLX)?;
    append_cargo_deps(root, CARGO_SQLX)?;
    println!("[resuma] added SQLx: src/db.rs, migrations/, .env.example");
    println!("  Set DATABASE_URL=sqlite:local.db (or postgres URL)");
    println!("  sqlx migrate run");
    Ok(())
}

fn add_turso(root: &Path) -> Result<()> {
    write_if_missing(root.join("src/turso.rs"), DB_TURSO)?;
    write_if_missing(root.join(".env.example"), ENV_TURSO)?;
    append_cargo_deps(root, CARGO_TURSO)?;
    println!("[resuma] added Turso: src/turso.rs, .env.example");
    println!("  TURSO_DATABASE_URL=file:local.db  (dev)");
    Ok(())
}

fn write_if_missing(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    let path = path.as_ref();
    if path.exists() {
        println!("[resuma] skip {} (already exists)", path.display());
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(path, contents).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn append_cargo_deps(root: &Path, deps: &str) -> Result<()> {
    let cargo_path = root.join("Cargo.toml");
    let mut cargo = fs::read_to_string(&cargo_path)?;
    if cargo.contains("sqlx =") || (deps.contains("libsql") && cargo.contains("libsql =")) {
        println!("[resuma] dependencies already present in Cargo.toml");
        return Ok(());
    }
    if let Some(idx) = cargo.find("[dependencies]") {
        let insert_at = cargo[idx..]
            .find('\n')
            .map(|i| idx + i + 1)
            .unwrap_or(cargo.len());
        cargo.insert_str(insert_at, deps);
        fs::write(&cargo_path, cargo)?;
    } else {
        cargo.push_str("\n[dependencies]\n");
        cargo.push_str(deps);
        fs::write(&cargo_path, cargo)?;
    }
    Ok(())
}
