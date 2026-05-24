use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <article class="card">
            <h1>"Full-stack Flow"</h1>
            <p>"SQLx + SQLite, " <code>"#[load]"</code> ", and " <code>"#[submit]"</code> ". See " <a href="/users">"/users"</a>"."</p>
        </article>
    }
}
