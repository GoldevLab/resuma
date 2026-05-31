use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    let users = crate::use_users_load();
    view! {
        <article class="card">
            <h1>"Users"</h1>
            <ul>
                {users.iter().map(|u| view! {
                    <li key={u.id.to_string()}>{format!("{} ({})", u.name, u.email)}</li>
                }).collect::<Vec<_>>()}
            </ul>
            <Form submit={crate::create_user}>
                <label>"Name" <input name="name" type="text" /></label>
                <label>"Email" <input name="email" type="email" /></label>
                <button type="submit">"Add user"</button>
            </Form>
        </article>
    }
}
