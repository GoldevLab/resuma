use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Server Actions"</h1>
            <p class="lead">"#[server] registers Rust functions as RPC endpoints callable from handlers and client code."</p>

            <h2>"Define an action"</h2>
            {code_block(r#"#[server]
async fn search(q: String) -> Vec<String> {
    db::search(&q).await
}

#[server]
async fn greet(name: String) -> String {
    format!("Hello, {name}!")
}"#)}

            <h2>"With FlowRequest"</h2>
            <p>"Server actions can access request context when Flow is enabled:"</p>
            {code_block(r#"#[server]
async fn list_items(req: &FlowRequest) -> Vec<Item> {
    let cookie = req.header("cookie").unwrap_or("");
    db::items_for_session(cookie).await
}"#)}

            <h2>"HTTP endpoint"</h2>
            <p>"Each action is exposed at " <code>"POST /_resuma/action/:name"</code> " with body " <code>"{ \"args\": [...] }"</code>". CSRF token required on mutations."</p>

            <h2>"Return Result for errors"</h2>
            {code_block(r#"#[server]
async fn create(name: String) -> Result<Item> {
    validate(&name)?;
    Ok(db::create(name).await?)
}"#)}

            <h2>"From handlers"</h2>
            {code_block(r#"view! {
    <button onClick={ js! {
        const rows = await __resuma.action("search", [state.q.value]);
        state.results.set(rows);
    }}>"Search"</button>
}"#)}

            <h2>"Production patterns"</h2>
            <p>"Validation, auth middleware, and fail-closed errors: " <a href="/docs/security/server_actions">"Secure server actions"</a> " · " <a href="/docs/security/todo">"Todo example"</a>"."</p>
        </>
    }
}
