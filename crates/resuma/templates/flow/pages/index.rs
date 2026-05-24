use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <article class="card">
            <h1>"Hello, Flow"</h1>
            <p>"File-based page: " <code>"src/pages/index.rs"</code> " → " <code>"/"</code></p>
        </article>
    }
}
