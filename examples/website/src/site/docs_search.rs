//! Static docs search index (server-side filter, zero client JS).

#[derive(Clone, Copy)]
pub struct DocEntry {
    pub title: &'static str,
    pub href: &'static str,
    pub keywords: &'static str,
}

pub const ENTRIES: &[DocEntry] = &[
    DocEntry { title: "Overview", href: "/docs", keywords: "introduction start" },
    DocEntry { title: "Getting Started", href: "/docs/getting_started", keywords: "install cli scaffold tutorial" },
    DocEntry { title: "Integrations", href: "/docs/integrations", keywords: "database sql turso drizzle" },
    DocEntry { title: "SQLx", href: "/docs/integrations/sqlx", keywords: "postgres sqlite orm query migrate" },
    DocEntry { title: "Turso", href: "/docs/integrations/turso", keywords: "libsql edge sqlite" },
    DocEntry { title: "Auth", href: "/docs/integrations/auth", keywords: "session login jwt middleware cookie" },
    DocEntry { title: "Supabase", href: "/docs/integrations/supabase", keywords: "postgres hosted backend" },
    DocEntry { title: "Validation", href: "/docs/integrations/validator", keywords: "validator zod submit form" },
    DocEntry { title: "E2E testing", href: "/docs/integrations/e2e", keywords: "playwright test integration" },
    DocEntry { title: "i18n", href: "/docs/integrations/i18n", keywords: "translation locale fluent" },
    DocEntry { title: "Tailwind CSS", href: "/docs/integrations/tailwind", keywords: "css styling utility" },
    DocEntry { title: "OG Image", href: "/docs/integrations/og_image", keywords: "open graph social preview" },
    DocEntry { title: "Loaders", href: "/docs/flow/loaders", keywords: "load routeLoader data fetch" },
    DocEntry { title: "Prefetch", href: "/docs/flow/prefetch", keywords: "viewport lazy handler" },
    DocEntry { title: "Caching", href: "/docs/flow/caching", keywords: "cache-control max-age" },
    DocEntry { title: "Loader invalidation", href: "/docs/cookbook/loader_invalidation", keywords: "revalidate stale" },
    DocEntry { title: "PRG pattern", href: "/docs/cookbook/prg", keywords: "post redirect get submit" },
    DocEntry { title: "Flow", href: "/docs/flow", keywords: "pages routing fullstack" },
    DocEntry { title: "Security", href: "/docs/security", keywords: "csrf rate limit auth" },
    DocEntry { title: "CLI", href: "/docs/cli", keywords: "dev build routes add" },
    DocEntry { title: "Package", href: "/docs/package", keywords: "crates install dependencies" },
    DocEntry { title: "Architecture", href: "/docs/architecture", keywords: "resumability hydration ssr" },
];

pub fn search(query: &str) -> Vec<DocEntry> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return ENTRIES.to_vec();
    }
    ENTRIES
        .iter()
        .copied()
        .filter(|e| {
            e.title.to_lowercase().contains(&q)
                || e.href.to_lowercase().contains(&q)
                || e.keywords.to_lowercase().contains(&q)
        })
        .collect()
}
