use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"FAQ"</h1>
            <p class="lead">"Common questions about resumability, bundle size, and how Resuma compares to hydration-based frameworks."</p>

            <h2>"What is resumability vs hydration?"</h2>
            <p>"Hydration re-executes your entire component tree on the client to attach event listeners. Resumability serializes signals and handler references into HTML during SSR; the client resumes only what the user interacts with — no full-tree replay."</p>

            <h2>"Does Resuma run Rust in the browser?"</h2>
            <p>"No. Components always execute on the server. Client-side code is limited to a tiny runtime (~3 KB loader + lazy core) and small JS chunks translated from handler closures at compile time (rs2js in resuma-macros). Business logic stays in Rust."</p>

            <h2>"How big is the client bundle?"</h2>
            <p>"Static pages ship zero JS. Interactive pages load loader.js (~1–2 KB gzipped), then core.js on first interaction. Handler and island chunks load on demand. See the " <a href="/docs/benchmark">"benchmark page"</a> " for measured numbers."</p>

            <h2>"How does resumability compare to hydration?"</h2>
            <p>"Hydration re-runs components on the client to attach listeners. Resumability serializes signals and handler references during SSR; the client resumes only the interactions users trigger. Every " <code>"#[component]"</code> " is a resumable boundary; " <code>"#[island]"</code> " is optional for heavy lazy bundles."</p>

            <h2>"Do I need Node.js?"</h2>
            <p>"Only if you rebuild the JS runtime from source. Prebuilt assets ship inside the " <code>"resuma"</code> " crate (" <code>"assets/"</code> "). For app development, Rust + cargo (or " <code>"cargo install resuma"</code> ") is enough."</p>

            <h2>"Can I use Resuma without Flow?"</h2>
            <p>"Yes. ResumaApp supports single-page apps with manual route registration — ideal for counters, widgets, and embedded UI. Flow adds multi-page routing, loaders, submits, and middleware when you need a full site."</p>

            <h2>"How do forms work without JavaScript?"</h2>
            <p>"The " <code>"Form"</code> " component renders a real HTML form with " <code>"POST /_resuma/submit/:name"</code> ". Progressive enhancement: the runtime intercepts submit when loaded, but forms work as plain POST without JS."</p>

            <h2>"Is Resuma production-ready?"</h2>
            <p>"v0.x — APIs may evolve. Security defaults (CSRF, headers, rate limits) are built in. See " <a href="/docs/security">"Security"</a> " and harden with the " <a href="/docs/security/todo">"todo reference"</a>"."</p>

            <h2>"Where is the backend security reference?"</h2>
            <p><code>"examples/todo"</code> " — guards, DTO validation, service layer, authorization. Docs: " <a href="/docs/security/todo">"/docs/security/todo"</a>"."</p>
        </>
    }
}
