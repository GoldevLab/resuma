//! Todo domain service — SQLx-backed with row-level filtering and ownership checks.

use resuma::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db;
use crate::security;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: i64,
    pub owner_id: String,
    pub title: String,
    pub done: bool,
}

#[derive(FromRow)]
struct TodoRow {
    id: i64,
    owner_id: String,
    title: String,
    done: i64,
}

fn row_to_todo(row: TodoRow) -> Todo {
    Todo {
        id: row.id,
        owner_id: row.owner_id,
        title: row.title,
        done: row.done != 0,
    }
}

#[derive(Debug, Deserialize)]
pub struct AddTodoInput {
    pub title: String,
}

impl AddTodoInput {
    pub fn into_title(self) -> Result<String> {
        security::normalize_title(&self.title)
    }
}

#[derive(Debug, Deserialize)]
pub struct RenameTodoInput {
    pub id: u64,
    pub title: String,
}

impl RenameTodoInput {
    pub fn validated(self) -> Result<(u64, String)> {
        security::valid_id(self.id)?;
        let title = security::normalize_title(&self.title)?;
        Ok((self.id, title))
    }
}

pub async fn list_for(req: &FlowRequest) -> Result<Vec<Todo>> {
    let uid = security::session_user(req);
    let is_admin = security::admin_users().iter().any(|a| a == &uid);
    let pool = db::pool();
    let rows = if is_admin {
        sqlx::query_as::<_, TodoRow>("SELECT id, owner_id, title, done FROM todos ORDER BY id")
            .fetch_all(&pool)
            .await
            .map_err(db_err)?
    } else {
        sqlx::query_as::<_, TodoRow>(
            "SELECT id, owner_id, title, done FROM todos WHERE owner_id = ? ORDER BY id",
        )
        .bind(&uid)
        .fetch_all(&pool)
        .await
        .map_err(db_err)?
    };
    Ok(rows.into_iter().map(row_to_todo).collect())
}

pub async fn add(title: String, req: &FlowRequest) -> Result<Vec<Todo>> {
    let title = AddTodoInput { title }.into_title()?;
    let owner = security::require_user(req)?.to_string();
    let pool = db::pool();
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM todos")
        .fetch_one(&pool)
        .await
        .map_err(db_err)?;
    security::can_add_todo(count.0 as usize)?;
    sqlx::query("INSERT INTO todos (owner_id, title, done) VALUES (?, ?, 0)")
        .bind(&owner)
        .bind(&title)
        .execute(&pool)
        .await
        .map_err(db_err)?;
    list_for(req).await
}

pub async fn rename(id: u64, title: String, req: &FlowRequest) -> Result<Vec<Todo>> {
    let (id, title) = RenameTodoInput { id, title }.validated()?;
    let owner = owner_for_id(id).await?;
    security::assert_owner(&owner, req)?;
    sqlx::query("UPDATE todos SET title = ? WHERE id = ?")
        .bind(&title)
        .bind(id as i64)
        .execute(&db::pool())
        .await
        .map_err(db_err)?;
    list_for(req).await
}

pub async fn toggle(id: u64, req: &FlowRequest) -> Result<Vec<Todo>> {
    security::valid_id(id)?;
    let owner = owner_for_id(id).await?;
    security::assert_owner(&owner, req)?;
    sqlx::query("UPDATE todos SET done = CASE done WHEN 1 THEN 0 ELSE 1 END WHERE id = ?")
        .bind(id as i64)
        .execute(&db::pool())
        .await
        .map_err(db_err)?;
    list_for(req).await
}

pub async fn remove(id: u64, req: &FlowRequest) -> Result<Vec<Todo>> {
    security::valid_id(id)?;
    let owner = owner_for_id(id).await?;
    security::assert_owner(&owner, req)?;
    sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id as i64)
        .execute(&db::pool())
        .await
        .map_err(db_err)?;
    list_for(req).await
}

pub async fn clear_done(req: &FlowRequest) -> Result<Vec<Todo>> {
    let uid = security::require_user(req)?.to_string();
    let is_admin = security::admin_users().iter().any(|a| a == &uid);
    let pool = db::pool();
    if is_admin {
        sqlx::query("DELETE FROM todos WHERE done = 1")
            .execute(&pool)
            .await
            .map_err(db_err)?;
    } else {
        sqlx::query("DELETE FROM todos WHERE done = 1 AND owner_id = ?")
            .bind(&uid)
            .execute(&pool)
            .await
            .map_err(db_err)?;
    }
    list_for(req).await
}

pub async fn mark_all_done(req: &FlowRequest) -> Result<Vec<Todo>> {
    let uid = security::require_user(req)?.to_string();
    let is_admin = security::admin_users().iter().any(|a| a == &uid);
    let pool = db::pool();
    if is_admin {
        sqlx::query("UPDATE todos SET done = 1")
            .execute(&pool)
            .await
            .map_err(db_err)?;
    } else {
        sqlx::query("UPDATE todos SET done = 1 WHERE owner_id = ?")
            .bind(&uid)
            .execute(&pool)
            .await
            .map_err(db_err)?;
    }
    list_for(req).await
}

async fn owner_for_id(id: u64) -> Result<String> {
    let row: Option<(String,)> = sqlx::query_as("SELECT owner_id FROM todos WHERE id = ?")
        .bind(id as i64)
        .fetch_optional(&db::pool())
        .await
        .map_err(db_err)?;
    row.map(|(owner,)| owner)
        .ok_or_else(|| ResumaError::Other("task not found".into()))
}

fn db_err(err: sqlx::Error) -> ResumaError {
    ResumaError::Other(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security;
    use std::collections::BTreeMap;

    fn req_for(user: &str) -> FlowRequest {
        let req = FlowRequest {
            method: "POST".into(),
            path: "/_resuma/action/list_todos".into(),
            headers: BTreeMap::from([(
                "cookie".into(),
                format!("{}={user}", security::DEMO_USER_COOKIE),
            )]),
            ..Default::default()
        };
        security::attach_session(req).expect("session")
    }

    #[tokio::test]
    async fn todo_db_security_suite() {
        db::reset_test_db().await.expect("test db");

        let guest = req_for("guest");
        let alice = req_for("alice");
        let bob = req_for("bob");

        let guest_list = list_for(&guest).await.expect("guest list");
        assert!(guest_list.iter().all(|t| t.owner_id == "guest"));
        assert_eq!(guest_list.len(), 1);

        let alice_list = list_for(&alice).await.expect("alice list");
        assert!(alice_list.len() >= 3);

        let err = toggle(2, &bob)
            .await
            .expect_err("bob cannot toggle guest task");
        assert!(matches!(err, ResumaError::Forbidden(_)));

        add("Guest task".into(), &guest).await.expect("add");
        let after = list_for(&guest).await.expect("guest after add");
        assert_eq!(after.len(), 2);

        toggle(after[0].id as u64, &guest)
            .await
            .expect("toggle own task");

        let err = add("".into(), &guest).await.expect_err("empty title");
        assert!(matches!(err, ResumaError::Other(_)));
    }
}
