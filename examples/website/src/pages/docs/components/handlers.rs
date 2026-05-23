use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Event Handlers"</h1>
            <p class="lead">"Closures in onClick and other event attributes are translated to JavaScript at compile time (rs2js in resuma-macros) and lazy-loaded on first interaction."</p>

            <h2>"onClick"</h2>
            {code_block(r#"let count = use_signal(0);

view! {
    <button onClick={ move |_| count.update(|c| *c += 1) }>
        "+"
    </button>
}"#)}

            <h2>"How translation works"</h2>
            <ol>
                <li>"view! captures the closure at compile time."</li>
                <li>"rs2js (inside resuma-macros) emits a small JS module."</li>
                <li>"SSR embeds a HandlerRef in " <code>"data-r-on:click"</code> " attributes."</li>
                <li>"On first click, the runtime fetches " <code>"/_resuma/handler/:chunk"</code> " and runs the handler."</li>
            </ol>

            <h2>"Calling server actions"</h2>
            {code_block(r#"let results = use_signal(Vec::<String>::new());

view! {
    <button onClick={ move |_| {
        // rs2js translates signal updates + action calls
        results.set(vec!["from server".into()]);
    }}>
        "Search"
    </button>
}"#)}

            <h2>"Multiple events"</h2>
            {code_block(r#"view! {
    <input
        onInput={ move |ev| name.set(ev) }
        onKeyDown={ move |key| { /* ... */ } }
    />
}"#)}

            <h2>"See also"</h2>
            <p>"For complex client logic, use " <a href="/docs/components/js">"js!"</a> " or " <a href="/docs/components/server">"#[server]"</a> " actions."</p>
        </>
    }
}
