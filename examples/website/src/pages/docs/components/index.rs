use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Components"</h1>
            <p class="lead">"Author UI with the view! macro, signals, and slots — components only execute on the server."</p>

            <h2>"Topics"</h2>
            <div class="grid-3">
                <a href="/docs/components/view" class="card" style="text-decoration: none;">
                    <h3>"view!"</h3>
                    <p>"JSX-like templates, attributes, dynamic bindings."</p>
                </a>
                <a href="/docs/components/signals" class="card" style="text-decoration: none;">
                    <h3>"Signals"</h3>
                    <p>"use_signal, ReadSignal, WriteSignal — fine-grained reactivity."</p>
                </a>
                <a href="/docs/components/effects" class="card" style="text-decoration: none;">
                    <h3>"Effects"</h3>
                    <p>"use_effect and use_computed for derived state."</p>
                </a>
                <a href="/docs/components/handlers" class="card" style="text-decoration: none;">
                    <h3>"Handlers"</h3>
                    <p>"onClick closures translated to lazy JS chunks."</p>
                </a>
                <a href="/docs/components/islands" class="card" style="text-decoration: none;">
                    <h3>"Islands"</h3>
                    <p>"computed! / effect! for client replay; #[island] optional for heavy widgets."</p>
                </a>
                <a href="/docs/components/server" class="card" style="text-decoration: none;">
                    <h3>"Server actions"</h3>
                    <p>"#[server] RPC — call Rust from the client."</p>
                </a>
                <a href="/docs/components/js" class="card" style="text-decoration: none;">
                    <h3>"js!"</h3>
                    <p>"Escape hatch for raw client-side JavaScript."</p>
                </a>
                <a href="/docs/components/slots" class="card" style="text-decoration: none;">
                    <h3>"Slots"</h3>
                    <p>"Content projection with named slots."</p>
                </a>
                <a href="/docs/components/nav_link" class="card" style="text-decoration: none;">
                    <h3>"NavLink"</h3>
                    <p>"Active-state navigation links."</p>
                </a>
                <a href="/docs/components/form" class="card" style="text-decoration: none;">
                    <h3>"Form"</h3>
                    <p>"Progressive-enhancement form submits."</p>
                </a>
                <a href="/docs/components/store" class="card" style="text-decoration: none;">
                    <h3>"Store"</h3>
                    <p>"Structured reactive state with use_store."</p>
                </a>
                <a href="/docs/components/context" class="card" style="text-decoration: none;">
                    <h3>"Context"</h3>
                    <p>"provide_context and use_context for descendants."</p>
                </a>
                <a href="/docs/components/tasks" class="card" style="text-decoration: none;">
                    <h3>"Tasks"</h3>
                    <p>"use_task, use_visible_task, use_debounce."</p>
                </a>
            </div>

            <h2>"Quick example"</h2>
            <p>"A minimal component with resumable state:"</p>
            {code_block(r#"#[component]
fn Counter() -> View {
    let count = use_signal(0);
    view! {
        <button onClick={ move |_| count.update(|c| *c += 1) }>
            {count}
        </button>
    }
}"#)}
        </>
    }
}
