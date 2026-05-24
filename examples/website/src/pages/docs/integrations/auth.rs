use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Auth"</h1>
            <p class="lead">
                "Session-based auth with Flow middleware — the Rust equivalent of "
                <a href="https://qwik.dev/docs/integrations/authjs/" target="_blank">"Qwik Auth.js"</a>"."
            </p>

            <h2>"Pattern"</h2>
            <ul>
                <li><code>"#[middleware]"</code> " — read session cookie, set " <code>"req.extensions"</code></li>
                <li><code>"req.is_authenticated()"</code> ", " <code>"req.user_id()"</code> ", " <code>"req.has_role()"</code></li>
                <li><code>"#[load]"</code> " / " <code>"#[submit]"</code> " — gate data and mutations"</li>
            </ul>

            <h2>"Session middleware"</h2>
            {code_block(r#"#[middleware]
async fn auth_session(mut req: FlowRequest) -> Result<FlowRequest> {
    if let Some(token) = req.header("cookie").and_then(|c| parse_session(c)) {
        if let Some(user) = sessions::verify(token).await {
            req.set_extension("authenticated", json!(true));
            req.set_extension("user_id", json!(user.id));
            req.set_extension("roles", json!(user.roles));
        }
    }
    Ok(req)
}

#[load]
async fn dashboard(req: &FlowRequest) -> DashboardData {
    if !req.is_authenticated() {
        return DashboardData::redirect("/login");
    }
    DashboardData::for_user(req.user_id().unwrap()).await
}"#)}

            <h2>"Login submit"</h2>
            {code_block(r#"#[derive(Deserialize)]
struct LoginForm { email: String, password: String }

#[submit]
async fn login(form: LoginForm, req: &FlowRequest) -> Result<LoginOk, SubmitError> {
    let user = db::verify_password(&form.email, &form.password).await
        .ok_or_else(|| SubmitError::new("Invalid credentials"))?;
    let token = sessions::issue(user.id);
    // Set-Cookie via response extension or redirect with cookie helper
    Ok(LoginOk { redirect: "/" })
}"#)}

            <h2>"Libraries"</h2>
            <table class="docs-table">
                <thead><tr><th>"Approach"</th><th>"Crates"</th></tr></thead>
                <tbody>
                    <tr><td>"Signed cookies"</td><td><code>"cookie"</code> ", " <code>"hmac"</code> ", " <code>"time"</code></td></tr>
                    <tr><td>"JWT API tokens"</td><td><code>"jsonwebtoken"</code></td></tr>
                    <tr><td>"OAuth (Google/GitHub)"</td><td><code>"oauth2"</code> ", " <code>"axum-extra"</code></td></tr>
                </tbody>
            </table>

            <p>"Combine with " <a href="/docs/security/middleware">"Security middleware"</a> " and " <a href="/docs/integrations/sqlx">"SQLx"</a> " for user tables."</p>
        </>
    }
}
