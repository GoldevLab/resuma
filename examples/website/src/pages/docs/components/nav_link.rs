use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"NavLink"</h1>
            <p class="lead">"NavLink renders an anchor that applies an active CSS class when the href matches the current path."</p>

            <h2>"Basic usage"</h2>
            {code_block(r#"view! {
    <nav>
        <NavLink href="/" activeClass="active">"Home"</NavLink>
        <NavLink href="/docs" activeClass="active">"Docs"</NavLink>
        <NavLink href="/about" activeClass="active">"About"</NavLink>
    </nav>
}"#)}

            <h2>"How matching works"</h2>
            <p>"Exact match wins. Prefix match applies when the current path starts with href followed by a slash — so " <code>"/docs"</code> " is active on " <code>"/docs/getting_started"</code>"."</p>

            <h2>"data-r-nav"</h2>
            <p>"NavLink sets " <code>"data-r-nav=\"true\""</code> " so the client runtime can enhance navigation without full page reloads when the runtime is loaded."</p>

            <h2>"In layouts"</h2>
            {code_block(r#"#[layout("/docs")]
fn DocsLayout() -> View {
    view! {
        <aside>
            <NavLink href="/docs/getting_started" activeClass="active">
                "Getting Started"
            </NavLink>
        </aside>
        <main><Slot /></main>
    }
}"#)}
        </>
    }
}
