use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Flow Endpoints"</h1>
            <p class="lead">"HTTP paths that connect browser forms and client code to Rust submit and server handlers."</p>

            <h2>"Submit endpoints"</h2>
            <table class="docs-table">
                <thead>
                    <tr><th>"Method"</th><th>"Path"</th><th>"Handler"</th></tr>
                </thead>
                <tbody>
                    <tr>
                        <td><code>"POST"</code></td>
                        <td><code>"/_resuma/submit/:name"</code></td>
                        <td><code>"#[submit]"</code>" function named " <code>":name"</code></td>
                    </tr>
                </tbody>
            </table>
            <p>"Accepts " <code>"application/x-www-form-urlencoded"</code> " (native form POST) or JSON when " <code>"Accept: application/json"</code> " is set."</p>

            <h2>"Server action endpoints"</h2>
            <table class="docs-table">
                <thead>
                    <tr><th>"Method"</th><th>"Path"</th><th>"Handler"</th></tr>
                </thead>
                <tbody>
                    <tr>
                        <td><code>"POST"</code></td>
                        <td><code>"/_resuma/action/:name"</code></td>
                        <td><code>"#[server]"</code>" function named " <code>":name"</code></td>
                    </tr>
                </tbody>
            </table>

            <h2>"Response shapes"</h2>
            <p>"Submit JSON responses include " <code>"ok"</code> ", " <code>"value"</code> ", " <code>"error"</code> ", and " <code>"field_errors"</code> ". Action responses return the serialized Rust return value as JSON."</p>

            <h2>"Full API"</h2>
            <p>"See " <a href="/docs/api">"API reference"</a> " for runtime asset endpoints (loader.js, handler chunks, islands)."</p>
        </>
    }
}
