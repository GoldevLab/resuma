use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Integrations"</h1>
            <p class="lead">
                "Connect Resuma Flow to databases, auth, styling, and testing — "
                "same role as Qwik City Integrations."
            </p>

            <h2>"Recommended stack"</h2>
            <p>
                <a href="/docs/integrations/sqlx"><strong>"SQLx"</strong></a>
                " + "
                <a href="/docs/integrations/turso"><strong>"Turso"</strong></a>
                " for edge apps, or SQLx + "
                <a href="/docs/integrations/supabase">"Supabase"</a>
                " / Fly Postgres for SaaS."
            </p>

            <h2>"Database"</h2>
            <div class="grid-3">
                <a href="/docs/integrations/sqlx" class="card" style="text-decoration: none;">
                    <h3>"SQLx"</h3>
                    <p>"Type-safe SQL, migrations, loaders and submits."</p>
                </a>
                <a href="/docs/integrations/turso" class="card" style="text-decoration: none;">
                    <h3>"Turso"</h3>
                    <p>"Edge libSQL — file locally, remote in prod."</p>
                </a>
                <a href="/docs/integrations/supabase" class="card" style="text-decoration: none;">
                    <h3>"Supabase"</h3>
                    <p>"Managed Postgres + optional auth."</p>
                </a>
            </div>

            <h2>"Auth & forms"</h2>
            <div class="grid-3">
                <a href="/docs/integrations/auth" class="card" style="text-decoration: none;">
                    <h3>"Auth"</h3>
                    <p>"Sessions, middleware, protected loaders."</p>
                </a>
                <a href="/docs/integrations/validator" class="card" style="text-decoration: none;">
                    <h3>"Validation"</h3>
                    <p>"validator crate in #[submit] handlers."</p>
                </a>
            </div>

            <h2>"UI & SEO"</h2>
            <div class="grid-3">
                <a href="/docs/integrations/tailwind" class="card" style="text-decoration: none;">
                    <h3>"Tailwind CSS"</h3>
                    <p>"Utility CSS build pipeline."</p>
                </a>
                <a href="/docs/integrations/og_image" class="card" style="text-decoration: none;">
                    <h3>"OG Image"</h3>
                    <p>"Social preview cards."</p>
                </a>
                <a href="/docs/integrations/i18n" class="card" style="text-decoration: none;">
                    <h3>"i18n"</h3>
                    <p>"Locale loaders and Fluent."</p>
                </a>
            </div>

            <h2>"Testing"</h2>
            <div class="grid-3">
                <a href="/docs/integrations/e2e" class="card" style="text-decoration: none;">
                    <h3>"E2E testing"</h3>
                    <p>"Playwright against SSR + submits."</p>
                </a>
            </div>

            <h2>"CLI scaffolding"</h2>
            {code_block("resuma new my-app --template flow-fullstack\nresuma add sqlx\nresuma add turso")}

            <h2>"Flow API mapping"</h2>
            <table class="docs-table">
                <thead>
                    <tr><th>"Qwik City"</th><th>"Resuma Flow"</th></tr>
                </thead>
                <tbody>
                    <tr><td><code>"routeLoader$"</code></td><td><code>"#[load]"</code></td></tr>
                    <tr><td><code>"routeAction$"</code></td><td><code>"#[submit]"</code></td></tr>
                    <tr><td><code>"server$"</code></td><td><code>"#[server]"</code></td></tr>
                </tbody>
            </table>
        </>
    }
}
