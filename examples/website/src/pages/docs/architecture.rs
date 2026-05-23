use resuma::prelude::*;
use crate::site::code_block;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Architecture"</h1>
            <p class="lead">"How Resuma turns Rust components into instantly-interactive HTML without hydration."</p>

            <h2>"The resumability promise"</h2>
            <p>"Traditional SSR: render on server → hydrate on client (re-run all components). Resuma: render once → serialize state → client resumes only what the user touches."</p>
            {code_block(r#"Server (Rust)  ──HTML + payload──►  Browser (~3KB)
render components              parse resuma/state
serialize signals              delegate events
                               lazy-import handlers"#)}

            <h2>"Pipeline of one click"</h2>
            <ol>
                <li><strong>"view! expansion"</strong>" — closure → resuma-rs2js → HandlerRef in HTML"</li>
                <li><strong>"SSR"</strong>" — walk View tree, emit data-r-on:* attributes + JSON payload"</li>
                <li><strong>"Runtime"</strong>" — document listener, lazy fetch handler chunk, update signals"</li>
            </ol>

            <h2>"Payload format"</h2>
            {code_block(r#"<script type="resuma/state" id="resuma-state">
{"signals":[...],"handlers":{...},"islands":[],"actions":[]}
</script>
<script type="module" src="/_resuma/loader.js"></script>"#)}

            <h2>"Crates"</h2>
            <table class="docs-table">
                <thead><tr><th>"Crate"</th><th>"Role"</th></tr></thead>
                <tbody>
                    <tr><td>"resuma-core"</td><td>"Signals, View, resumability primitives"</td></tr>
                    <tr><td>"resuma-macros"</td><td>"view!, #[component], #[load], #[submit]"</td></tr>
                    <tr><td>"resuma-ssr"</td><td>"HTML rendering + streaming chunks"</td></tr>
                    <tr><td>"resuma-server"</td><td>"axum HTTP, /_resuma/* endpoints"</td></tr>
                    <tr><td>"resuma-flow"</td><td>"FlowApp, pages, loads, submits"</td></tr>
                    <tr><td>"resuma"</td><td>"Umbrella — depend on this only"</td></tr>
                </tbody>
            </table>

            <h2>"HTTP endpoints"</h2>
            <ul>
                <li><code>"GET /_resuma/runtime.js"</code>" — client bootstrap"</li>
                <li><code>"POST /_resuma/action/:name"</code>" — #[server] RPC"</li>
                <li><code>"POST /_resuma/submit/:name"</code>" — #[submit] forms"</li>
                <li><code>"GET /_resuma/handler/:chunk"</code>" — lazy handler JS"</li>
            </ul>
        </>
    }
}
