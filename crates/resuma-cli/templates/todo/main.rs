//! Resuma todo — full feature showcase.
//!
//! Demonstrates (ResumaApp / core):
//!   * `#[component]` composition and `view!`
//!   * `use_signal`, `use_store`, `use_computed`, `use_effect`
//!   * `#[server]` RPC actions (`/_resuma/action/:name`)
//!   * `#[island]` lazy JS boundary for the interactive workspace
//!   * `js!` for async server calls and island-local UI updates
//!   * `provide_theme` / `theme_css_vars`
//!   * `security.rs` — session guard, authorization, validation, audit, SecurityConfig
//!
//! Run: `cargo run -p example-todo` → http://127.0.0.1:3000

mod security;
mod todo_store;

use resuma::prelude::*;
use serde::{Deserialize, Serialize};

use todo_store::Todo;

// ── Domain ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UiState {
    /// `"all"` | `"active"` | `"done"`
    filter: String,
    search: String,
    /// Demo session user (mirrors cookie).
    session_user: String,
    /// `0` = not editing
    editing_id: u64,
    edit_draft: String,
    busy: bool,
    status: String,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            filter: "all".into(),
            search: String::new(),
            session_user: security::DEFAULT_USER.into(),
            editing_id: 0,
            edit_draft: String::new(),
            busy: false,
            status: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Default)]
struct Stats {
    total: usize,
    done: usize,
    pending: usize,
}

// ── Server actions (Next.js Server Actions / NestJS Controllers) ─────────────

#[server]
async fn list_todos(req: &FlowRequest) -> Result<Vec<Todo>> {
    todo_store::list_for(req)
}

#[server]
async fn add_todo(title: String, req: &FlowRequest) -> Result<Vec<Todo>> {
    todo_store::add(title, req)
}

#[server]
async fn rename_todo(id: u64, title: String, req: &FlowRequest) -> Result<Vec<Todo>> {
    todo_store::rename(id, title, req)
}

#[server]
async fn toggle_todo(id: u64, req: &FlowRequest) -> Result<Vec<Todo>> {
    todo_store::toggle(id, req)
}

#[server]
async fn remove_todo(id: u64, req: &FlowRequest) -> Result<Vec<Todo>> {
    todo_store::remove(id, req)
}

#[server]
async fn clear_done(req: &FlowRequest) -> Result<Vec<Todo>> {
    todo_store::clear_done(req)
}

#[server]
async fn mark_all_done(req: &FlowRequest) -> Result<Vec<Todo>> {
    todo_store::mark_all_done(req)
}

// ── Components ───────────────────────────────────────────────────────────────

#[component]
fn FeatureBadges() -> View {
    view! {
        <details class="badges-details">
            <summary>"What this demo uses"</summary>
            <ul class="badges">
                <li>"use_signal"</li>
                <li>"use_store"</li>
                <li>"use_computed"</li>
                <li>"use_effect"</li>
                <li>"#[server]"</li>
                <li>"#[island]"</li>
                <li>"js!"</li>
                <li>"theme"</li>
                <li>"CSRF"</li>
                <li>"rate limit"</li>
                <li>"validation"</li>
                <li>"auth guard"</li>
                <li>"service layer"</li>
                <li>"DTO validation"</li>
                <li>"request id"</li>
            </ul>
        </details>
    }
}

#[component]
fn SecurityNote() -> View {
    view! {
        <aside class="security-note">
            <h2>"NestJS + Next.js patterns"</h2>
            <ul>
                <li><strong>"Guard"</strong>" — " <code>"attach_session()"</code> " in action middleware"</li>
                <li><strong>"ValidationPipe"</strong>" — " <code>"AddTodoInput"</code> " / " <code>"RenameTodoInput"</code> " DTOs in " <code>"todo_store.rs"</code></li>
                <li><strong>"Service"</strong>" — " <code>"todo_store::add()"</code> " etc. (business logic)"</li>
                <li><strong>"Controller"</strong>" — thin " <code>"#[server]"</code> " actions delegate to store"</li>
                <li><strong>"Server Action"</strong>" — " <code>"list_todos"</code> " re-fetch (Next.js revalidate pattern)"</li>
                <li><strong>"Interceptor"</strong>" — request id + audit log per action"</li>
                <li><strong>"Throttler"</strong>" — " <code>"SecurityConfig.actions_per_minute"</code></li>
                <li><strong>"Exception filter"</strong>" — " <code>"Result<T, ResumaError>"</code> " → HTTP status"</li>
            </ul>
            <p class="security-note__hint">"See " <code>"src/security.rs"</code> ", " <code>"src/todo_store.rs"</code> " · Docs: " <code>"/docs/security/todo"</code></p>
        </aside>
    }
}

#[component]
fn SessionBar() -> View {
    view! {
        <div class="session-bar" role="group" aria-label="Demo session">
            <span class="session-bar__label">"Demo user:"</span>
            {security::demo_users().iter().map(|user| {
                let u = (*user).to_string();
                view! {
                    <button
                        type="button"
                        class="session-btn"
                        data-user={u.clone()}
                        onClick={
                            js! {
                                const user = event.currentTarget.dataset.user;
                                document.cookie = "resuma_demo_user=" + user + "; path=/; SameSite=Strict";
                                location.reload();
                            }
                        }
                    >{u}</button>
                }
            }).collect::<Vec<_>>()}
            <span class="session-bar__hint">"Switch user to test authorization"</span>
        </div>
    }
}

#[component]
fn ProgressRing(stats: Stats) -> View {
    let pct = if stats.total == 0 {
        0
    } else {
        (stats.done * 100) / stats.total
    };
    view! {
        <div class="progress-ring" role="progressbar" aria-valuenow={pct.to_string()} aria-valuemin="0" aria-valuemax="100">
            <div class="progress-ring__track">
                <div class="progress-ring__fill" style={format!("width:{pct}%")} />
            </div>
            <span class="progress-ring__label">{format!("{pct}% done")}</span>
        </div>
    }
}

#[component]
fn StatPills(stats: Stats) -> View {
    view! {
        <div class="stat-pills" aria-label="Task counts">
            <span class="pill"><strong>{stats.total.to_string()}</strong>" total"</span>
            <span class="pill pill--pending"><strong>{stats.pending.to_string()}</strong>" active"</span>
            <span class="pill pill--done"><strong>{stats.done.to_string()}</strong>" done"</span>
        </div>
    }
}

/// Interactive todo UI — ships as an island chunk (`/_resuma/island/...`).
#[island]
fn todo_workspace() -> View {
    let todos = use_signal(Vec::<Todo>::new());
    let new_title = use_signal(String::new());
    let ui = use_store(UiState::default());

    use_visible_task(
        r#"
        (async () => {
            try {
                const next = await __resuma.action("list_todos", []);
                state.todos.set(next);
            } catch (e) {
                state.ui.update(s => { s.status = "Could not load tasks"; });
            }
        })()
    "#,
    );

    let todos_for_stats = todos.clone();
    let stats = use_computed(move || {
        let list = todos_for_stats.get();
        let done = list.iter().filter(|t| t.done).count();
        Stats {
            total: list.len(),
            done,
            pending: list.len() - done,
        }
    });

    let todos_for_visible = todos.clone();
    let ui_for_visible = ui.clone();
    let visible = use_computed(move || {
        let list = todos_for_visible.get();
        let ui = ui_for_visible.get();
        let q = ui.search.to_lowercase();
        let mut rows: Vec<Todo> = list
            .into_iter()
            .filter(|t| match ui.filter.as_str() {
                "active" => !t.done,
                "done" => t.done,
                _ => true,
            })
            .filter(|t| q.is_empty() || t.title.to_lowercase().contains(&q))
            .collect();
        rows.sort_by_key(|t| (t.done, t.id));
        rows
    });

    let new_title_for_effect = new_title.clone();
    let ui_for_effect = ui.clone();
    use_effect(move || {
        let draft = new_title_for_effect.get();
        if !draft.is_empty() {
            ui_for_effect.update(|s| {
                if !s.status.is_empty() {
                    s.status.clear();
                }
            });
        }
    });

    let ui_snap = ui.get();
    let stats_snap = stats.get();
    let can_add = !new_title.get().trim().is_empty() && !ui_snap.busy;
    let show_clear = stats_snap.done > 0 && !ui_snap.busy;
    let show_mark_all = stats_snap.pending > 0 && !ui_snap.busy;
    let empty_msg = if stats_snap.total == 0 {
        "Your list is empty — add your first task above."
    } else if !ui_snap.search.is_empty() {
        "No tasks match your search. Try different words or clear the search."
    } else {
        match ui_snap.filter.as_str() {
            "active" => "No active tasks. You're all caught up!",
            "done" => "Nothing completed yet. Check off a task when you're done.",
            _ => "No tasks to show.",
        }
    };

    view! {
        <section class="workspace" aria-busy={if ui_snap.busy { "true" } else { "false" }}>
            <div class="workspace-head">
                <StatPills stats={stats_snap} />
                <ProgressRing stats={stats_snap} />
            </div>

            {if !ui_snap.status.is_empty() {
                view! {
                    <p class="flash" role="status" aria-live="polite">
                        {ui_snap.status.clone()}
                        <button
                            type="button"
                            class="flash-dismiss"
                            aria-label="Dismiss message"
                            onClick={
                                js! {
                                    state.ui.update(s => { s.status = ""; });
                                }
                            }
                        >"×"</button>
                    </p>
                }
            } else {
                View::Empty
            }}

            <form
                class="add-form"
                aria-label="Add a task"
                onSubmit={
                    js! {
                        event.preventDefault();
                        const title = state.new_title.value.trim();
                        if (!title || state.ui.value.busy) return;
                        state.ui.update(s => { s.busy = true; s.status = ""; });
                        try {
                            const next = await __resuma.action("add_todo", [title]);
                            state.todos.set(next);
                            state.new_title.set("");
                            state.ui.update(s => { s.busy = false; s.status = "Task added"; });
                        } catch (e) {
                            state.ui.update(s => { s.busy = false; s.status = "Could not add task"; });
                        }
                    }
                }
            >
                <label class="sr-only" for="new-todo">"New task"</label>
                <input
                    id="new-todo"
                    type="text"
                    autocomplete="off"
                    maxlength={security::MAX_TITLE_LEN.to_string()}
                    placeholder="What needs doing?"
                    value={new_title.get()}
                    onInput={
                        js! {
                            state.new_title.set(event.target.value);
                        }
                    }
                />
                <button type="submit" class="btn-primary" disabled={!can_add}>
                    {if ui_snap.busy { "Adding…" } else { "Add" }}
                </button>
            </form>
            <p class="form-hint">{format!("Up to {} chars · max {} tasks · CSRF protected", security::MAX_TITLE_LEN, security::MAX_TODOS)}</p>

            <div class="toolbar">
                <label class="search-wrap">
                    <span class="sr-only">"Search tasks"</span>
                    <input
                        class="search"
                        type="search"
                        placeholder="Search tasks…"
                        value={ui_snap.search.clone()}
                        onInput={
                            js! {
                                state.ui.update(s => { s.search = event.target.value; });
                            }
                        }
                    />
                    {if !ui_snap.search.is_empty() {
                        view! {
                            <button
                                type="button"
                                class="search-clear"
                                aria-label="Clear search"
                                onClick={
                                    js! {
                                        state.ui.update(s => { s.search = ""; });
                                    }
                                }
                            >"×"</button>
                        }
                    } else {
                        View::Empty
                    }}
                </label>

                <div class="filters" role="group" aria-label="Filter tasks">
                    <button
                        class={format!("filter-btn{}", if ui_snap.filter == "all" { " active" } else { "" })}
                        type="button"
                        aria-pressed={if ui_snap.filter == "all" { "true" } else { "false" }}
                        onClick={
                            js! {
                                state.ui.update(s => { s.filter = "all"; });
                            }
                        }
                    >{format!("All ({})", stats_snap.total)}</button>
                    <button
                        class={format!("filter-btn{}", if ui_snap.filter == "active" { " active" } else { "" })}
                        type="button"
                        aria-pressed={if ui_snap.filter == "active" { "true" } else { "false" }}
                        onClick={
                            js! {
                                state.ui.update(s => { s.filter = "active"; });
                            }
                        }
                    >{format!("Active ({})", stats_snap.pending)}</button>
                    <button
                        class={format!("filter-btn{}", if ui_snap.filter == "done" { " active" } else { "" })}
                        type="button"
                        aria-pressed={if ui_snap.filter == "done" { "true" } else { "false" }}
                        onClick={
                            js! {
                                state.ui.update(s => { s.filter = "done"; });
                            }
                        }
                    >{format!("Done ({})", stats_snap.done)}</button>
                </div>

                <div class="bulk-actions">
                    {if show_mark_all {
                        view! {
                            <button
                                type="button"
                                class="btn-ghost"
                                onClick={
                                    js! {
                                        if (state.ui.value.busy) return;
                                        state.ui.update(s => { s.busy = true; s.status = ""; });
                                        try {
                                            const next = await __resuma.action("mark_all_done", []);
                                            state.todos.set(next);
                                            state.ui.update(s => { s.busy = false; s.status = "All tasks marked done"; });
                                        } catch (e) {
                                            state.ui.update(s => { s.busy = false; s.status = "Action failed"; });
                                        }
                                    }
                                }
                            >"Mark all done"</button>
                        }
                    } else {
                        View::Empty
                    }}
                    {if show_clear {
                        view! {
                            <button
                                type="button"
                                class="btn-ghost btn-ghost--danger"
                                onClick={
                                    js! {
                                        if (state.ui.value.busy) return;
                                        if (!confirm("Remove all completed tasks?")) return;
                                        state.ui.update(s => { s.busy = true; s.status = ""; });
                                        try {
                                            const next = await __resuma.action("clear_done", []);
                                            state.todos.set(next);
                                            state.ui.update(s => { s.busy = false; s.status = "Completed tasks cleared"; });
                                        } catch (e) {
                                            state.ui.update(s => { s.busy = false; s.status = "Action failed"; });
                                        }
                                    }
                                }
                            >"Clear done"</button>
                        }
                    } else {
                        View::Empty
                    }}
                </div>
            </div>

            <ul class="todo-list" aria-label="Tasks">
                {visible.get().into_iter().map(|t| {
                    let id = t.id.to_string();
                    let title = t.title.clone();
                    let done = t.done;
                    let editing = ui.get().editing_id == t.id;
                    view! {
                        <li class={format!("todo-item{}", if done { " done" } else { "" })} data-id={id.clone()}>
                            <label class="todo-check">
                                <input
                                    type="checkbox"
                                    checked={done}
                                    aria-label={format!("Mark \"{title}\" as {}", if done { "not done" } else { "done" })}
                                    onChange={
                                        js! {
                                            if (state.ui.value.busy) return;
                                            const row = event.target.closest("li");
                                            const id = Number(row.dataset.id);
                                            state.ui.update(s => { s.busy = true; });
                                            try {
                                                const next = await __resuma.action("toggle_todo", [id]);
                                                state.todos.set(next);
                                                state.ui.update(s => { s.busy = false; });
                                            } catch (e) {
                                                state.ui.update(s => { s.busy = false; s.status = "Could not update task"; });
                                            }
                                        }
                                    }
                                />
                                <span class="check-ui" aria-hidden="true" />
                            </label>

                            {if editing {
                                view! {
                                    <form
                                        class="edit-form"
                                        onSubmit={
                                            js! {
                                                event.preventDefault();
                                                const title = state.ui.value.edit_draft.trim();
                                                const id = state.ui.value.editing_id;
                                                if (!title || !id || state.ui.value.busy) return;
                                                state.ui.update(s => { s.busy = true; s.status = ""; });
                                                try {
                                                    const next = await __resuma.action("rename_todo", [id, title]);
                                                    state.todos.set(next);
                                                    state.ui.update(s => {
                                                        s.busy = false;
                                                        s.editing_id = 0;
                                                        s.edit_draft = "";
                                                        s.status = "Task updated";
                                                    });
                                                } catch (e) {
                                                    state.ui.update(s => { s.busy = false; s.status = "Could not save"; });
                                                }
                                            }
                                        }
                                    >
                                        <input
                                            type="text"
                                            class="edit-input"
                                            maxlength={security::MAX_TITLE_LEN.to_string()}
                                            value={ui.get().edit_draft.clone()}
                                            onInput={
                                                js! {
                                                    state.ui.update(s => { s.edit_draft = event.target.value; });
                                                }
                                            }
                                        />
                                        <button type="submit" class="btn-icon" aria-label="Save">"✓"</button>
                                        <button
                                            type="button"
                                            class="btn-icon"
                                            aria-label="Cancel edit"
                                            onClick={
                                                js! {
                                                    state.ui.update(s => { s.editing_id = 0; s.edit_draft = ""; });
                                                }
                                            }
                                        >"✕"</button>
                                    </form>
                                }
                            } else {
                                view! {
                                    <span class="todo-title">{title}</span>
                                }
                            }}

                            {if !editing {
                                view! {
                                    <div class="todo-actions">
                                        <button
                                            type="button"
                                            class="btn-icon"
                                            aria-label={format!("Edit \"{title}\"")}
                                            onClick={
                                                js! {
                                                    const row = event.target.closest("li");
                                                    const id = Number(row.dataset.id);
                                                    const title = row.querySelector(".todo-title").textContent;
                                                    state.ui.update(s => { s.editing_id = id; s.edit_draft = title; });
                                                }
                                            }
                                        >"✎"</button>
                                        <button
                                            type="button"
                                            class="btn-icon btn-icon--danger"
                                            aria-label={format!("Delete \"{title}\"")}
                                            onClick={
                                                js! {
                                                    if (state.ui.value.busy) return;
                                                    const row = event.target.closest("li");
                                                    const id = Number(row.dataset.id);
                                                    state.ui.update(s => { s.busy = true; });
                                                    try {
                                                        const next = await __resuma.action("remove_todo", [id]);
                                                        state.todos.set(next);
                                                        state.ui.update(s => { s.busy = false; s.status = "Task removed"; });
                                                    } catch (e) {
                                                        state.ui.update(s => { s.busy = false; s.status = "Could not delete"; });
                                                    }
                                                }
                                            }
                                        >"🗑"</button>
                                    </div>
                                }
                            } else {
                                View::Empty
                            }}
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ul>

            {if visible.get().is_empty() {
                view! {
                    <div class="empty">
                        <p>{empty_msg}</p>
                        {if stats_snap.total > 0 && ui_snap.filter != "all" {
                            view! {
                                <button
                                    type="button"
                                    class="btn-link"
                                    onClick={
                                        js! {
                                            state.ui.update(s => { s.filter = "all"; s.search = ""; });
                                        }
                                    }
                                >"Show all tasks"</button>
                            }
                        } else {
                            View::Empty
                        }}
                    </div>
                }
            } else {
                View::Empty
            }}
        </section>
    }
}

#[component]
fn App() -> View {
    let theme = use_signal(Theme::default());

    provide_theme(theme.get());

    view! {
        <div class="app" style={theme_css_vars(&theme.get())}>
            <a class="skip-link" href="#main">"Skip to tasks"</a>
            <header class="hero">
                <div class="hero-top">
                    <div>
                        <p class="eyebrow">"Resuma showcase"</p>
                        <h1>"Todo"</h1>
                    </div>
                    <button
                        type="button"
                        class="theme-toggle"
                        aria-label={if theme.get().mode == "dark" { "Switch to light theme" } else { "Switch to dark theme" }}
                        onClick={
                            js! {
                                const t = state.theme.value;
                                if (t.mode === "dark") {
                                    state.theme.set({
                                        mode: "light",
                                        primary: "#4f46e5",
                                        background: "#f4f7fb",
                                        foreground: "#0f172a",
                                    });
                                } else {
                                    state.theme.set({
                                        mode: "dark",
                                        primary: "#818cf8",
                                        background: "#0c1022",
                                        foreground: "#e8ecf7",
                                    });
                                }
                            }
                        }
                    >
                        {if theme.get().mode == "dark" { "☀ Light" } else { "🌙 Dark" }}
                    </button>
                </div>
                <p class="lead">
                    "SSR-first task list with resumable interactivity — no hydration, minimal JS until you interact."
                </p>
                <FeatureBadges />
                <SessionBar />
                <SecurityNote />
            </header>

            <main id="main">
                {todo_workspace()}
            </main>

            <footer class="foot">
                <p>"Backend security reference — see " <code>"examples/todo/src/security.rs"</code> " and " <code>"docs/SECURITY.md"</code> "."</p>
            </footer>
        </div>
    }
}

const CSS: &str = r##"<style>
:root {
  color-scheme: light dark;
  --radius: 12px;
  --shadow: 0 12px 40px color-mix(in srgb, var(--resuma-fg) 8%, transparent);
  --transition: 180ms ease;
}
* { box-sizing: border-box; }
body {
  margin: 0;
  font-family: "Segoe UI", ui-sans-serif, system-ui, sans-serif;
  background:
    radial-gradient(1200px 600px at 10% -10%, color-mix(in srgb, var(--resuma-primary) 18%, transparent), transparent),
    var(--resuma-bg);
  color: var(--resuma-fg);
  line-height: 1.5;
}
.app {
  min-height: 100vh;
  max-width: 640px;
  margin: 0 auto;
  padding: 1.5rem 1rem 3rem;
}
.skip-link {
  position: absolute;
  left: -9999px;
  top: 0;
  background: var(--resuma-primary);
  color: white;
  padding: .5rem .75rem;
  border-radius: 0 0 8px 8px;
  text-decoration: none;
  z-index: 100;
}
.skip-link:focus { left: 1rem; }
.hero-top { display: flex; align-items: flex-start; justify-content: space-between; gap: 1rem; }
.eyebrow { margin: 0; font-size: .75rem; letter-spacing: .08em; text-transform: uppercase; opacity: .65; }
.hero h1 { margin: .15rem 0 0; font-size: clamp(2rem, 5vw, 2.5rem); line-height: 1.1; }
.lead { opacity: .82; margin: .75rem 0 1rem; max-width: 42ch; }
.badges-details { margin: 0; }
.badges-details summary {
  cursor: pointer;
  font-size: .85rem;
  opacity: .75;
  user-select: none;
}
.badges {
  display: flex; flex-wrap: wrap; gap: .35rem;
  list-style: none; padding: .75rem 0 0; margin: 0;
}
.badges li {
  font-size: .72rem; padding: .25rem .55rem; border-radius: 999px;
  background: color-mix(in srgb, var(--resuma-primary) 20%, transparent);
  border: 1px solid color-mix(in srgb, var(--resuma-primary) 35%, transparent);
}
.theme-toggle {
  background: color-mix(in srgb, var(--resuma-fg) 6%, transparent);
  border: 1px solid color-mix(in srgb, var(--resuma-fg) 18%, transparent);
  color: inherit; border-radius: 999px; padding: .45rem .85rem;
  cursor: pointer; transition: background var(--transition), transform var(--transition);
  white-space: nowrap;
}
.theme-toggle:hover { background: color-mix(in srgb, var(--resuma-fg) 10%, transparent); }
.theme-toggle:active { transform: scale(.98); }
.workspace {
  background: color-mix(in srgb, var(--resuma-bg) 82%, var(--resuma-fg));
  border: 1px solid color-mix(in srgb, var(--resuma-fg) 12%, transparent);
  border-radius: calc(var(--radius) + 4px);
  padding: 1.25rem;
  margin-top: 1rem;
  box-shadow: var(--shadow);
}
.workspace-head { display: grid; gap: .75rem; margin-bottom: 1rem; }
.stat-pills { display: flex; flex-wrap: wrap; gap: .5rem; }
.pill {
  font-size: .82rem; padding: .35rem .65rem; border-radius: 999px;
  background: color-mix(in srgb, var(--resuma-fg) 8%, transparent);
}
.pill--pending strong { color: var(--resuma-primary); }
.pill--done strong { color: #22c55e; }
.progress-ring__track {
  height: 8px; border-radius: 999px; overflow: hidden;
  background: color-mix(in srgb, var(--resuma-fg) 10%, transparent);
}
.progress-ring__fill {
  height: 100%; border-radius: inherit;
  background: linear-gradient(90deg, var(--resuma-primary), #22c55e);
  transition: width 320ms ease;
}
.progress-ring__label { display: block; margin-top: .35rem; font-size: .78rem; opacity: .7; }
.flash {
  display: flex; align-items: center; justify-content: space-between; gap: .75rem;
  margin: 0 0 .85rem; padding: .55rem .75rem; border-radius: var(--radius);
  background: color-mix(in srgb, var(--resuma-primary) 16%, transparent);
  border: 1px solid color-mix(in srgb, var(--resuma-primary) 35%, transparent);
  font-size: .9rem;
}
.flash-dismiss {
  background: transparent; border: 0; color: inherit; cursor: pointer;
  font-size: 1.1rem; line-height: 1; opacity: .7; padding: .15rem .35rem;
}
.add-form { display: flex; gap: .5rem; margin-bottom: .35rem; }
.add-form input, .search, .edit-input {
  flex: 1; padding: .65rem .85rem; border-radius: var(--radius);
  border: 1px solid color-mix(in srgb, var(--resuma-fg) 18%, transparent);
  background: var(--resuma-bg); color: inherit; font: inherit;
  transition: border-color var(--transition), box-shadow var(--transition);
}
.add-form input:focus, .search:focus, .edit-input:focus {
  outline: none;
  border-color: var(--resuma-primary);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--resuma-primary) 25%, transparent);
}
.btn-primary {
  background: var(--resuma-primary); color: white; border: 0;
  border-radius: var(--radius); padding: .65rem 1rem; cursor: pointer;
  font-weight: 600; transition: opacity var(--transition), transform var(--transition);
}
.btn-primary:disabled { opacity: .45; cursor: not-allowed; }
.btn-primary:not(:disabled):hover { filter: brightness(1.06); }
.btn-primary:not(:disabled):active { transform: scale(.98); }
.form-hint { margin: 0 0 1rem; font-size: .78rem; opacity: .6; }
.toolbar { display: grid; gap: .65rem; margin-bottom: 1rem; }
.search-wrap { position: relative; display: flex; }
.search-wrap .search { width: 100%; padding-right: 2.25rem; }
.search-clear {
  position: absolute; right: .35rem; top: 50%; transform: translateY(-50%);
  background: transparent; border: 0; color: inherit; opacity: .55;
  cursor: pointer; font-size: 1.2rem; line-height: 1; padding: .25rem .45rem;
}
.filters { display: flex; flex-wrap: wrap; gap: .35rem; }
.filter-btn, .btn-ghost {
  background: transparent; color: inherit;
  border: 1px solid color-mix(in srgb, var(--resuma-fg) 18%, transparent);
  border-radius: 999px; padding: .4rem .75rem; cursor: pointer; font-size: .85rem;
  transition: background var(--transition), color var(--transition), border-color var(--transition);
}
.filter-btn.active, .filter-btn[aria-pressed="true"] {
  background: var(--resuma-primary); color: white; border-color: transparent;
}
.btn-ghost--danger:hover { border-color: #ef4444; color: #ef4444; }
.bulk-actions { display: flex; flex-wrap: wrap; gap: .5rem; }
.todo-list { list-style: none; padding: 0; margin: 0; display: grid; gap: .5rem; }
.todo-item {
  display: grid; grid-template-columns: auto 1fr auto; align-items: center; gap: .65rem;
  padding: .65rem .75rem; border-radius: var(--radius);
  background: var(--resuma-bg);
  border: 1px solid color-mix(in srgb, var(--resuma-fg) 10%, transparent);
  animation: fade-in 240ms ease both;
}
@keyframes fade-in {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after { animation: none !important; transition: none !important; }
}
.todo-item.done .todo-title { text-decoration: line-through; opacity: .62; }
.todo-check { position: relative; display: inline-flex; cursor: pointer; }
.todo-check input {
  position: absolute; opacity: 0; width: 1.15rem; height: 1.15rem; cursor: pointer;
}
.check-ui {
  width: 1.15rem; height: 1.15rem; border-radius: .35rem;
  border: 2px solid color-mix(in srgb, var(--resuma-fg) 35%, transparent);
  display: inline-block; transition: background var(--transition), border-color var(--transition);
}
.todo-check input:checked + .check-ui {
  background: var(--resuma-primary);
  border-color: var(--resuma-primary);
  box-shadow: inset 0 0 0 2px var(--resuma-bg);
}
.todo-title { word-break: break-word; padding: .15rem 0; }
.edit-form { display: flex; gap: .35rem; align-items: center; width: 100%; }
.todo-actions { display: flex; gap: .15rem; opacity: .55; transition: opacity var(--transition); }
.todo-item:hover .todo-actions, .todo-item:focus-within .todo-actions { opacity: 1; }
.btn-icon {
  background: transparent; border: 0; cursor: pointer; color: inherit;
  border-radius: 8px; padding: .25rem .4rem; font-size: .95rem;
}
.btn-icon:hover { background: color-mix(in srgb, var(--resuma-fg) 8%, transparent); }
.btn-icon--danger:hover { background: color-mix(in srgb, #ef4444 15%, transparent); }
.empty {
  text-align: center; padding: 1.25rem .5rem .5rem; opacity: .75;
}
.empty p { margin: 0 0 .5rem; }
.btn-link {
  background: transparent; border: 0; color: var(--resuma-primary);
  cursor: pointer; text-decoration: underline; font: inherit;
}
.foot { margin-top: 1.5rem; font-size: .84rem; opacity: .72; max-width: 48ch; }
.security-note {
  margin-top: 1rem; padding: 1rem 1.1rem; border-radius: var(--radius);
  border: 1px solid color-mix(in srgb, #22c55e 35%, transparent);
  background: color-mix(in srgb, #22c55e 8%, transparent);
}
.security-note h2 { margin: 0 0 .5rem; font-size: .95rem; }
.security-note ul { margin: 0; padding-left: 1.1rem; font-size: .85rem; opacity: .9; }
.security-note li { margin: .2rem 0; }
.security-note__hint { margin: .65rem 0 0; font-size: .78rem; opacity: .75; }
.security-note code { font-size: .78rem; }
.session-bar {
  display: flex; flex-wrap: wrap; align-items: center; gap: .4rem;
  margin-top: .75rem; padding: .65rem .75rem; border-radius: var(--radius);
  background: color-mix(in srgb, var(--resuma-fg) 5%, transparent);
  border: 1px solid color-mix(in srgb, var(--resuma-fg) 12%, transparent);
  font-size: .85rem;
}
.session-bar__label { opacity: .75; margin-right: .15rem; }
.session-bar__hint { opacity: .55; font-size: .78rem; margin-left: .25rem; }
.session-btn {
  background: transparent; border: 1px solid color-mix(in srgb, var(--resuma-fg) 20%, transparent);
  color: inherit; border-radius: 999px; padding: .25rem .65rem; cursor: pointer; font-size: .82rem;
}
.session-btn:hover { border-color: var(--resuma-primary); color: var(--resuma-primary); }
.sr-only {
  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0,0,0,0); white-space: nowrap; border: 0;
}
</style>"##;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    security::install();

    let site_url = std::env::var("SITE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".into());

    ResumaApp::new()
        .with_title("Resuma · Todo")
        .with_description("Secure resumable todo — CSRF, rate limits, validated server actions")
        .with_site_url(site_url)
        .with_head(CSS)
        .page("/", || App::render(AppProps::default()))
        .serve(security::serve_options())
        .await
}
