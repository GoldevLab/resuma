use resuma::prelude::*;
use crate::site::code_block;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Resuma Flow"</h1>
            <p class="lead">"Full-stack layer: pages, layouts, server data, forms, and middleware — included in the resuma crate."</p>

            <h2>"Topics"</h2>
            <div class="grid-3">
                <a href="/docs/flow/routing" class="card" style="text-decoration: none;">
                    <h3>"Routing"</h3>
                    <p>"File conventions: [id], [...slug]."</p>
                </a>
                <a href="/docs/flow/pages" class="card" style="text-decoration: none;">
                    <h3>"Pages"</h3>
                    <p>"auto_pages and PagesRegistry."</p>
                </a>
                <a href="/docs/flow/layouts" class="card" style="text-decoration: none;">
                    <h3>"Layouts"</h3>
                    <p>"#[layout], Slot, nested shells."</p>
                </a>
                <a href="/docs/flow/loaders" class="card" style="text-decoration: none;">
                    <h3>"Loaders"</h3>
                    <p>"#[load], use_*_load, LoadValue."</p>
                </a>
                <a href="/docs/flow/submits" class="card" style="text-decoration: none;">
                    <h3>"Actions"</h3>
                    <p>"#[submit], SubmitError, Form."</p>
                </a>
                <a href="/docs/flow/middleware" class="card" style="text-decoration: none;">
                    <h3>"Middleware"</h3>
                    <p>"Request pipeline hooks."</p>
                </a>
                <a href="/docs/flow/endpoints" class="card" style="text-decoration: none;">
                    <h3>"Endpoints"</h3>
                    <p>"Submit and action HTTP paths."</p>
                </a>
                <a href="/docs/flow/errors" class="card" style="text-decoration: none;">
                    <h3>"Error handling"</h3>
                    <p>"FlowError, not_found_page."</p>
                </a>
                <a href="/docs/flow/caching" class="card" style="text-decoration: none;">
                    <h3>"Caching"</h3>
                    <p>"Cache-Control on #[load]."</p>
                </a>
                <a href="/docs/flow/streaming" class="card" style="text-decoration: none;">
                    <h3>"Streaming"</h3>
                    <p>"Deferred #[load(stream)] SSR."</p>
                </a>
            </div>

            <h2>"Flow path"</h2>
            {code_block(r#"#[load]  →  SSR  →  resumability payload  →  user  →  #[submit]"#)}

            <h2>"Bootstrap"</h2>
            {code_block(r#"FlowApp::new()
    .with_title("My App")
    .streaming(true)
    .auto_pages("src/pages", PagesRegistry)
    .not_found(|| not_found_page())
    .serve(FlowServeOptions::default())
    .await"#)}
        </>
    }
}
