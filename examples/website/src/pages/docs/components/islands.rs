use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Islands (optional)"</h1>
            <p class="lead">"Resumability is the default for every " <code>"#[component]"</code> ". Islands are an optional extra boundary for heavy lazy JS, " <code>"load = \"visible\""</code> ", or dev HMR."</p>

            <h2>"Default: resumable components"</h2>
            {code_block(r#"#[component]
fn Counter() -> View {
    let n = use_signal(0);
    view! {
        <button onClick={move |_| n.update(|v| *v += 1)}>"+"</button>
    }
}
// Handlers lazy-load from /_resuma/handler/Counter.js — no #[island] needed."#)}

            <h2>"#[island] — when you need more"</h2>
            {code_block(r#"#[island(load = "visible")]
fn LiveChart() -> View {
    let points = use_signal(vec![1, 4, 2, 8]);
    view! { /* heavy widget */ }
}"#)}

            <h2>"When to use islands"</h2>
            <ul>
                <li>"Very heavy client-only widgets (charts, editors)"</li>
                <li>"Defer JS until visible (" <code>"load = \"visible\""</code> ")"</li>
                <li>"Dev HMR refresh via " <code>"/_resuma/island/:instance"</code></li>
            </ul>
            <p>"For counters, forms, filters, and most UI — " <code>"#[component]"</code> " + " <code>"computed!"</code> " / " <code>"effect!"</code> " is enough."</p>
        </>
    }
}
