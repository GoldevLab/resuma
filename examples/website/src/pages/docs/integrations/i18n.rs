use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"i18n"</h1>
            <p class="lead">"Internationalization in Resuma Flow — load locale strings server-side in " <code>"#[load]"</code>"."</p>

            <h2>"Recommended crates"</h2>
            <ul>
                <li><code>"fluent"</code> " / " <code>"fluent-bundle"</code> " — Mozilla Fluent (.ftl files)"</li>
                <li><code>"rust-i18n"</code> " — compile-time JSON/YAML catalogs"</li>
            </ul>

            <h2>"Locale loader"</h2>
            {code_block(r#"#[load]
async fn i18n(req: &FlowRequest) -> Messages {
    let lang = req.query_param("lang")
        .or_else(|| req.header("accept-language").map(|s| s.split(',').next().unwrap_or("en")))
        .unwrap_or("en");
    Messages::load(lang).await
}

pub fn page(_req: FlowRequest) -> View {
    let t = use_i18n_load();
    view! {
        <h1>{t.get("home.title")}</h1>
        <p>{t.get("home.lead")}</p>
    }
}"#)}

            <h2>"URL strategy"</h2>
            <p><code>"/en/docs"</code> ", " <code>"/es/docs"</code> " via Flow file routes or " <code>"?lang=es"</code> " query param with " <code>"#[load]"</code> " cache keys per locale."</p>
        </>
    }
}
