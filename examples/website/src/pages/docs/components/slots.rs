use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Slots"</h1>
            <p class="lead">"Slots project child content into component templates — default and named slots via the slot attribute."</p>

            <h2>"Default slot"</h2>
            {code_block(r#"#[component]
fn Panel() -> View {
    view! {
        <section class="panel">
            <Slot />
        </section>
    }
}

// Usage:
view! {
    <Panel>
        <p>"Child content lands in the default slot."</p>
    </Panel>
}"#)}

            <h2>"Named slots"</h2>
            {code_block(r#"#[component]
fn Card() -> View {
    view! {
        <article class="card">
            <header><Slot name="header" /></header>
            <div class="body"><Slot /></div>
            <footer><Slot name="footer" /></footer>
        </article>
    }
}

view! {
    <Card>
        <h2 slot="header">"Title"</h2>
        <p>"Body paragraph."</p>
        <a slot="footer" href="/more">"Read more"</a>
    </Card>
}"#)}

            <h2>"Layouts"</h2>
            <p>"Flow layouts use " <code>"<Slot />"</code> " to wrap page content. See " <a href="/docs/flow/layouts">"Layouts"</a> " for nested layout chains."</p>
        </>
    }
}
