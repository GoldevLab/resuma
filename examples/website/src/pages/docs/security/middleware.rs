use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Auth middleware"</h1>
            <p class="lead">
                "Two paths depending on your app type. Mechanics of "
                <code>"#[middleware]"</code> " are in "
                <a href="/docs/flow/middleware">"Flow middleware"</a>"."
            </p>

            <h2>"ResumaApp (single-page / todo template)"</h2>
            <p>"Use " <code>"set_action_middleware"</code> " for " <code>"#[server]"</code> " actions only. See " <a href="/docs/security/todo">"todo example"</a>"."</p>
            {code_block(r#"set_action_middleware(|req| {
    Box::pin(async move {
        let req = attach_session(req)?;
        Ok(req)
    })
});

// attach_session sets:
req.set_extension("user_id", json!(user));
req.set_extension("roles", json!(roles));"#)}

            <h2>"FlowApp (multi-page site)"</h2>
            <p>"Use " <code>"#[middleware]"</code> " — runs before pages, loaders, submits, and actions."</p>
            {code_block(r#"#[middleware]
async fn require_auth(mut req: FlowRequest) -> Result<FlowRequest> {
    if req.header("authorization").is_none() {
        return Err(ResumaError::Unauthorized);
    }
    req.set_extension("authenticated", json!(true));
    Ok(req)
}"#)}

            <h2>"What happens on Err?"</h2>
            <ul>
                <li><strong>"Pages"</strong>" — error view (401/403/429)"</li>
                <li><strong>"Submits"</strong>" — JSON/HTML error response"</li>
                <li><strong>"Actions"</strong>" — " <code>"{ ok: false, error: \"...\" }"</code></li>
            </ul>

            <h2>"Helpers on FlowRequest"</h2>
            {code_block(r#"req.is_authenticated()
req.user_id()      // Option<&str>
req.has_role("admin")"#)}
        </>
    }
}
