use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Backend patterns"</h1>
            <p class="lead">
                "How common backend responsibilities map to Resuma. Every row is implemented in "
                <a href="/docs/security/todo">"examples/todo"</a>"."
            </p>

            <table class="docs-table">
                <thead>
                    <tr><th>"Pattern"</th><th>"Resuma"</th><th>"Todo file"</th></tr>
                </thead>
                <tbody>
                    <tr><td>"Client RPC"</td><td><code>"#[server]"</code></td><td><code>"main.rs"</code></td></tr>
                    <tr><td>"Refetch after mutation"</td><td><code>"list_todos"</code> " + island reload"</td><td><code>"main.rs"</code></td></tr>
                    <tr><td>"Request pipeline"</td><td><code>"set_action_middleware"</code> " / " <code>"#[middleware]"</code></td><td><code>"security.rs"</code></td></tr>
                    <tr><td>"HTTP handler"</td><td>"Thin " <code>"#[server]"</code> " fn"</td><td><code>"main.rs"</code></td></tr>
                    <tr><td>"Domain logic"</td><td>"Service module"</td><td><code>"todo_store.rs"</code></td></tr>
                    <tr><td>"Auth check"</td><td><code>"attach_session()"</code></td><td><code>"security.rs"</code></td></tr>
                    <tr><td>"Input validation"</td><td>"DTO + validate"</td><td><code>"todo_store.rs"</code></td></tr>
                    <tr><td>"Request logging"</td><td>"Request id + audit"</td><td><code>"security.rs"</code></td></tr>
                    <tr><td>"Error mapping"</td><td><code>"Result<T>"</code> " + " <code>"ResumaError"</code></td><td>"actions"</td></tr>
                    <tr><td>"Rate limiting"</td><td><code>"SecurityConfig"</code></td><td><code>"security.rs"</code></td></tr>
                    <tr><td>"Security headers"</td><td><code>"SecurityConfig"</code> " defaults"</td><td>"framework"</td></tr>
                </tbody>
            </table>

            <p><a href="/docs/security/todo">"Run the todo example →"</a></p>
        </>
    }
}
