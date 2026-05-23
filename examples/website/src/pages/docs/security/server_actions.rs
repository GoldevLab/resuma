use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Secure server actions"</h1>
            <p class="lead">
                "Every " <code>"#[server]"</code> " function is a public API. Validate input, return "
                <code>"Result<T>"</code> ", and use middleware for auth. Live code: "
                <a href="/docs/security/todo">"todo example"</a>"."
            </p>

            <h2>"1. Return Result (fail closed)"</h2>
            <p>"Do not silently ignore bad input. Return " <code>"Result<T, ResumaError>"</code> " — the framework maps errors to HTTP 401/403/422."</p>
            {code_block(r#"#[server]
async fn add_todo(title: String, req: &FlowRequest) -> Result<Vec<Todo>> {
    let title = security::normalize_title(&title)?;
    security::can_add_todo(store.len())?;
    todo_store::add(title, req)
}"#)}

            <h2>"2. DTO validation (ValidationPipe pattern)"</h2>
            {code_block(r#"pub struct AddTodoInput { pub title: String }

impl AddTodoInput {
    pub fn into_title(self) -> Result<String> {
        security::normalize_title(&self.title)
    }
}"#)}

            <h2>"3. FlowRequest for auth & audit"</h2>
            <p>"Add " <code>"req: &FlowRequest"</code> " as the last parameter."</p>
            {code_block(r#"let uid = req.user_id().ok_or(ResumaError::Unauthorized)?;
    security::assert_owner(&todo.owner_id, req)?;"#)}

            <h2>"4. Action middleware (ResumaApp guard)"</h2>
            {code_block(r#"pub fn install() {
    set_action_middleware(action_pipeline);
}

fn action_pipeline(req: FlowRequest) -> ... {
    let req = attach_session(req)?;   // guard
    audit_action(&req);               // interceptor
    Ok(req)
}"#)}
            <p>"For Flow multi-page apps, use " <code>"#[middleware]"</code> " instead — see " <a href="/docs/security/middleware">"Auth middleware"</a>"."</p>

            <h2>"5. CSRF is automatic"</h2>
            <p>"Runtime sends " <code>"X-Resuma-CSRF"</code> ". Forms include hidden " <code>"_csrf"</code> " via " <code>"form()"</code>"."</p>

            <h2>"6. Handle errors in the UI"</h2>
            {code_block(r#"try {
    const next = await __resuma.action("add_todo", [title]);
    state.todos.set(next);
} catch (e) {
    state.ui.update(s => { s.status = "Forbidden or rate limited"; });
}"#)}
        </>
    }
}
