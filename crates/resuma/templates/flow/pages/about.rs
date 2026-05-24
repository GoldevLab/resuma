use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <article class="card">
            <h1>"About"</h1>
            <p>"Route: " <code>"src/pages/about.rs"</code> " → " <code>"/about"</code></p>
        </article>
    }
}
