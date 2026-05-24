use crate::site::code_block;
use resuma::prelude::*;

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
                <li><strong>"view! expansion"</strong>" — closure → rs2js (in resuma-macros) → HandlerRef in HTML"</li>
                <li><strong>"SSR"</strong>" — walk View tree, emit data-r-on:* attributes + JSON payload"</li>
                <li><strong>"Runtime"</strong>" — document listener, lazy fetch handler chunk, update signals"</li>
            </ol>

            <h2>"Payload format"</h2>
            {code_block(r#"<script type="resuma/state" id="resuma-state">
{"signals":[...],"handlers":{},"lazy_chunks":["Counter"],"islands":[],"actions":[]}
</script>
<script type="module" src="/_resuma/loader.js"></script>"#)}

            <h2>"Component boundaries"</h2>
            <p>"Each " <code>"#[component]"</code> " emits " <code>"<resuma-boundary data-r-chunk=\"Counter\">"</code> " for viewport prefetch. Handler JS loads from " <code>"/_resuma/handler/Counter.js"</code> " — not inlined in the payload."</p>

            <h2>"Crates"</h2>
            <table class="docs-table">
                <thead><tr><th>"Crate / module"</th><th>"Role"</th></tr></thead>
                <tbody>
                    <tr><td><code>"resuma"</code></td><td>"Single runtime crate — depend on this only"</td></tr>
                    <tr><td><code>"resuma::core"</code></td><td>"Signals, View, resumability primitives"</td></tr>
                    <tr><td><code>"resuma::ssr"</code></td><td>"HTML rendering + streaming chunks"</td></tr>
                    <tr><td><code>"resuma::server"</code></td><td>"axum HTTP, /_resuma/* endpoints"</td></tr>
                    <tr><td><code>"resuma::flow"</code></td><td>"FlowApp, pages, loads, submits"</td></tr>
                    <tr><td><code>"resuma::router"</code></td><td>"File-based page scanner"</td></tr>
                    <tr><td><code>"resuma-macros"</code></td><td>"view!, #[component], #[load], #[submit] (proc-macros)"</td></tr>
                </tbody>
            </table>

            <h2>"Resumability vs hydration"</h2>
            <table class="docs-table">
                <thead><tr><th>"Aspect"</th><th>"Classic SSR + hydration"</th><th>"Resuma"</th></tr></thead>
                <tbody>
                    <tr><td>"Client after load"</td><td>"Re-run components"</td><td>"Resume handlers only"</td></tr>
                    <tr><td>"Initial JS"</td><td>"App bundle grows with UI"</td><td>"~3KB runtime + lazy chunks"</td></tr>
                    <tr><td>"Static pages"</td><td>"Often still ship framework JS"</td><td>"Zero client JS"</td></tr>
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
