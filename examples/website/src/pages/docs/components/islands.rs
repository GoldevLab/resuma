use resuma::prelude::*;
use crate::site::code_block;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Islands"</h1>
            <p class="lead">"Islands are independently resumable component boundaries that ship as separate JS chunks."</p>

            <h2>"#[island]"</h2>
            {code_block(r#"#[island]
fn LiveChart() -> View {
    let points = use_signal(vec![1, 4, 2, 8]);
    view! {
        <svg class="chart">
            {points.get().iter().map(|p| view! {
                <rect height={*p} />
            }).collect::<Vec<_>>()}
        </svg>
    }
}"#)}

            <h2>"Usage"</h2>
            {code_block(r#"view! {
    <article>
        <h1>"Static SSR content"</h1>
        <LiveChart />
    </article>
}"#)}

            <h2>"Runtime behavior"</h2>
            <p>"SSR wraps the island in " <code>"<resuma-island>"</code> " with a chunk reference. After bootstrap, the runtime dynamically imports " <code>"/_resuma/island/:chunk"</code> " and calls resume on the island boundary."</p>

            <h2>"When to use islands"</h2>
            <ul>
                <li>"Heavy client-only widgets (charts, editors)"</li>
                <li>"Third-party JS integration boundaries"</li>
                <li>"Code that should not block the main page payload"</li>
            </ul>
        </>
    }
}
