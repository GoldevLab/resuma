use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Tailwind CSS"</h1>
            <p class="lead">"Utility-first CSS with a build step — same role as Qwik's Tailwind integration."</p>

            <h2>"Option A: Standalone CSS file"</h2>
            {code_block(r#"# Build tailwind.css in dev/CI (npm)
npx tailwindcss -i ./styles/input.css -o ./assets/site.css --minify

# FlowApp
FlowApp::new().with_stylesheet("/assets/site.css")"#)}

            <h2>"Option B: Inline for prototypes"</h2>
            <p>"Keep " <code>"with_head(CSS)"</code> " with hand-written utilities (this docs site uses custom CSS in " <code>"SITE_CSS"</code>")."</p>

            <h2>"Content paths"</h2>
            {code_block(r#"// tailwind.config.js
content: ["./src/**/*.rs"]  // scan view! class strings at build time"#)}
        </>
    }
}
