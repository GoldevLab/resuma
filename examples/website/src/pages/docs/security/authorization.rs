use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Authorization & RLS"</h1>
            <p class="lead">
                "Row-level access in Rust — no magic ACL columns. Working demo: "
                <a href="/docs/security/todo">"todo example"</a> " (switch guest / alice / bob)."
            </p>

            <h2>"Check ownership in handlers"</h2>
            {code_block(r#"pub fn assert_owner(owner_id: &str, req: &FlowRequest) -> Result<()> {
    let uid = req.user_id().ok_or(ResumaError::Unauthorized)?;
    if owner_id != uid && !req.has_role("admin") {
        return Err(ResumaError::Forbidden("not your task".into()));
    }
    Ok(())
}"#)}

            <h2>"Default-deny checklist"</h2>
            <ul>
                <li>"Read " <code>"user_id()"</code> " from middleware — never trust client-sent ids"</li>
                <li>"Every mutation checks ownership or admin role"</li>
                <li>"Return " <code>"Forbidden"</code> ", not silent success"</li>
                <li>"Admin secrets only in env, never in SSR payload"</li>
            </ul>

            <h2>"Postgres row-level security (optional)"</h2>
            <p>"Last line of defense when using a database:"</p>
            {code_block(r#"ALTER TABLE todos ENABLE ROW LEVEL SECURITY;
CREATE POLICY todo_owner ON todos
  USING (owner_id = current_setting('app.user_id', true));"#)}
            <p>"Set " <code>"app.user_id"</code> " on the DB connection from your auth middleware before queries."</p>
        </>
    }
}
