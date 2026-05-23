use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Loaders"</h1>
            <p class="lead">"#[load] handlers fetch server data before a page renders — the Flow equivalent of route loaders."</p>

            <h2>"Define a loader"</h2>
            {code_block(r#"#[derive(Clone, Serialize, Deserialize)]
struct HomeData {
    title: String,
}

#[load]
async fn home(req: &FlowRequest) -> HomeData {
    HomeData { title: "Welcome".into() }
}"#)}

            <h2>"Consume in pages"</h2>
            {code_block(r#"pub fn page(_req: FlowRequest) -> View {
    let data = use_home_load();
    view! {
        <h1>{data.title.clone()}</h1>
    }
}"#)}

            <h2>"With try_use_load"</h2>
            {code_block(r#"pub fn page(_req: FlowRequest) -> View {
    let data = match try_use_load::<HomeData>("home") {
        Ok(d) => d,
        Err(e) => return error_page(&FlowError::Loader(e)),
    };
    view! { <h1>{data.title.clone()}</h1> }
}"#)}

            <h2>"LoadValue"</h2>
            <ul>
                <li><code>"LoadValue::Ok(T)"</code>" — data ready"</li>
                <li><code>"LoadValue::Err(LoaderError)"</code>" — loader failed"</li>
                <li><code>"LoadValue::Pending"</code>" — deferred streaming (see " <a href="/docs/flow/streaming">"Streaming"</a>")"</li>
            </ul>

            <h2>"FlowRequest access"</h2>
            {code_block(r#"#[load]
async fn profile(req: &FlowRequest) -> ProfileData {
    let id = req.param("id").unwrap_or("0");
    db::profile(id).await
}"#)}
        </>
    }
}
