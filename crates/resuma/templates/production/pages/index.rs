use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <main class="card">
            <h1>"%NAME%"</h1>
            <p>"Production-ready Resuma Flow app — CSRF, CSP, and health checks enabled."</p>
            <p>"Set " <code>"RESUMA_ENV=production"</code> " and deploy with Docker or Fly."</p>
            <p><NavLink href="/ops" activeClass="active">"Open ops dashboard →"</NavLink></p>
        </main>
    }
}
