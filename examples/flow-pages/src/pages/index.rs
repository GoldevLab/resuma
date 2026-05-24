use resuma::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HomeData {
    message: String,
}

#[load(cache = "public, max-age=120")]
async fn home(_req: &FlowRequest) -> HomeData {
    HomeData {
        message: "Auto-wired from src/pages/index.rs".into(),
    }
}

pub fn page(_req: FlowRequest) -> View {
    let data = match try_use_load::<HomeData>("home") {
        Ok(d) => d,
        Err(e) => return error_page(&FlowError::Loader(e)),
    };

    with_view_transition(
        "home",
        vec![Child::View(view! {
            <article class="card">
                <h1>{data.message.clone()}</h1>
                <p>"Pages discovered from " <code>"src/pages/"</code> " and wired via " <code>"PagesRegistry"</code></p>
                <p>"Data loaded with " <code>"#[load]"</code> " and cached for 120s."</p>
            </article>
        })],
    )
}
