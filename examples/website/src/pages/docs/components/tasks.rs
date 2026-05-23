use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Tasks"</h1>
            <p class="lead">"Tasks run side effects on the server during render, or register client-only work for after visibility."</p>

            <h2>"use_task"</h2>
            <p>"Alias for use_effect — runs during SSR and re-runs when tracked signals change."</p>
            {code_block(r#"let filter = use_signal("all".into());

use_task(move || {
    let f = filter.get();
    // Sync derived state, logging, etc.
    println!("filter: {f}");
});"#)}

            <h2>"use_visible_task"</h2>
            <p>"Registers a client-only task in the resumability payload. The runtime executes the JS body after the component becomes visible."</p>
            {code_block(r##"use_visible_task(r#"
    const el = document.querySelector('[data-chart]');
    if (el) initChart(el);
"#);"##)}

            <h2>"use_debounce"</h2>
            <p>"Debounce signal-driven callbacks — see the " <a href="/docs/cookbook/debouncer">"Debouncer cookbook"</a> " for a full search example."</p>
            {code_block(r#"let query = use_signal(String::new());

use_debounce(&query, 300, move |q| {
    if !q.is_empty() {
        println!("search: {q}");
    }
});"#)}

            <h2>"Choosing a hook"</h2>
            <ul>
                <li><code>"use_task"</code>" — server-safe effects tied to signals"</li>
                <li><code>"use_visible_task"</code>" — browser-only initialization (analytics, charts)"</li>
                <li><code>"use_debounce"</code>" — rate-limit expensive reactions to input"</li>
            </ul>
        </>
    }
}
