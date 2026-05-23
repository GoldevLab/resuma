use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Documentation"</h1>
            <p class="lead">"Everything you need to build resumable Rust web apps — from a counter to a full-stack Flow site."</p>

            <h2>"Quick links"</h2>
            <div class="grid-3">
                <a href="/docs/getting_started" class="card" style="text-decoration: none;">
                    <h3>"Getting Started Resumably"</h3>
                    <p>"Prerequisites, resuma new/create, resuma dev — your first app in 5 minutes."</p>
                </a>
                <a href="/docs/package" class="card" style="text-decoration: none;">
                    <h3>"Resuma + Flow"</h3>
                    <p>"One cargo dependency. Core + full-stack — how the package map works."</p>
                </a>
                <a href="/docs/flow" class="card" style="text-decoration: none;">
                    <h3>"Resuma Flow"</h3>
                    <p>"#[load], #[submit], layouts, middleware, file-based routing."</p>
                </a>
            </div>

            <h2>"What is Resuma?"</h2>
            <p>"Resuma is a Rust web framework built around resumability: the server renders components once, embeds signals and handler references in HTML, and a tiny client runtime resumes interactivity on demand — no hydration pass."</p>
            <p>"For multi-page apps, " <strong>"Resuma Flow"</strong> " adds Qwik City–style features with native naming: loads instead of routeLoader$, submits instead of routeAction$, FlowApp instead of a separate meta-framework crate."</p>
        </>
    }
}
