use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Configure security"</h1>
            <p class="lead">"Tune Resuma's HTTP hardening from Rust or environment variables — like Express Helmet + body-parser limits."</p>

            <h2>"ServeOptions + SecurityConfig"</h2>
            {code_block(r#"use resuma::prelude::*;

pub fn serve_options() -> ServeOptions {
    ServeOptions {
        addr: "127.0.0.1:3000".parse().unwrap(),
        security: SecurityConfig {
            csrf: true,
            origin_check: true,
            trust_proxy: true,          // Fly.io / nginx
            body_limit_bytes: 256 * 1024,
            actions_per_minute: 90,
            submits_per_minute: 45,
            hide_benchmark: true,
            production: true,
        },
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    ResumaApp::new()
        .page("/", || page())
        .serve(serve_options())
        .await
}"#)}

            <h2>"Environment variables"</h2>
            <p>"When you omit explicit config, " <code>"SecurityConfig::from_env()"</code> " reads:"</p>
            {code_block(r#"RESUMA_ENV=production        # generic client errors
RESUMA_TRUST_PROXY=1       # X-Forwarded-For / Proto
RESUMA_CSRF=1              # default on (set 0 to disable)
RESUMA_ORIGIN_CHECK=1      # default on
RESUMA_BODY_LIMIT=1048576
RESUMA_RATE_ACTIONS=120
RESUMA_RATE_SUBMITS=60"#)}

            <h2>"Flow apps"</h2>
            {code_block(r#"FlowApp::new()
    .pages_from_dir("src/pages")
    .serve(FlowServeOptions {
        addr: ([0, 0, 0, 0], 3000).into(),
        security: SecurityConfig::from_env(),
    })
    .await"#)}

            <h2>"Fly.io"</h2>
            <p>
                "Production deploy guide (Dockerfile, "
                <code>"fly.toml"</code> ", health checks): "
                <a href="/docs/cookbook/docker">"Docker deploy cookbook"</a>". "
                "This repo ships a working config at the workspace root — live at "
                <a href="https://resuma-docs.fly.dev/" target="_blank">"resuma-docs.fly.dev"</a>"."
            </p>
            {code_block(r#"# fly.toml [env] — minimum for production
RESUMA_ENV = "production"
RESUMA_TRUST_PROXY = "1"
SITE_URL = "https://your-app.fly.dev"
HOST = "0.0.0.0"
PORT = "3000"

[http_service]
  force_https = true"#)}

            <h2>"Security checklist before deploy"</h2>
            <ul>
                <li><code>"RESUMA_ENV=production"</code> " — sanitized errors"</li>
                <li><code>"RESUMA_TRUST_PROXY=1"</code> " — real client IP + HTTPS detection"</li>
                <li><code>"force_https = true"</code> " on Fly / reverse proxy"</li>
                <li>"Run container as non-root (see Dockerfile)"</li>
                <li>"Auth + validation on sensitive " <code>"#[server]"</code> " actions — " <a href="/docs/security/todo">"todo example"</a></li>
            </ul>
        </>
    }
}
