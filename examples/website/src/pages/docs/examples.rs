use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Examples"</h1>
            <p class="lead">
                "Runnable crates in the repo — clone and run with "
                <code>"cargo run -p PACKAGE"</code>". All listen on "
                <code>"http://127.0.0.1:3000"</code> " by default."
            </p>

            <table class="docs-table">
                <thead>
                    <tr>
                        <th>"Example"</th>
                        <th>"Command"</th>
                        <th>"App type"</th>
                        <th>"What it demonstrates"</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td><strong>"todo"</strong></td>
                        <td><code>"cargo run -p example-todo"</code></td>
                        <td>"ResumaApp"</td>
                        <td>
                            "Full showcase: signals, "
                            <code>"#[server]"</code> ", "
                            <code>"#[island]"</code> ", "
                            <code>"js!"</code> ", theme, backend security (guards, DTOs, service layer). "
                            <a href="/docs/security/todo">"Docs →"</a>
                        </td>
                    </tr>
                    <tr>
                        <td><strong>"counter"</strong></td>
                        <td><code>"cargo run -p example-counter"</code></td>
                        <td>"ResumaApp"</td>
                        <td>"Minimal resumable counter — smallest interactive app."</td>
                    </tr>
                    <tr>
                        <td><strong>"flow-demo"</strong></td>
                        <td><code>"cargo run -p example-flow-demo"</code></td>
                        <td>"FlowApp"</td>
                        <td>
                            <code>"#[load]"</code> ", streaming SSR, "
                            <code>"#[load(stream)]"</code> ", deferred chunks. "
                            <a href="/docs/flow/streaming">"Docs →"</a>
                        </td>
                    </tr>
                    <tr>
                        <td><strong>"flow-pages"</strong></td>
                        <td><code>"cargo run -p example-flow-pages"</code></td>
                        <td>"FlowApp"</td>
                        <td>
                            "File-based routing, layouts, "
                            <code>"auto_pages"</code> ", "
                            <code>"resuma routes --generate"</code>". "
                            <a href="/docs/flow/pages">"Docs →"</a>
                        </td>
                    </tr>
                    <tr>
                        <td><strong>"website"</strong></td>
                        <td><code>"cargo run -p example-website"</code></td>
                        <td>"FlowApp"</td>
                        <td>
                            "This documentation site. Deployed at "
                            <a href="https://resuma-docs.fly.dev/" target="_blank">"resuma-docs.fly.dev"</a>". "
                            <a href="/docs/cookbook/docker">"Deploy →"</a>
                        </td>
                    </tr>
                </tbody>
            </table>

            <h2>"Choose an example"</h2>
            <ul>
                <li><strong>"Learning Resuma?"</strong>" → " <code>"counter"</code> " then " <code>"todo"</code></li>
                <li><strong>"Production backend patterns?"</strong>" → " <code>"todo"</code> " + " <a href="/docs/security">"Security docs"</a></li>
                <li><strong>"Multi-page site?"</strong>" → " <code>"flow-pages"</code> " or " <code>"website"</code></li>
                <li><strong>"Streaming / loaders?"</strong>" → " <code>"flow-demo"</code></li>
            </ul>

            <h2>"CLI templates"</h2>
            <p>
                <code>"resuma new my-app --template basic"</code> " scaffolds like a minimal "
                <code>"counter"</code>". "
                <code>"--template todo"</code> " copies the " <code>"todo"</code> " example (main + security + todo_store)."
            </p>
        </>
    }
}
