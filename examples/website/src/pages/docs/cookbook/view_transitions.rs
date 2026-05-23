use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"View Transitions"</h1>
            <p class="lead">"Wrap page content with with_view_transition for smooth animated navigation using the View Transitions API."</p>

            <h2>"Page wrapper"</h2>
            {code_block(r#"pub fn page(_req: FlowRequest) -> View {
    with_view_transition(
        "home",
        vec![Child::View(view! {
            <article class="page">
                <h1>"Home"</h1>
                <p>"Content animates on navigation."</p>
            </article>
        })],
    )
}"#)}

            <h2>"Unique transition names"</h2>
            <p>"Use a distinct name per route (e.g. " <code>"home"</code> ", " <code>"about"</code>") so the browser can cross-fade between pages."</p>

            <h2>"CSS"</h2>
            {code_block(r#"::view-transition-old(root) {
    animation: fade-out 200ms ease;
}
::view-transition-new(root) {
    animation: fade-in 200ms ease;
}"#)}

            <h2>"Fallback"</h2>
            <p>"Browsers without View Transitions support render content normally — no polyfill required."</p>
        </>
    }
}
