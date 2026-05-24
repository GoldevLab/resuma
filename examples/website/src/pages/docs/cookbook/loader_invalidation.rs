use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Loader invalidation"</h1>
            <p class="lead">"Refresh stale " <code>"#[load]"</code> " data after mutations — similar to revalidatePath in Next.js."</p>

            <h2>"Short cache TTL"</h2>
            {code_block(r#"#[load(cache = "public, max-age=10")]
async fn product_list(_req: &FlowRequest) -> Vec<Product> {
    db::products().await
}"#)}

            <h2>"Private per-user data"</h2>
            {code_block(r#"#[load(cache = "private, no-store")]
async fn cart(req: &FlowRequest) -> Cart {
    cart_for(req.user_id()).await
}"#)}

            <h2>"After submit: full page navigation"</h2>
            <p>
                "The simplest invalidation — redirect to GET (see "
                <a href="/docs/cookbook/prg">"PRG pattern"</a>
                "). The next SSR run re-executes all loaders."
            </p>

            <h2>"set_load_cache (runtime)"</h2>
            {code_block(r#"// After successful mutation in #[server] or enhanced submit client path:
set_load_cache("product_list", "public, max-age=0");"#)}

            <h2>"FlowExtensions for DB"</h2>
            <p>
                "Use " <code>"FlowApp::with_extension(\"db\", \"ready\")"</code> " so loaders know the pool is initialized. "
                "See " <a href="/docs/integrations">"Integrations"</a>"."
            </p>
        </>
    }
}
