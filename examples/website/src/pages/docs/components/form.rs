use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Form"</h1>
            <p class="lead">"The Form component wires HTML forms to #[submit] handlers with progressive enhancement."</p>

            <h2>"Basic form"</h2>
            {code_block(r#"#[submit]
async fn contact(form: ContactForm, _req: &FlowRequest)
    -> Result<ContactResult, SubmitError>
{
    if form.email.is_empty() {
        return Err(SubmitError::new("Fix errors")
            .field("email", "Required"));
    }
    Ok(ContactResult { ok: true })
}

view! {
    <Form submit={contact}>
        <input name="email" type="email" />
        <input name="name" type="text" />
        <button type="submit">"Send"</button>
    </Form>
}"#)}

            <h2>"Progressive enhancement"</h2>
            <p>"Form renders " <code>"method=\"POST\""</code> " and " <code>"action=\"/_resuma/submit/:name\""</code> ". Without JavaScript, the browser submits normally. With the runtime loaded, submit is intercepted for SPA-style updates."</p>

            <h2>"Client-side feedback"</h2>
            <p>"When the runtime intercepts submit, responses include " <code>"ok"</code> ", " <code>"value"</code> ", " <code>"error"</code> ", and " <code>"field_errors"</code> ". Use js! for optimistic UI updates after submit."</p>

            <h2>"See also"</h2>
            <p>"Submit handler details on " <a href="/docs/flow/submits">"Actions (submits)"</a>"."</p>
        </>
    }
}
