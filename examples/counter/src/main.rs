//! Resuma counter example.
//!
//! Demonstrates:
//!   * `#[component]` definition and `view!` template syntax.
//!   * `use_signal` reactive state captured by event handlers.
//!   * Resumability — the SSR HTML embeds a serialised state payload that the
//!     tiny client runtime resumes on first interaction.
//!
//! Run:
//!
//! ```sh
//! cargo run -p example-counter
//! # then open http://127.0.0.1:3000
//! ```

use resuma::prelude::*;

#[component]
fn Counter() -> View {
    let count = use_signal(0_i32);

    view! {
        <main class="card">
            <h1>"Resuma Counter"</h1>
            <p>"Current count: " {count}</p>
            <div class="row">
                <button onClick={ move |_| count.update(|c| *c -= 1) }>"-"</button>
                <button onClick={ move |_| count.update(|c| *c += 1) }>"+"</button>
                <button onClick={ move |_| count.set(0) }>"reset"</button>
            </div>
            <p class="hint">"This page is fully resumable — try opening DevTools, the only JS that loads is the ~3KB runtime."</p>
        </main>
    }
}

const INLINE_CSS: &str = r#"<style>
* { box-sizing: border-box; }
body { font-family: ui-sans-serif, system-ui, sans-serif; background: #0b1020; color: #e6e8ee; margin: 0; min-height: 100vh; display: grid; place-items: center; }
.card { background: #14182b; border: 1px solid #2a2f4a; padding: 2rem 2.5rem; border-radius: 16px; box-shadow: 0 24px 48px rgba(0,0,0,.35); }
.card h1 { margin: 0 0 .5rem; font-size: 1.5rem; }
.card p  { margin: .5rem 0; color: #b9bfd2; }
.card .row { display: flex; gap: .5rem; }
.card button { background: #6366f1; color: white; border: 0; border-radius: 8px; padding: .5rem .9rem; font-weight: 600; cursor: pointer; }
.card button:hover { background: #818cf8; }
.card .hint { font-size: .85rem; opacity: .7; max-width: 28ch; }
</style>"#;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    ResumaApp::new()
        .with_title("Resuma · Counter")
        .with_head(INLINE_CSS)
        .page("/", || Counter::render(CounterProps::default()))
        .serve(ServeOptions::default())
        .await
}
