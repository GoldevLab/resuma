use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Prefetch"</h1>
            <p class="lead">"Resuma prefetches lazy handler chunks when resumable boundaries enter the viewport (v0.3+)."</p>

            <h2>"Handler prefetch (automatic)"</h2>
            <p>
                "Every " <code>"#[component]"</code> " registers " <code>"/_resuma/handler/{Name}.js"</code> ". "
                "The ~884 B loader uses " <code>"IntersectionObserver"</code> " to prefetch handlers before the user clicks."
            </p>

            <h2>"Loader prefetch (app-level)"</h2>
            <p>
                "Use " <code>"NavLink"</code> " for internal routes — the browser fetches the next page on hover/focus when you add "
                <code>"data-r-prefetch"</code> " (future) or short " <code>"cache"</code> " on loaders for CDN edge caching:"
            </p>
            {code_block(r#"#[load(cache = "public, max-age=30")]
async fn docs_index(_req: &FlowRequest) -> DocsNav {
    DocsNav { sections: list_sections().await }
}"#)}

            <h2>"Related"</h2>
            <ul>
                <li><a href="/docs/flow/caching">"Caching"</a></li>
                <li><a href="/docs/cookbook/loader_invalidation">"Loader invalidation"</a></li>
                <li><a href="/docs/architecture">"Architecture"</a></li>
            </ul>
        </>
    }
}
