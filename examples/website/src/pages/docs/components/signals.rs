use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Signals"</h1>
            <p class="lead">"Signals are fine-grained reactive cells serialized in the resumability payload and resumed on the client."</p>

            <h2>"use_signal"</h2>
            {code_block(r#"let count = use_signal(0);

count.set(5);
count.update(|c| *c += 1);
let current = count.get();"#)}

            <h2>"In templates"</h2>
            <p>"Interpolating " <code>"{count}"</code> " in view! renders the current value and registers a subscription."</p>
            {code_block(r#"view! {
    <p>"Count: " {count}</p>
    <button onClick={ move |_| count.update(|c| *c += 1) }>
        "+"
    </button>
}"#)}

            <h2>"ReadSignal and WriteSignal"</h2>
            <p>"Split a signal when you want read-only or write-only access — useful for passing props to child components."</p>
            {code_block(r#"let count = use_signal(0);
let (read, write) = count.split();

// read.get() — read-only
// write.set(n) / write.update(...) — write-only"#)}

            <h2>"Serialization"</h2>
            <p>"Signal values must implement Serialize. They travel in the " <code>"resuma/state"</code> " script tag and sync on the client after resume."</p>
        </>
    }
}
