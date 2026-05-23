use resuma::prelude::*;
use crate::site::code_block;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"HTTP API Reference"</h1>
            <p class="lead">"Built-in endpoints served by resuma-server on every Resuma and Flow app."</p>

            <h2>"Runtime assets"</h2>
            <table class="docs-table">
                <thead>
                    <tr><th>"Method"</th><th>"Path"</th><th>"Description"</th></tr>
                </thead>
                <tbody>
                    <tr>
                        <td><code>"GET"</code></td>
                        <td><code>"/_resuma/loader.js"</code></td>
                        <td>"Tiny event bootstrap (~1–2 KB). First script on interactive pages."</td>
                    </tr>
                    <tr>
                        <td><code>"GET"</code></td>
                        <td><code>"/_resuma/core.js"</code></td>
                        <td>"Lazy-loaded resumability core — fetched on first user interaction."</td>
                    </tr>
                    <tr>
                        <td><code>"GET"</code></td>
                        <td><code>"/_resuma/runtime.js"</code></td>
                        <td>"Legacy monolithic runtime (loader + core combined)."</td>
                    </tr>
                    <tr>
                        <td><code>"GET"</code></td>
                        <td><code>"/_resuma/benchmark.json"</code></td>
                        <td>"Bundle size metrics for tooling and docs benchmarks."</td>
                    </tr>
                </tbody>
            </table>

            <h2>"Server actions & forms"</h2>
            <table class="docs-table">
                <thead>
                    <tr><th>"Method"</th><th>"Path"</th><th>"Description"</th></tr>
                </thead>
                <tbody>
                    <tr>
                        <td><code>"POST"</code></td>
                        <td><code>"/_resuma/action/:name"</code></td>
                        <td>"Invoke a " <code>"#[server]"</code> " RPC handler. Body: " <code>"{ \"args\": [...] }"</code>"."</td>
                    </tr>
                    <tr>
                        <td><code>"POST"</code></td>
                        <td><code>"/_resuma/submit/:name"</code></td>
                        <td>"Submit a " <code>"#[submit]"</code> " form. Accepts form-urlencoded or JSON."</td>
                    </tr>
                </tbody>
            </table>

            <h2>"Lazy chunks"</h2>
            <table class="docs-table">
                <thead>
                    <tr><th>"Method"</th><th>"Path"</th><th>"Description"</th></tr>
                </thead>
                <tbody>
                    <tr>
                        <td><code>"GET"</code></td>
                        <td><code>"/_resuma/handler/:chunk"</code></td>
                        <td>"Handler JS chunk lazy-loaded when a user triggers an event."</td>
                    </tr>
                    <tr>
                        <td><code>"GET"</code></td>
                        <td><code>"/_resuma/island/:chunk"</code></td>
                        <td>"Island component JS bundle for " <code>"#[island]"</code> " boundaries."</td>
                    </tr>
                </tbody>
            </table>

            <h2>"Request format (actions)"</h2>
            <p>"Server actions expect JSON with an " <code>"args"</code> " array matching the Rust function parameters."</p>
            {code_block(r#"POST /_resuma/action/greet
Content-Type: application/json

{"args": ["World"]}"#)}
        </>
    }
}
