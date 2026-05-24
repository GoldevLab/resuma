use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Validation"</h1>
            <p class="lead">"Typed form validation in " <code>"#[submit]"</code> " with the " <code>"validator"</code> " crate — Rust equivalent of Zod in Qwik actions."</p>

            <h2>"Install"</h2>
            {code_block(r#"validator = { version = "0.19", features = ["derive"] }"#)}

            <h2>"Submit with Validate trait"</h2>
            {code_block(r#"use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate)]
struct SignupForm {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8))]
    password: String,
}

#[submit]
async fn signup(form: SignupForm, _req: &FlowRequest) -> Result<(), SubmitError> {
    if let Err(errors) = form.validate() {
        let mut err = SubmitError::new("Fix the errors below.");
        for (field, msgs) in errors.field_errors() {
            if let Some(m) = msgs.first().and_then(|m| m.message.as_ref()) {
                err = err.field(field, m);
            }
        }
        return Err(err);
    }
    db::create_user(&form.email, &form.password).await?;
    Ok(())
}"#)}

            <p>"Field errors map to " <code>"Form"</code> " client enhancement automatically. See " <a href="/docs/flow/submits">"Actions"</a>"."</p>
        </>
    }
}
