use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Caching"</h1>
            <p class="lead">"Set Cache-Control headers on #[load] responses to cache server data at the edge or in the browser."</p>

            <h2>"cache attribute"</h2>
            {code_block(r#"#[load(cache = "public, max-age=60")]
async fn home(_req: &FlowRequest) -> HomeData {
    HomeData { title: "Welcome".into() }
}

#[load(cache = "public, max-age=120")]
async fn docs_index(_req: &FlowRequest) -> DocsData {
    fetch_docs().await
}"#)}

            <h2>"How it works"</h2>
            <p>"The cache string is applied as a Cache-Control response header on the page response when the loader completes. Use short max-age for frequently changing data, longer for static-ish content."</p>

            <h2>"Private caching"</h2>
            {code_block(r#"#[load(cache = "private, max-age=300")]
async fn dashboard(req: &FlowRequest) -> DashData {
    user_dashboard(req).await
}"#)}

            <h2>"When not to cache"</h2>
            <p>"Omit the cache attribute for personalized or auth-gated loaders. Default behavior sends no Cache-Control override."</p>
        </>
    }
}
