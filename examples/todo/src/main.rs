//! Resuma todo example.
//!
//! Demonstrates:
//!   * `#[server]` actions called from event handlers via `actions::name(arg)`.
//!   * Server-side state shared across requests through a `parking_lot::Mutex`.
//!   * `#[island]` interactive components.
//!
//! Run:
//!
//! ```sh
//! cargo run -p example-todo
//! # open http://127.0.0.1:3000
//! ```

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use resuma::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub done: bool,
}

static TODOS: Lazy<Mutex<Vec<Todo>>> = Lazy::new(|| {
    Mutex::new(vec![
        Todo { id: 1, title: "Write the Resuma readme".into(), done: false },
        Todo { id: 2, title: "Beat Leptos to resumability".into(), done: true },
    ])
});

static NEXT_ID: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(3));

#[server]
async fn list_todos(req: &FlowRequest) -> Vec<Todo> {
    if let Some(ua) = req.header("user-agent") {
        println!("[todo] list_todos from {ua}");
    }
    TODOS.lock().clone()
}

#[server]
async fn add_todo(title: String) -> Vec<Todo> {
    let mut id_guard = NEXT_ID.lock();
    let id = *id_guard;
    *id_guard += 1;
    drop(id_guard);

    let mut todos = TODOS.lock();
    todos.push(Todo { id, title, done: false });
    todos.clone()
}

#[server]
async fn toggle_todo(id: u64) -> Vec<Todo> {
    let mut todos = TODOS.lock();
    if let Some(t) = todos.iter_mut().find(|t| t.id == id) {
        t.done = !t.done;
    }
    todos.clone()
}

#[server]
async fn remove_todo(id: u64) -> Vec<Todo> {
    let mut todos = TODOS.lock();
    todos.retain(|t| t.id != id);
    todos.clone()
}

#[component]
fn TodoApp() -> View {
    // Initial state is captured on the server. The list is then patched by
    // server actions executed from event handlers.
    let todos = use_signal::<Vec<Todo>>(TODOS.lock().clone());
    let new_title = use_signal(String::new());

    view! {
        <main class="card">
            <h1>"Resuma Todos"</h1>
            <p class="hint">"Backed by a #[server] action — no client framework code, only the ~3KB runtime."</p>

            <form
                onSubmit={
                    js! {
                        event.preventDefault();
                        const t = state.new_title.value;
                        if (!t) return;
                        const next = await __resuma.action("add_todo", [t]);
                        state.todos.set(next);
                        state.new_title.set("");
                    }
                }
            >
                <input
                    type="text"
                    placeholder="What needs doing?"
                    onInput={
                        js! {
                            state.new_title.set(event.target.value);
                        }
                    }
                />
                <button type="submit">"Add"</button>
            </form>

            <ul id="todo-list">
                {format!("{} item(s)", todos.peek().len())}
            </ul>
        </main>
    }
}

const INLINE_CSS: &str = r#"<style>
body { font-family: ui-sans-serif, system-ui, sans-serif; background: #0b1020; color: #e6e8ee; margin: 0; padding: 3rem; }
.card { max-width: 420px; margin: 0 auto; background: #14182b; padding: 2rem; border-radius: 16px; }
.card h1 { margin: 0 0 .5rem; }
.card form { display: flex; gap: .5rem; margin: 1rem 0; }
.card input { flex: 1; padding: .5rem .8rem; background: #1d233f; border: 1px solid #2a2f4a; border-radius: 8px; color: white; }
.card button { background: #6366f1; color: white; border: 0; border-radius: 8px; padding: .5rem .9rem; cursor: pointer; }
.card ul { list-style: none; padding: 0; }
.card .hint { font-size: .85rem; opacity: .7; }
</style>"#;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    ResumaApp::new()
        .with_title("Resuma · Todo")
        .with_head(INLINE_CSS)
        .page("/", || TodoApp::render(TodoAppProps::default()))
        .serve(ServeOptions::default())
        .await
}
