use resuma::prelude::*;
use crate::site::code_block;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Docker Deploy"</h1>
            <p class="lead">"Build a minimal production image for a Resuma Flow app."</p>

            <h2>"Dockerfile"</h2>
            <p>"This repo includes a production Dockerfile at the workspace root for the docs site binary " <code>"website"</code> "."</p>
            {code_block(r##"FROM rust:1-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY examples ./examples
COPY runtime ./runtime
RUN cargo build --release -p example-website

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/website /app/website
COPY --from=builder /app/examples/website/src/pages /app/pages
ENV HOST=0.0.0.0
ENV PORT=3000
ENV RESUMA_PAGES_ROOT=/app/pages
EXPOSE 3000
CMD ["/app/website"]"##)}

            <h2>"Build and run locally"</h2>
            {code_block(r#"docker build -t resuma-docs .
docker run -p 3000:3000 -e HOST=0.0.0.0 -e PORT=3000 resuma-docs"#)}

            <h2>"Fly.io"</h2>
            <p>"A " <code>"fly.toml"</code> " is included at the repo root (same pattern as other apps in your Fly org)."</p>
            {code_block(r##"# First time (creates app resuma-docs in iad)
fly launch --no-deploy

# Deploy
fly deploy

# Open in browser
fly open"##)}

            <h2>"Bind address"</h2>
            <p>"Flow reads " <code>"RESUMA_ADDR"</code> " or " <code>"HOST"</code> " + " <code>"PORT"</code> ". Fly sets " <code>"HOST=0.0.0.0"</code> " and " <code>"PORT=3000"</code> " in fly.toml."</p>

            <h2>"Notes"</h2>
            <ul>
                <li>"Replace " <code>"example-website"</code> " / " <code>"website"</code> " with your crate and binary names."</li>
                <li>"Prebuilt JS runtime assets are embedded in resuma-server — no Node.js in the runtime image."</li>
                <li>"Set " <code>"RESUMA_ENV=production"</code> " and " <code>"RESUMA_TRUST_PROXY=1"</code> " — see " <a href="/docs/security/configure">"Configure security"</a>"."</li>
                <li>"Health check hits " <code>"/"</code> "; " <code>"/robots.txt"</code> " and " <code>"/sitemap.xml"</code> " on Flow apps."</li>
            </ul>
        </>
    }
}
