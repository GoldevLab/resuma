//! Full docs navigation — mapped to Resuma APIs.

use resuma::prelude::*;

macro_rules! nav {
    ($($href:expr => $label:expr),+ $(,)?) => {
        view! {
            <nav>
                $(<NavLink href={$href} activeClass="active">{$label}</NavLink>)+
            </nav>
        }
    };
}

pub fn doc_sidebar(_active_path: &str) -> View {
    view! {
        <aside class="docs-sidebar">
            <form method="get" action="/docs/search" class="docs-search-form">
                <input type="search" name="q" placeholder="Search docs..." aria-label="Search docs" />
            </form>
            <h4>"Introduction"</h4>
            {nav!(
                "/docs" => "Overview",
                "/docs/getting_started" => "Getting Started",
                "/docs/examples" => "Examples",
                "/docs/project_structure" => "Project structure",
                "/docs/faq" => "FAQ",
            )}

            <h4>"Security"</h4>
            {nav!(
                "/docs/security" => "Overview",
                "/docs/security/configure" => "Configure server",
                "/docs/security/server_actions" => "Server actions",
                "/docs/security/middleware" => "Auth middleware",
                "/docs/security/authorization" => "Authorization & RLS",
                "/docs/security/backend_patterns" => "NestJS + Next.js",
                "/docs/security/todo" => "Todo example",
            )}

            <h4>"Components"</h4>
            {nav!(
                "/docs/components" => "Overview",
                "/docs/components/view" => "view!",
                "/docs/components/signals" => "Signals",
                "/docs/components/effects" => "Effects",
                "/docs/components/handlers" => "Handlers",
                "/docs/components/islands" => "Islands",
                "/docs/components/server" => "Server actions",
                "/docs/components/js" => "js!",
                "/docs/components/slots" => "Slots",
                "/docs/components/nav_link" => "NavLink",
                "/docs/components/form" => "Form",
                "/docs/components/store" => "Store",
                "/docs/components/context" => "Context",
                "/docs/components/tasks" => "Tasks",
            )}

            <h4>"Resuma Flow"</h4>
            {nav!(
                "/docs/flow" => "Overview",
                "/docs/flow/routing" => "Routing",
                "/docs/flow/pages" => "Pages",
                "/docs/flow/layouts" => "Layouts",
                "/docs/flow/loaders" => "Loaders",
                "/docs/flow/submits" => "Actions",
                "/docs/flow/middleware" => "Middleware",
                "/docs/flow/endpoints" => "Endpoints",
                "/docs/flow/errors" => "Error handling",
                "/docs/flow/caching" => "Caching",
                "/docs/flow/streaming" => "Streaming",
                "/docs/flow/prefetch" => "Prefetch",
            )}

            <h4>"Integrations"</h4>
            {nav!(
                "/docs/integrations" => "Overview",
                "/docs/integrations/sqlx" => "SQLx",
                "/docs/integrations/turso" => "Turso",
                "/docs/integrations/supabase" => "Supabase",
                "/docs/integrations/auth" => "Auth",
                "/docs/integrations/validator" => "Validation",
                "/docs/integrations/i18n" => "i18n",
                "/docs/integrations/tailwind" => "Tailwind",
                "/docs/integrations/og_image" => "OG Image",
                "/docs/integrations/e2e" => "E2E testing",
            )}

            <h4>"Cookbook"</h4>
            {nav!(
                "/docs/cookbook" => "Overview",
                "/docs/cookbook/debouncer" => "Debouncer",
                "/docs/cookbook/portals" => "Portals",
                "/docs/cookbook/view_transitions" => "View transitions",
                "/docs/cookbook/theme" => "Theme",
                "/docs/cookbook/streaming_loaders" => "Streaming loaders",
                "/docs/cookbook/prg" => "PRG pattern",
                "/docs/cookbook/loader_invalidation" => "Loader invalidation",
                "/docs/cookbook/docker" => "Docker deploy",
            )}

            <h4>"Reference"</h4>
            {nav!(
                "/docs/architecture" => "Architecture",
                "/docs/package" => "Package",
                "/docs/benchmark" => "Benchmark",
                "/docs/cli" => "CLI",
                "/docs/api" => "API reference",
            )}

            <h4>"Resources"</h4>
            <nav>
                <a href="https://crates.io/crates/resuma" target="_blank">"crates.io"</a>
                <a href="https://docs.rs/resuma" target="_blank">"docs.rs"</a>
                <a href="https://github.com/GolfredoPerezFernandez/resuma" target="_blank">"GitHub"</a>
            </nav>
        </aside>
    }
}
