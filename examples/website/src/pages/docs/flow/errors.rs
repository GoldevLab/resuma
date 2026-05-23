use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Error Handling"</h1>
            <p class="lead">"FlowError unifies not-found, loader failures, and render errors into consistent error pages."</p>

            <h2>"FlowError variants"</h2>
            <ul>
                <li><code>"FlowError::NotFound"</code>" — 404, no matching route"</li>
                <li><code>"FlowError::Loader(LoaderError)"</code>" — #[load] failed with status + message"</li>
                <li><code>"FlowError::Render(String)"</code>" — page render panic or explicit error"</li>
            </ul>

            <h2>"not_found_page"</h2>
            {code_block(r#"FlowApp::new()
    .not_found(|| not_found_page())
    .serve(FlowServeOptions::default())
    .await"#)}

            <h2>"error_page"</h2>
            {code_block(r#"match use_home_load() {
    LoadValue::Ok(data) => render_home(&data),
    LoadValue::Err(e) => error_page(&FlowError::Loader(e)),
    LoadValue::Pending => view! { {stream_slot("home")} },
}"#)}

            <h2>"LoaderError"</h2>
            {code_block(r#"LoaderError::new(404, "User not found")
    LoaderError::new(500, "Database unavailable")"#)}

            <h2>"Status codes"</h2>
            <p>"FlowError::status() returns the HTTP status — 404 for NotFound, the loader status for Loader errors, 500 for Render."</p>
        </>
    }
}
