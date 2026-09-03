use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <main>
            <h1>"%NAME%"</h1>
            <p>"Sample booking app with query-driven loaders."</p>
            <p><NavLink href="/book" activeClass="active">"Book an appointment"</NavLink></p>
        </main>
    }
}
