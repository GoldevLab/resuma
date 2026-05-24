use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Effects"</h1>
            <p class="lead">"Derived state and side effects. Use macros for client replay; plain functions for SSR-only work."</p>

            <h2>"computed! — client + SSR"</h2>
            {code_block(r#"let first = use_signal("Ada".into());
let last = use_signal("Lovelace".into());

let full_name = computed!([first, last], move || {
    format!("{} {}", first.get(), last.get())
});

view! { <p>{full_name}</p> }"#)}

            <h2>"effect! — client side effects"</h2>
            {code_block(r#"let query = use_signal(String::new());

effect!([query], move || {
    let q = query.get();
    // Runs on SSR and replays in the browser when query changes
    println!("query changed: {q}");
});"#)}

            <h2>"use_effect / use_computed — SSR only"</h2>
            <p>"Plain " <code>"use_effect()"</code> " and " <code>"use_computed()"</code> " run during server render. For browser replay, use " <code>"computed!"</code> " and " <code>"effect!"</code> " (rs2js-translated)."</p>

            <h2>"debounce!"</h2>
            {code_block(r#"let search = use_signal(String::new());
debounce!([search], 300, move || {
    // Fires 300ms after search stops changing (client + SSR)
});"#)}
        </>
    }
}
