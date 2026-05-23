use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"view!"</h1>
            <p class="lead">"The view! macro builds a View tree from JSX-like syntax — elements, text, and Rust expressions."</p>

            <h2>"Basic syntax"</h2>
            {code_block(r#"view! {
    <main class="page">
        <h1>"Hello Resuma"</h1>
        <p>"Static text in quotes."</p>
    </main>
}"#)}

            <h2>"Dynamic bindings"</h2>
            <p>"Wrap Rust expressions in curly braces. Signals, strings, and numbers interpolate directly."</p>
            {code_block(r#"let title = "Docs".to_string();
let count = use_signal(0);

view! {
    <h1>{title}</h1>
    <p>"Count: " {count}</p>
}"#)}

            <h2>"Attributes"</h2>
            <p>"Static attributes use quoted strings. Dynamic values use braces."</p>
            {code_block(r#"view! {
    <a href="/docs" class={"card ".to_string() + &extra}>
        "Link"
    </a>
    <input type="text" value={name} disabled={is_loading} />
}"#)}

            <h2>"Fragments"</h2>
            <p>"Use " <code>"<>"</code> " to group siblings without a wrapper element."</p>
            {code_block(r#"view! {
    <>
        <h1>{title}</h1>
        <Slot />
    </>
}"#)}

            <h2>"Components"</h2>
            <p>"Capitalized tags invoke " <code>"#[component]"</code> " functions. Props are Rust struct fields."</p>
            {code_block(r#"view! {
    <Greeting name={"World".into()} />
    <Card>
        <h2 slot="header">"Title"</h2>
        <p>"Body content"</p>
    </Card>
}"#)}

            <h2>"Boolean attributes"</h2>
            {code_block(r#"view! {
    <button disabled={true}> "Save" </button>
    <input required={false} />
}"#)}
        </>
    }
}
