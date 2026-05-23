use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Middleware"</h1>
            <p class="lead">"#[middleware] runs before pages, loaders, submits, and server actions in FlowApp."</p>

            <h2>"Define middleware"</h2>
            {code_block(r#"#[middleware]
async fn log_all(req: FlowRequest) -> Result<FlowRequest> {
    println!("[{}] {}", req.method, req.path);
    Ok(req)
}"#)}

            <h2>"Execution order"</h2>
            <p>"Registration order = execution order. Return " <code>"Ok(req)"</code> " to continue or " <code>"Err(ResumaError::...)"</code> " to abort with the matching HTTP status."</p>

            <h2>"Use cases"</h2>
            <ul>
                <li>"Logging and request tracing"</li>
                <li>"Session / auth injection via " <code>"set_extension"</code></li>
                <li>"Locale from Accept-Language"</li>
                <li>"Redirect guards for protected routes"</li>
            </ul>

            <h2>"Auth patterns"</h2>
            <p>"Session cookies, guards, and ResumaApp action middleware: " <a href="/docs/security/middleware">"Auth middleware guide"</a>"."</p>

            <h2>"Built-in security"</h2>
            <p>"CSRF, Helmet-style headers, rate limits, and Origin checks are enabled by default. See " <a href="/docs/security">"Security overview"</a>"."</p>
        </>
    }
}
