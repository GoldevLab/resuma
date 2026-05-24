use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[component]
fn Counter() -> impl IntoView {
    let (count, set_count) = signal(0);
    view! {
        <main>
            <h1>"Leptos Counter"</h1>
            <p>"Current count: " {count}</p>
            <button on:click=move |_| set_count.update(|n| *n += 1)>"+"</button>
        </main>
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(Counter);
}
