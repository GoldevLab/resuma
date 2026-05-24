use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Documentation"</h1>
            <p class="lead">
                "Resumable SSR in Rust — core components, full-stack Flow, production security, and deploy guides. "
                "Published as "
                <a href="https://crates.io/crates/resuma" target="_blank">"resuma 0.3.1"</a>
                " on crates.io · "
                <a href="https://docs.rs/resuma" target="_blank">"API on docs.rs"</a>"."
            </p>

            <h2>"Start here"</h2>
            <div class="grid-3">
                <a href="/docs/getting_started" class="card" style="text-decoration: none;">
                    <h3>"Getting Started"</h3>
                    <p>"CLI, templates (basic / todo), first app in minutes."</p>
                </a>
                <a href="/docs/security/todo" class="card" style="text-decoration: none;">
                    <h3>"Todo example"</h3>
                    <p>"Full showcase: signals, islands, server actions, backend security."</p>
                </a>
                <a href="/docs/flow" class="card" style="text-decoration: none;">
                    <h3>"Resuma Flow"</h3>
                    <p>"Multi-page apps: loads, submits, layouts, file routing."</p>
                </a>
            </div>

            <h2>"Learn by topic"</h2>
            <div class="grid-3">
                <a href="/docs/components" class="card" style="text-decoration: none;">
                    <h3>"Components"</h3>
                    <p>"view!, signals, islands, handlers, server actions."</p>
                </a>
                <a href="/docs/security" class="card" style="text-decoration: none;">
                    <h3>"Security"</h3>
                    <p>"CSRF, rate limits, auth, validation — defaults + hardening."</p>
                </a>
                <a href="/docs/cookbook" class="card" style="text-decoration: none;">
                    <h3>"Cookbook"</h3>
                    <p>"Theme, portals, streaming loaders, Docker deploy."</p>
                </a>
                <a href="/docs/integrations" class="card" style="text-decoration: none;">
                    <h3>"Integrations"</h3>
                    <p>"SQLx and Turso — database layer for loaders and submits."</p>
                </a>
                <a href="/docs/architecture" class="card" style="text-decoration: none;">
                    <h3>"Architecture"</h3>
                    <p>"Resumability vs hydration — how the SSR payload works."</p>
                </a>
                <a href="/docs/project_structure" class="card" style="text-decoration: none;">
                    <h3>"Project structure"</h3>
                    <p>"ResumaApp vs FlowApp layouts."</p>
                </a>
                <a href="/docs/examples" class="card" style="text-decoration: none;">
                    <h3>"Examples"</h3>
                    <p>"Runnable crates: todo, counter, flow-demo, website."</p>
                </a>
                <a href="/docs/cli" class="card" style="text-decoration: none;">
                    <h3>"CLI"</h3>
                    <p>"new, dev, build, routes --generate."</p>
                </a>
            </div>

            <h2>"What is Resuma?"</h2>
            <p>"Components run on the server once. SSR embeds signals and handler refs in HTML; a ~3 KB client runtime resumes interactivity on demand."</p>
            <p><strong>"Resuma Flow"</strong>" adds file-based pages, " <code>"#[load]"</code> ", " <code>"#[submit]"</code> ", and middleware — one " <code>"resuma"</code> " crate, Rust-native."</p>
        </>
    }
}
