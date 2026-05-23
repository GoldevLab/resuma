use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"HTTP API Reference"</h1>
            <p class="lead">
                "Built-in endpoints on every Resuma and Flow app. Flow also serves "
                <code>"/robots.txt"</code> " and " <code>"/sitemap.xml"</code>"."
            </p>

            <h2>"Runtime assets"</h2>
            <table class="docs-table">
                <thead>
                    <tr><th>"Method"</th><th>"Path"</th><th>"Description"</th></tr>
                </thead>
                <tbody>
                    <tr>
                        <td><code>"GET"</code></td>
                        <td><code>"/_resuma/loader.js"</code></td>
                        <td>"Event bootstrap (~1-2 KB gzipped). First script on interactive pages."</td>
                    </tr>
                    <tr>
                        <td><code>"GET"</code></td>
                        <td><code>"/_resuma/core.js"</code></td>
                        <td>"Resumability core, lazy-loaded on first interaction."</td>
                    </tr>
                    <tr>
                        <td><code>"GET"</code></td>
                        <td><code>"/_resuma/runtime.js"</code></td>
                        <td>"Legacy combined loader and core."</td>
                    </tr>
                    <tr>
                        <td><code>"GET"</code></td>
                        <td><code>"/_resuma/benchmark.json"</code></td>
                        <td>"Bundle metrics. Hidden in production mode."</td>
                    </tr>
                </tbody>
            </table>

            <h2>"Server actions"</h2>
            <table class="docs-table">
                <thead>
                    <tr><th>"Method"</th><th>"Path"</th><th>"Description"</th></tr>
                </thead>
                <tbody>
                    <tr>
                        <td><code>"POST"</code></td>
                        <td><code>"/_resuma/action/:name"</code></td>
                        <td><code>"#[server]"</code> " RPC. JSON body with args array. CSRF required."</td>
                    </tr>
                </tbody>
            </table>

            <h3>"Action request example"</h3>
            {code_block(r##"POST /_resuma/action/add_todo
Content-Type: application/json
X-Resuma-CSRF: <token>

{"args": ["Buy milk"]}"##)}

            <h3>"Action response shape"</h3>
            {code_block(r##"// 200 OK
{"ok": true, "value": [...], "error": null}

// 401 / 403 / 429 / 422
{"ok": false, "value": null, "error": "Forbidden"}"##)}

            <h2>"Form submits (Flow only)"</h2>
            <table class="docs-table">
                <thead>
                    <tr><th>"Method"</th><th>"Path"</th><th>"Description"</th></tr>
                </thead>
                <tbody>
                    <tr>
                        <td><code>"POST"</code></td>
                        <td><code>"/_resuma/submit/:name"</code></td>
                        <td><code>"#[submit]"</code> " handler. Form or JSON. CSRF required."</td>
                    </tr>
                </tbody>
            </table>
            <p>"See " <a href="/docs/flow/endpoints">"Flow endpoints"</a>"."</p>

            <h2>"Lazy chunks"</h2>
            <table class="docs-table">
                <thead>
                    <tr><th>"Method"</th><th>"Path"</th><th>"Description"</th></tr>
                </thead>
                <tbody>
                    <tr>
                        <td><code>"GET"</code></td>
                        <td><code>"/_resuma/handler/:chunk"</code></td>
                        <td>"Handler JS, lazy on first event"</td>
                    </tr>
                    <tr>
                        <td><code>"GET"</code></td>
                        <td><code>"/_resuma/island/:chunk"</code></td>
                        <td>"Island bundle for " <code>"#[island]"</code></td>
                    </tr>
                </tbody>
            </table>

            <h2>"SEO routes (Flow only)"</h2>
            <table class="docs-table">
                <thead>
                    <tr><th>"Method"</th><th>"Path"</th><th>"Description"</th></tr>
                </thead>
                <tbody>
                    <tr>
                        <td><code>"GET"</code></td>
                        <td><code>"/robots.txt"</code></td>
                        <td>"Crawler rules and sitemap link"</td>
                    </tr>
                    <tr>
                        <td><code>"GET"</code></td>
                        <td><code>"/sitemap.xml"</code></td>
                        <td>"XML sitemap from Flow SEO config"</td>
                    </tr>
                </tbody>
            </table>

            <h2>"Security headers"</h2>
            <p>"Applied by default. Configure via " <a href="/docs/security/configure">"SecurityConfig"</a>"."</p>
            <ul>
                <li>"CSP with per-request nonce on HTML pages"</li>
                <li><code>"Strict-Transport-Security"</code> " when HTTPS is detected"</li>
                <li><code>"X-Frame-Options: DENY"</code></li>
                <li><code>"X-Content-Type-Options: nosniff"</code></li>
                <li>"Rate limiting on POST actions and submits"</li>
            </ul>

            <h2>"CSRF"</h2>
            <p>
                "SSR embeds csrf_token in the resuma state script and sets the "
                <code>"__resuma-csrf"</code> " cookie. Send header "
                <code>"X-Resuma-CSRF"</code> " on mutations (automatic in client runtime)."
            </p>
        </>
    }
}
