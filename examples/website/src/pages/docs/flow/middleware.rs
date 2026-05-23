use resuma::prelude::*;
use crate::site::code_block;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Middleware"</h1>
            <p class="lead">"#[middleware] functions transform FlowRequest before pages, loaders, and submits run."</p>

            <h2>"Define middleware"</h2>
            {code_block(r#"#[middleware]
async fn log_all(req: FlowRequest) -> resuma::Result<FlowRequest> {
    println!("[{}] {}", req.method, req.path);
    Ok(req)
}

#[middleware]
async fn require_auth(req: FlowRequest) -> resuma::Result<FlowRequest> {
    if req.header("authorization").is_none() {
        return Err(resuma::ResumaError::Other("Unauthorized".into()));
    }
    Ok(req)
}"#)}

            <h2>"Execution order"</h2>
            <p>"Middleware runs in registration order for incoming requests. Each handler receives the possibly-modified FlowRequest and returns Ok(req) to continue or Err to abort."</p>

            <h2>"Use cases"</h2>
            <ul>
                <li>"Request logging and tracing"</li>
                <li>"Authentication and session injection"</li>
                <li>"Locale detection from Accept-Language"</li>
                <li>"Redirect guards for protected routes"</li>
            </ul>

            <h2>"Action middleware"</h2>
            <p>"Server actions at " <code>"/_resuma/action/*"</code> " can also pass through a global middleware pipeline registered via set_action_middleware."</p>
        </>
    }
}
