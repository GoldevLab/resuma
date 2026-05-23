use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Todo example — backend reference"</h1>
            <p class="lead">
                <code>"cargo run -p example-todo"</code> " — full Resuma showcase plus production backend patterns (NestJS / Next.js equivalents in Rust)."
            </p>

            <h2>"Files"</h2>
            <table class="docs-table">
                <thead><tr><th>"File"</th><th>"Role"</th></tr></thead>
                <tbody>
                    <tr><td><code>"src/main.rs"</code></td><td>"UI, " <code>"#[server]"</code> " controllers, island"</td></tr>
                    <tr><td><code>"src/todo_store.rs"</code></td><td>"Service layer + DTO validation"</td></tr>
                    <tr><td><code>"src/security.rs"</code></td><td>"Guards, interceptors, SecurityConfig"</td></tr>
                </tbody>
            </table>

            <h2>"Try it"</h2>
            <ol>
                <li>"Run the app, open DevTools → Network"</li>
                <li>"Default user: guest — sees only guest tasks"</li>
                <li>"Click alice — admin, sees all tasks"</li>
                <li>"As guest, try toggling alice's task → 403 Forbidden"</li>
                <li>"Check terminal logs: request id + user + IP per action"</li>
            </ol>
            {code_block("cargo run -p example-todo")}

            <h2>"Patterns implemented"</h2>
            <ul>
                <li><strong>"Guard"</strong>" — " <code>"attach_session()"</code> " in action middleware"</li>
                <li><strong>"ValidationPipe"</strong>" — " <code>"AddTodoInput"</code> ", " <code>"RenameTodoInput"</code></li>
                <li><strong>"Service"</strong>" — " <code>"todo_store::add()"</code> " etc."</li>
                <li><strong>"Controller"</strong>" — thin " <code>"#[server]"</code> " → delegates to store"</li>
                <li><strong>"Server Action"</strong>" — " <code>"__resuma.action()"</code> " with CSRF"</li>
                <li><strong>"Revalidate"</strong>" — " <code>"list_todos"</code> " on island mount via " <code>"use_visible_task"</code></li>
                <li><strong>"Interceptor"</strong>" — request id + audit log"</li>
                <li><strong>"Exception filter"</strong>" — " <code>"Result<T>"</code> " → HTTP status"</li>
            </ul>

            <h2>"Env vars"</h2>
            <table class="docs-table">
                <thead><tr><th>"Variable"</th><th>"Purpose"</th></tr></thead>
                <tbody>
                    <tr><td><code>"RESUMA_ENV=production"</code></td><td>"Sanitized client error messages"</td></tr>
                    <tr><td><code>"RESUMA_TRUST_PROXY=1"</code></td><td>"Real client IP behind Fly/nginx"</td></tr>
                    <tr><td><code>"RESUMA_TODO_ADMINS"</code></td><td>"Admin users (default: alice)"</td></tr>
                    <tr><td><code>"RESUMA_TODO_API_KEY"</code></td><td>"Optional shared secret for actions"</td></tr>
                </tbody>
            </table>

            <p>"Conceptual map: " <a href="/docs/security/backend_patterns">"NestJS + Next.js → Resuma"</a> " · Authorization: " <a href="/docs/security/authorization">"RLS guide"</a>"."</p>
        </>
    }
}
