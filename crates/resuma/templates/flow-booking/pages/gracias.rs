use resuma::prelude::*;

#[component]
pub fn GraciasPage() -> View {
    view! {
        <main>
            <h1>"Thanks!"</h1>
            <p>"Your appointment is reserved."</p>
            <NavLink href="/book" activeClass="active">"Book another"</NavLink>
        </main>
    }
}
