use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Effects"</h1>
            <p class="lead">"Effects re-run when tracked signals change. Computed values derive state without manual synchronization."</p>

            <h2>"use_effect"</h2>
            <p>"Runs immediately and again whenever a signal read inside the closure changes."</p>
            {code_block(r#"let query = use_signal(String::new());

use_effect(move || {
    let q = query.get();
    // Log, fetch, or sync side effects
    println!("query changed: {q}");
});"#)}

            <h2>"use_computed"</h2>
            <p>"Returns a read-only computed signal that updates when dependencies change."</p>
            {code_block(r#"let first = use_signal("Ada".into());
let last = use_signal("Lovelace".into());

let full_name = use_computed(move || {
    format!("{} {}", first.get(), last.get())
});

view! {
    <p>{full_name}</p>
}"#)}

            <h2>"SSR vs client"</h2>
            <p>"On the server, effects run during render to capture derived state. The client runtime mirrors the effect graph after resume. For browser-only work, prefer " <a href="/docs/components/tasks">"use_visible_task"</a>"."</p>
        </>
    }
}
