use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Streaming"</h1>
            <p class="lead">"Deferred #[load(stream)] handlers let SSR flush the shell immediately while slow data streams in."</p>

            <h2>"Stream loader"</h2>
            {code_block(r#"#[load(stream)]
async fn home(req: &FlowRequest) -> HomeData {
    // Slow DB query — does not block initial HTML
    db::home(req).await
}"#)}

            <h2>"Page pattern"</h2>
            {code_block(r#"pub fn page(_req: FlowRequest) -> View {
    match use_home_load() {
        LoadValue::Pending => view! {
            <article>
                <h1>"Home"</h1>
                {stream_slot("home")}
            </article>
        },
        LoadValue::Ok(data) => home_view(&data),
        LoadValue::Err(e) => error_page(&FlowError::Loader(e)),
    }
}

fn home_view(data: &HomeData) -> View {
    view! {
        <article>
            <h1>{data.title.clone()}</h1>
            <p>{data.body.clone()}</p>
        </article>
    }
}"#)}

            <h2>"stream_slot"</h2>
            <p>"stream_slot(name) marks where deferred HTML is inserted when the loader completes. Enable streaming on FlowApp:"</p>
            {code_block(r#"FlowApp::new()
    .streaming(true)
    .auto_pages("src/pages", PagesRegistry)
    .serve(FlowServeOptions::default())
    .await"#)}

            <h2>"Cookbook"</h2>
            <p>"See " <a href="/docs/cookbook/streaming_loaders">"Streaming loaders cookbook"</a> " for a complete deferred pattern."</p>
        </>
    }
}
