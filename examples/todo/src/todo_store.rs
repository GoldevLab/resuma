//! Todo domain service — NestJS-style service layer (business logic separate from HTTP/actions).

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use resuma::prelude::*;
use serde::{Deserialize, Serialize};

use crate::security;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u64,
    pub owner_id: String,
    pub title: String,
    pub done: bool,
}

/// DTO + validation pipe (NestJS `ValidationPipe` / class-validator equivalent).
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

static TODOS: Lazy<Mutex<Vec<Todo>>> = Lazy::new(|| {
    Mutex::new(vec![
        Todo {
            id: 1,
            owner_id: "alice".into(),
            title: "Read how resumability works (no hydration)".into(),
            done: true,
        },
        Todo {
            id: 2,
            owner_id: "guest".into(),
            title: "Add a task — server action via #[server]".into(),
            done: false,
        },
        Todo {
            id: 3,
            owner_id: "bob".into(),
            title: "Toggle, edit, or filter — island chunk loads on first click".into(),
            done: false,
        },
    ])
});

static NEXT_ID: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(4));

pub fn list_for(req: &FlowRequest) -> Result<Vec<Todo>> {
    let uid = security::session_user(req);
    let is_admin = security::admin_users().iter().any(|a| a == &uid);
    Ok(TODOS
        .lock()
        .iter()
        .filter(|t| is_admin || t.owner_id == uid)
        .cloned()
        .collect())
}

pub fn add(title: String, req: &FlowRequest) -> Result<Vec<Todo>> {
    let input = AddTodoInput { title };
    let title = input.into_title()?;
    let owner = security::require_user(req)?.to_string();
    let mut todos = TODOS.lock();
    security::can_add_todo(todos.len())?;
    let mut id = NEXT_ID.lock();
    let new_id = *id;
    *id += 1;
    drop(id);
    todos.push(Todo {
        id: new_id,
        owner_id: owner,
        title,
        done: false,
    });
    drop(todos);
    list_for(req)
}

pub fn rename(id: u64, title: String, req: &FlowRequest) -> Result<Vec<Todo>> {
    let (id, title) = RenameTodoInput { id, title }.validated()?;
    let mut todos = TODOS.lock();
    let owner = todos
        .iter()
        .find(|t| t.id == id)
        .map(|t| t.owner_id.clone())
        .ok_or_else(|| ResumaError::Other("task not found".into()))?;
    security::assert_owner(&owner, req)?;
    if let Some(t) = todos.iter_mut().find(|t| t.id == id) {
        t.title = title;
    }
    drop(todos);
    list_for(req)
}

pub fn toggle(id: u64, req: &FlowRequest) -> Result<Vec<Todo>> {
    security::valid_id(id)?;
    let mut todos = TODOS.lock();
    let owner = todos
        .iter()
        .find(|t| t.id == id)
        .map(|t| t.owner_id.clone())
        .ok_or_else(|| ResumaError::Other("task not found".into()))?;
    security::assert_owner(&owner, req)?;
    if let Some(t) = todos.iter_mut().find(|t| t.id == id) {
        t.done = !t.done;
    }
    drop(todos);
    list_for(req)
}

pub fn remove(id: u64, req: &FlowRequest) -> Result<Vec<Todo>> {
    security::valid_id(id)?;
    let mut todos = TODOS.lock();
    let owner = todos
        .iter()
        .find(|t| t.id == id)
        .map(|t| t.owner_id.clone())
        .ok_or_else(|| ResumaError::Other("task not found".into()))?;
    security::assert_owner(&owner, req)?;
    todos.retain(|t| t.id != id);
    drop(todos);
    list_for(req)
}

pub fn clear_done(req: &FlowRequest) -> Result<Vec<Todo>> {
    let uid = security::require_user(req)?.to_string();
    let is_admin = security::admin_users().iter().any(|a| a == &uid);
    let mut todos = TODOS.lock();
    todos.retain(|t| {
        if !t.done {
            return true;
        }
        is_admin || t.owner_id == uid
    });
    drop(todos);
    list_for(req)
}

pub fn mark_all_done(req: &FlowRequest) -> Result<Vec<Todo>> {
    let uid = security::require_user(req)?.to_string();
    let is_admin = security::admin_users().iter().any(|a| a == &uid);
    let mut todos = TODOS.lock();
    for t in todos.iter_mut() {
        if is_admin || t.owner_id == uid {
            t.done = true;
        }
    }
    drop(todos);
    list_for(req)
}
