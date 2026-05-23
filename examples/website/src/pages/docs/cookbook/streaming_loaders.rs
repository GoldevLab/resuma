use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Streaming Loaders"</h1>
            <p class="lead">"Ship the HTML shell immediately and stream slow loader results into stream_slot placeholders."</p>

            <h2>"Enable streaming"</h2>
            {code_block(r#"FlowApp::new()
    .streaming(true)
    .auto_pages("src/pages", PagesRegistry)
    .serve(FlowServeOptions::default())
    .await"#)}

            <h2>"Deferred loader"</h2>
            {code_block(r#"#[derive(Clone, Serialize, Deserialize)]
struct ProductData {
    name: String,
    price: u32,
}

#[load(stream)]
async fn product(req: &FlowRequest) -> ProductData {
    let id = req.param("id").unwrap_or("0");
    // Simulated slow query
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
    db::product(id).await
}"#)}

            <h2>"Page with placeholder"</h2>
            {code_block(r#"pub fn page(_req: FlowRequest) -> View {
    match use_product_load() {
        LoadValue::Pending => view! {
            <article>
                <h1>"Product"</h1>
                <div class="skeleton">{stream_slot("product")}</div>
            </article>
        },
        LoadValue::Ok(data) => view! {
            <article>
                <h1>{data.name.clone()}</h1>
                <p>"$" {data.price.to_string()}</p>
            </article>
        },
        LoadValue::Err(e) => error_page(&FlowError::Loader(e)),
    }
}"#)}

            <h2>"User experience"</h2>
            <p>"Users see layout and headings instantly. Product details replace the skeleton when the loader completes — no blank screen while waiting on the database."</p>
        </>
    }
}
