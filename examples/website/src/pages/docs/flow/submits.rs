use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Actions (Submits)"</h1>
            <p class="lead">"#[submit] handlers process form POSTs with typed validation and field-level errors."</p>

            <h2>"Define a submit handler"</h2>
            {code_block(r#"#[derive(Deserialize)]
struct ContactForm {
    email: String,
    message: String,
}

#[submit]
async fn contact(form: ContactForm, _req: &FlowRequest)
    -> Result<ContactResult, SubmitError>
{
    if form.email.is_empty() {
        return Err(SubmitError::new("Fix errors")
            .field("email", "Required"));
    }
    Ok(ContactResult { ok: true })
}"#)}

            <h2>"Wire to Form"</h2>
            {code_block(r#"view! {
    <Form submit={contact}>
        <input name="email" type="email" />
        <textarea name="message"></textarea>
        <button type="submit">"Send"</button>
    </Form>
}"#)}

            <h2>"SubmitError"</h2>
            <p>"Return structured validation errors. The runtime maps field_errors to form fields when using client enhancement."</p>
            {code_block(r#"SubmitError::new("Invalid input")
    .field("email", "Must be a valid email")
    .field("message", "Too short")"#)}

            <h2>"HTTP endpoint"</h2>
            <p>"Forms POST to " <code>"/_resuma/submit/:name"</code> ". Works without JavaScript via standard form submission."</p>
        </>
    }
}
