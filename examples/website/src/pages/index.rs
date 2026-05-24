use resuma::prelude::*;

use crate::site::{code_block, feature_card, metric_item, pillar_card, pipeline_step};

pub fn page(_req: FlowRequest) -> View {
    view! {
        <main id="main-content" class="landing">
            <div class="hero-wrap">
                <section class="hero">
                    <div>
                        <span class="hero-badge">
                            <span class="hero-badge-dot"></span>
                            "v0.3.1 · Rust · Resumability · Zero hydration"
                        </span>
        <h1>
                            "Build "
                            <span class="accent">"instantly-interactive"</span>
                            " web apps in Rust"
                        </h1>
                        <p class="hero-tagline">"Ship HTML. Resume interactivity — never rehydrate."</p>
                        <p class="hero-lead">
                            "Components run once on the server. Resuma serialises signals and handler references into the page — the browser resumes only what users touch. ~884 B loader, lazy handler chunks, native Rust end-to-end."
                        </p>
                        <div class="hero-actions">
                            <a href="/docs/getting_started" class="btn btn-primary">"Get Started"</a>
                            <a href="/docs" class="btn btn-ghost">"Read the Docs"</a>
                            <a href="https://docs.rs/resuma/0.3.1" class="btn btn-ghost" target="_blank">"docs.rs"</a>
                        </div>
                        <p class="hero-note">
                            "Install: " <code>"cargo install resuma"</code> " · one crate for core + Flow + CLI"
                        </p>
                    </div>
                    <div class="hero-panel">
                        <div class="hero-panel-top">
                            <div class="hero-panel-dots">
                                <span></span><span></span><span></span>
                            </div>
                            <span class="hero-panel-label">"counter.rs"</span>
                        </div>
                        <div class="hero-panel-body">
                            {code_block(r#"#[component]
fn Counter() -> View {
    let count = use_signal(0);
    view! {
        <button onClick={
            move |_| count.update(|c| *c += 1)
        }>
            "Count: " {count}
        </button>
    }
}
// Handler lazy-loads from /_resuma/handler/Counter.js"#)}
                            <p class="hero-panel-caption">
                                <strong>"No hydration."</strong>
                                " rs2js translates the closure; the runtime resumes signal state on first click."
                            </p>
                        </div>
                    </div>
                </section>

                <div class="metrics-bar">
                    {metric_item("~884 B", "loader (gzip)")}
                    {metric_item("~3 KB", "core on first interaction")}
                    {metric_item("0", "JS on static pages")}
                    {metric_item("1", "cargo dependency")}
                </div>
            </div>

            <section class="section">
                <p class="section-eyebrow">"Performance model"</p>
                <h2 class="section-title">"Interactive from the first click"</h2>
                <p class="section-sub">"Resumability means the client never re-runs your component tree. State and handlers are already in the HTML — the tiny runtime wires them up lazily."</p>
                <div class="pillars">
                    {pillar_card("⚡", "Instant on interaction", "No hydration pass. Event delegation + resumed signals attach on first user input — not on page load.")}
                    {pillar_card("🧩", "Resumable by default", "Every #[component] is a lazy boundary. Handlers externalise to /_resuma/handler/{Component}.js — no #[island] required.")}
                    {pillar_card("📦", "Lazy handler chunks", "Viewport prefetch loads boundaries before the user clicks. Payload stays small — only tiny __page__ handlers stay inline.")}
                    {pillar_card("🦀", "Rust end-to-end", "Business logic, #[server] actions, and #[submit] forms stay in Rust. rs2js compiles handler closures to small JS.")}
                </div>
            </section>

            <section class="section section-alt">
                <p class="section-eyebrow">"Under the hood"</p>
                <h2 class="section-title">"How does it work?"</h2>
                <p class="section-sub">"One SSR pass. One resumability payload. Lazy execution on the client."</p>
                <div class="pipeline">
                    {pipeline_step("1", "SSR renders once", "Rust walks the View tree, emits HTML + data-r-on:* attributes, and serialises signals into <script type=\"resuma/state\">.")}
                    {pipeline_step("2", "Payload travels light", "Handler sources move to lazy chunks. computed! / effect! / debounce! replay on the client via rs2js.")}
                    {pipeline_step("3", "Browser resumes", "Loader (~884 B) bootstraps signals. Core loads on first interaction. Handlers fetch on demand — or prefetch in viewport.")}
                </div>
            </section>

            <section class="section">
                <div class="showcase">
                    <div class="showcase-copy">
                        <p class="section-eyebrow">"Components"</p>
                        <h3>"Write UI once — on the server"</h3>
                        <p>"Use view! with JSX-like syntax, fine-grained signals, and onClick handlers that compile to lazy JavaScript. No WASM bundle. No client-side component re-execution."</p>
                        <ul class="showcase-list">
                            <li>"use_signal for reactive state"</li>
                            <li>"computed! / effect! for client replay"</li>
                            <li>"#[component] props builder generated for you"</li>
                        </ul>
                        <a href="/docs/components/view" class="btn btn-ghost">"Component guide →"</a>
                    </div>
                    <div class="showcase-code">
                        <div class="code-window">
                            {code_block(r#"#[component]
fn SearchBar() -> View {
    let q = use_signal(String::new());
    let len = computed!([q], move || q.get().len());

    view! {
        <input
            value={q}
            onInput={move |e| q.set(e.value)}
            placeholder="Filter…"
        />
        <p>{format!("{} chars", len.get())}</p>
    }
}"#)}
                        </div>
                    </div>
                </div>
            </section>

            <section class="section section-alt">
                <div class="showcase showcase-reverse">
                    <div class="showcase-copy">
                        <p class="section-eyebrow">"Server actions"</p>
                        <h3>"Call Rust from the browser"</h3>
                        <p>"#[server] registers JSON-RPC at /_resuma/action/:name. Invoke from translated handlers or js!{} — CSRF-protected, typed, no manual API wiring."</p>
                        <ul class="showcase-list">
                            <li>"Async Rust functions as RPC endpoints"</li>
                            <li>"Forms via #[submit] and progressive enhancement"</li>
                            <li>"Security defaults: CSRF, headers, rate limits"</li>
                        </ul>
                        <a href="/docs/components/server" class="btn btn-ghost">"Server actions →"</a>
                    </div>
                    <div class="showcase-code">
                        <div class="code-window">
                            {code_block(r#"#[server]
async fn search(q: String) -> Vec<String> {
    db::search(&q).await
}

#[component]
fn LiveSearch() -> View {
    let query = use_signal(String::new());
    view! {
        <input onInput={ js! {
            state.query.set(event.target.value);
            const r = await __resuma.action(
                'search', [event.target.value]
            );
            state.results.set(r);
        }} />
    }
}"#)}
                        </div>
                    </div>
                </div>
            </section>

            <section class="section">
                <p class="section-eyebrow">"Why Resuma?"</p>
                <h2 class="section-title">"Everything you need for modern SSR"</h2>
                <p class="section-sub">"Resumable SSR in Rust — one install, progressive enhancement, full-stack Flow when you need it."</p>
                <div class="grid-3">
                    {feature_card("🌊", "Resuma Flow", "File-based pages, #[load], #[submit], layouts, middleware — built into the same crate.")}
                    {feature_card("📄", "Static export", "resuma build --static scaffolds HTML from src/pages/ for edge-friendly deploys.")}
                    {feature_card("🔧", "Dev experience", "resuma dev with HMR WebSocket, resuma new templates (basic, todo, flow).")}
                    {feature_card("🔗", "JS bridge", "view! translates Rust closures via rs2js. js!{} for escape hatches when you need raw client code.")}
                    {feature_card("🏝️", "Islands (optional)", "#[island(load = \"visible\")] for heavy widgets — most UI only needs #[component].")}
                    {feature_card("🛡️", "Security built in", "Crypto CSRF, security headers, rate limits — see examples/todo for production patterns.")}
                </div>
            </section>

            <section class="section section-alt">
                <p class="section-eyebrow">"One package"</p>
                <h2 class="section-title">"Resuma¹ + Flow²"</h2>
                <p class="section-sub">"Two layers, one dependency. Core stays stable; Flow adds routing, data loading, and forms."</p>
                <div class="package-diagram">
                    <article class="package-box">
                        <p class="tag">"RESUMA¹ — CORE"</p>
                        <h3>"Components & resumability"</h3>
                        <ul>
                            <li>"view!, #[component], use_signal"</li>
                            <li>"computed! / effect! / debounce!"</li>
                            <li>"#[server], ResumaApp, ~3KB runtime"</li>
                        </ul>
                    </article>
                    <div class="package-plus">"+"</div>
                    <article class="package-box">
                        <p class="tag">"FLOW² — FULL-STACK"</p>
                        <h3>"Pages, loads & submits"</h3>
                        <ul>
                            <li>"FlowApp, src/pages/, #[layout]"</li>
                            <li>"#[load], #[submit], #[middleware]"</li>
                            <li>"Streaming SSR, cache headers"</li>
                        </ul>
                    </article>
                </div>
            </section>

            <section class="section">
                <p class="section-eyebrow">"Integrations"</p>
                <h2 class="section-title">"Database, auth, and tooling"</h2>
                <p class="section-sub">"Qwik-style integration guides for SQLx, Turso, auth, validation, i18n, and E2E testing."</p>
                <div class="grid-3">
                    <a href="/docs/integrations/sqlx" class="card" style="text-decoration: none;">
                        <h3>"SQLx"</h3>
                        <p>"Type-safe SQL in #[load] and #[submit]."</p>
                    </a>
                    <a href="/docs/integrations/turso" class="card" style="text-decoration: none;">
                        <h3>"Turso"</h3>
                        <p>"Edge libSQL — file in dev, remote in prod."</p>
                    </a>
                    <a href="/docs/integrations/auth" class="card" style="text-decoration: none;">
                        <h3>"Auth"</h3>
                        <p>"Sessions and middleware for protected routes."</p>
                    </a>
                </div>
                <p style="text-align: center; margin-top: 1rem;">
                    <a href="/docs/integrations">"All integrations"</a>
                    " · "
                    <a href="/docs/search">"Search docs"</a>
                </p>
            </section>

            <section class="section">
                <p class="section-eyebrow">"Compare"</p>
                <h2 class="section-title">"Resumability vs hydration"</h2>
                <p class="section-sub">"Classic frameworks re-run components on the client to attach listeners. Resuma resumes serialized state instead."</p>
                <div class="compare-wrap">
                    <table class="compare">
                        <thead>
                            <tr>
                                <th></th>
                                <th>"Classic SSR + hydration"</th>
                                <th>"Resuma"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr><td>"After first paint"</td><td>"Re-run components on the client"</td><td class="yes">"Resume handlers only"</td></tr>
                            <tr><td>"Initial JS"</td><td>"App bundle scales with UI"</td><td class="yes">"~3KB runtime + lazy chunks"</td></tr>
                            <tr><td>"Static pages"</td><td>"Often still ship framework JS"</td><td class="yes">"Zero client JS"</td></tr>
                            <tr><td>"Interactive boundaries"</td><td>"Manual code splitting"</td><td class="yes">"Every #[component] resumable"</td></tr>
                            <tr><td>"Full-stack"</td><td>"Separate routing layer"</td><td class="yes">"Flow built in (one crate)"</td></tr>
                        </tbody>
                    </table>
                </div>
            </section>

            <section class="cta-section">
                <div class="cta-banner">
                    <h2>"Start building in 60 seconds"</h2>
                    <p>"Install the CLI, scaffold a project, and serve instantly-interactive Rust UI — no Node.js required for app development."</p>
                    <a href="/docs/getting_started" class="btn btn-primary">"Read the tutorial"</a>
                    <div class="cta-install">"cargo install resuma && resuma new my-app --template todo"</div>
                </div>
            </section>
        </main>
    }
}
