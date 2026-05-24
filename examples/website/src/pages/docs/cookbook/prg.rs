use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"PRG pattern"</h1>
            <p class="lead">"Post/Redirect/Get — avoid duplicate form submissions after " <code>"#[submit]"</code>"."</p>

            <h2>"Why"</h2>
            <p>"After a successful POST, redirect to a GET URL so refresh does not re-submit the form."</p>

            <h2>"Submit returns redirect hint"</h2>
            {code_block(r#"#[derive(Serialize)]
struct CreateResult { redirect: String }

#[submit]
async fn create_item(form: ItemForm, _req: &FlowRequest) -> Result<CreateResult, SubmitError> {
    db::insert(&form).await.map_err(|_| SubmitError::new("Failed"))?;
    Ok(CreateResult { redirect: "/items".into() })
}"#)}

            <h2>"Client enhancement"</h2>
            <p>
                "When JavaScript is enabled, the runtime can follow " <code>"redirect"</code> " in the submit JSON response. "
                "Without JS, use a server redirect response on " <code>"/_resuma/submit/:name"</code> " (303 See Other) — configure in custom submit handler if needed."
            </p>

            <h2>"Flash messages"</h2>
            <p>"Store a one-time toast in session or query " <code>"?created=1"</code> " on the redirect target page."</p>
        </>
    }
}
