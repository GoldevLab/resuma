use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"NestJS + Next.js → Resuma"</h1>
            <p class="lead">
                "Conceptual mapping only. Every row is implemented in "
                <a href="/docs/security/todo">"examples/todo"</a>"."
            </p>

            <table class="docs-table">
                <thead>
                    <tr><th>"NestJS / Next.js"</th><th>"Resuma"</th><th>"Todo file"</th></tr>
                </thead>
                <tbody>
                    <tr><td>"Server Actions (Next)"</td><td><code>"#[server]"</code></td><td><code>"main.rs"</code></td></tr>
                    <tr><td>"revalidatePath (Next)"</td><td><code>"list_todos"</code> " + refetch"</td><td><code>"main.rs"</code></td></tr>
                    <tr><td>"middleware.ts (Next)"</td><td><code>"set_action_middleware"</code> " / " <code>"#[middleware]"</code></td><td><code>"security.rs"</code></td></tr>
                    <tr><td>"Controller (Nest)"</td><td>"Thin " <code>"#[server]"</code> " fn"</td><td><code>"main.rs"</code></td></tr>
                    <tr><td>"Service (Nest)"</td><td>"Domain module"</td><td><code>"todo_store.rs"</code></td></tr>
                    <tr><td>"Guard (Nest)"</td><td><code>"attach_session()"</code></td><td><code>"security.rs"</code></td></tr>
                    <tr><td>"ValidationPipe (Nest)"</td><td>"DTO + validate"</td><td><code>"todo_store.rs"</code></td></tr>
                    <tr><td>"Interceptor (Nest)"</td><td>"Request id + audit"</td><td><code>"security.rs"</code></td></tr>
                    <tr><td>"ExceptionFilter (Nest)"</td><td><code>"Result<T>"</code> " + " <code>"ResumaError"</code></td><td>"actions"</td></tr>
                    <tr><td>"ThrottlerModule (Nest)"</td><td><code>"SecurityConfig"</code></td><td><code>"security.rs"</code></td></tr>
                    <tr><td>"Helmet (Express)"</td><td><code>"SecurityConfig"</code> " headers"</td><td>"framework"</td></tr>
                </tbody>
            </table>

            <p><a href="/docs/security/todo">"Run the todo example →"</a></p>
        </>
    }
}
