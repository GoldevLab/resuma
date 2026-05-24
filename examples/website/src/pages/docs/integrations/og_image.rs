use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"OG Image"</h1>
            <p class="lead">"Dynamic social preview images — like Qwik's OG Image integration."</p>

            <h2>"Static default"</h2>
            <p>"Set " <code>"FlowApp::with_og_image(\"/og.svg\")"</code> " (this docs site). Good for most pages."</p>

            <h2>"Per-route OG via loader metadata"</h2>
            {code_block(r#"#[load]
async fn blog_post(req: &FlowRequest) -> Post {
    let slug = req.param("slug").unwrap_or("");
    db::post(slug).await
}

// In page head via FlowApp page options or custom endpoint:
// GET /og/blog/[slug].png -> image/png from resvg/usvg rendering title"#)}

            <h2>"Dedicated OG endpoint"</h2>
            {code_block(r#"// Register on ResumaApp / Flow fallback:
// Return SVG or PNG with post title + site branding
// Reference in meta: og:image = https://yoursite.com/og/post/my-slug.png"#)}

            <p>"Use " <code>"SITE_URL"</code> " env for absolute " <code>"og:url"</code> " and " <code>"og:image"</code> " URLs in production."</p>
        </>
    }
}
