use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    View::fragment(vec![
        Child::View(view! {
            <article class="card">
                <h1>"About"</h1>
                <p>"File-based route: " <code>"src/pages/about.rs"</code> " → " <code>"/about"</code></p>
            </article>
        }),
        Child::View(portal(
            "modals",
            vec![Child::View(view! {
                <aside class="card" style="margin-top:1rem">
                    <p>"This aside is portaled to " <code>"#modals"</code></p>
                </aside>
            })],
        )),
    ])
}
