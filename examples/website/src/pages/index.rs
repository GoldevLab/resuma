use resuma::prelude::*;

use crate::site::{code_block, feature_card};

pub fn page(_req: FlowRequest) -> View {
    view! {
        <main id="main-content">
            <section class="hero">
                <div>
                    <span class="hero-badge">"Rust · SSR · Resumability · Zero hydration"</span>
                    <h1>
                        "The framework that "
                        <span>"resumes"</span>
                        " where the server left off"
                    </h1>
                    <p class="hero-lead">
                        "Resuma ships HTML plus a resumability payload. The browser never re-runs your components — it only resumes the interactions users actually trigger. ~884 B loader. Native Rust."
                    </p>
                    <div class="hero-actions">
                        <a href="/docs/getting_started" class="btn btn-primary">"Get Started"</a>
                        <a href="/docs" class="btn btn-ghost">"Read the Docs"</a>
                    </div>
                    <div class="hero-stats">
                        <div><strong>"~884 B"</strong>" loader (gzip)"</div>
                        <div><strong>"0"</strong>" JS on static pages"</div>
                        <div><strong>"1"</strong>" cargo dependency"</div>
                    </div>
                </div>
                <div class="hero-panel">
                    <div class="hero-panel-header">
                        <span></span><span></span><span></span>
                    </div>
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
}"#)}
                </div>
            </section>

            <section class="section">
                <h2 class="section-title">"Why Resuma?"</h2>
                <p class="section-sub">"Pick the best ideas from Qwik-style resumability and Leptos-grade Rust — without shipping a WASM hydration bundle."</p>
                <div class="grid-3">
                    {feature_card("⚡", "Resumable, not hydrated", "Components run on the server only. State and handlers serialize into HTML — the client resumes lazily.")}
                    {feature_card("🏝️", "Islands by default", "Mark interactive regions with #[island]. Each island ships its own JS chunk on demand.")}
                    {feature_card("🦀", "Server actions", "#[server] async fn registers RPC at /_resuma/action/:name — callable from handlers and forms.")}
                    {feature_card("🌊", "Resuma Flow", "Full-stack layer: #[load], #[submit], layouts, middleware, file-based pages — one dependency.")}
                    {feature_card("📦", "One package", "cargo add resuma gives you core + Flow + macros + server. Like Qwik + Qwik City, unified.")}
                    {feature_card("🔗", "JS bridge", "view! translates Rust closures to JS at compile time. js!{} for escape hatches.")}
                </div>
            </section>

            <section class="section">
                <h2 class="section-title">"Resuma¹ + Flow²"</h2>
                <p class="section-sub">"Two layers, one install. Core stays stable; Flow adds routing, data, and forms."</p>
                <div class="package-diagram">
                    <article class="package-box">
                        <p class="tag">"RESUMA¹ — CORE"</p>
                        <h3>"Components & resumability"</h3>
                        <ul>
                            <li>"view!, #[component], use_signal"</li>
                            <li>"#[server], #[island]"</li>
                            <li>"ResumaApp, SSR, ~3KB runtime"</li>
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
                <p style="margin-top: 1.5rem;">
                    <a href="/docs/package" class="btn btn-primary">"Install guide →"</a>
                </p>
            </section>

            <section class="section">
                <h2 class="section-title">"Compare"</h2>
                <table class="compare">
                    <thead>
                        <tr>
                            <th></th>
                            <th>"Hydration frameworks"</th>
                            <th>"Resuma"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr><td>"Initial JS"</td><td>"Full app bundle"</td><td class="yes">"~3KB runtime"</td></tr>
                        <tr><td>"Client execution"</td><td>"Re-run components"</td><td class="yes">"Resume handlers only"</td></tr>
                        <tr><td>"Islands"</td><td>"Add-on"</td><td class="yes">"First-class #[island]"</td></tr>
                        <tr><td>"Full-stack"</td><td>"Separate meta-framework"</td><td class="yes">"Flow included"</td></tr>
                        <tr><td>"Language"</td><td>"JS / WASM"</td><td class="yes">"Native Rust SSR"</td></tr>
                    </tbody>
                </table>
            </section>
        </main>
    }
}
