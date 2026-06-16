//! Store derive and For macro smoke tests.

use resuma::prelude::*;

#[derive(Clone, serde::Serialize, serde::Deserialize, Store)]
struct CounterState {
    count: i32,
    label: String,
}

#[test]
fn store_derive_setters_update_value() {
    let store = use_store(CounterState {
        count: 0,
        label: "n".into(),
    });
    store.set_count(5);
    store.set_label("hi".into());
    assert_eq!(store.count(), 5);
    assert_eq!(store.label(), "hi");
}

#[test]
fn for_macro_expands_like_map() {
    let items = vec!["a", "b"];
    let via_for = view! {
        <ul>
            <For each={items.clone()} let:item>
                <li>{item}</li>
            </For>
        </ul>
    };
    let via_map = view! {
        <ul>
            {items.into_iter().map(|item| view! { <li>{item}</li> }).collect::<Vec<_>>()}
        </ul>
    };
    let html_for = resuma::ssr::render_view(&via_for);
    let html_map = resuma::ssr::render_view(&via_map);
    assert_eq!(html_for, html_map);
}
