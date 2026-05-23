use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Debouncer"</h1>
            <p class="lead">"Debounce a search input so API calls fire only after the user stops typing."</p>

            <h2>"Search component"</h2>
            {code_block(r#"#[component]
fn SearchBox() -> View {
    let query = use_signal(String::new());
    let results = use_signal(Vec::<String>::new());

    use_debounce(&query, 300, move |q| {
        if q.len() >= 2 {
            // In production: call #[server] search action
            results.set(vec![format!("Result for {q}")]);
        }
    });

    view! {
        <div class="search">
            <input
                type="search"
                placeholder="Search…"
                onInput={ js! {
                    state.query.set(event.target.value);
                }}
            />
            <ul>
                {results.get().iter().map(|r| view! {
                    <li>{r.clone()}</li>
                }).collect::<Vec<_>>()}
            </ul>
        </div>
    }
}"#)}

            <h2>"How it works"</h2>
            <p>"use_debounce watches the signal and delays the callback. On SSR the callback runs once; on the client the runtime applies the delay before re-firing."</p>
        </>
    }
}
