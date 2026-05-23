//! Resuma Flow demo — loads, submits, layouts, middleware, slots, NavLink.

use resuma::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Greeting {
    message: String,
}

#[load(stream, cache = "public, max-age=60")]
async fn home(_req: &FlowRequest) -> Greeting {
    Greeting {
        message: "Welcome to Resuma Flow".into(),
    }
}

fn home_stream_view(data: &Greeting) -> View {
    view! {
        <>
            <h1>{data.message.clone()}</h1>
            <p>"Loaded via deferred " <code>"#[load(stream)]"</code> " · layout via " <code>"#[layout]"</code></p>
            <Form submit={contact}>
                <label>
                    "Name"
                    <input name="name" type="text" required=true />
                </label>
                <label>
                    "Email"
                    <input name="email" type="email" required=true />
                </label>
                <button type="submit">"Send"</button>
            </Form>
        </>
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContactForm {
    name: String,
    email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContactResult {
    ok: bool,
    summary: String,
}

#[submit]
async fn contact(data: ContactForm, _req: &FlowRequest) -> Result<ContactResult, SubmitError> {
    if data.name.trim().is_empty() {
        return Err(SubmitError::new("Fix the errors below.").field("name", "Name is required"));
    }
    if !data.email.contains('@') {
        return Err(SubmitError::new("Fix the errors below.").field("email", "Invalid email"));
    }
    Ok(ContactResult {
        ok: true,
        summary: format!("Thanks {}, we'll reply to {}.", data.name, data.email),
    })
}

#[middleware]
async fn log_requests(req: FlowRequest) -> resuma::Result<FlowRequest> {
    println!("[flow] {} {}", req.method, req.path);
    Ok(req)
}

#[layout("/")]
fn AppLayout() -> View {
    view! {
        <div class="shell">
            <nav class="nav">
                <NavLink href="/" activeClass="active">"Home"</NavLink>
            </nav>
            <Slot />
        </div>
    }
}

#[component]
fn PageShell() -> View {
    view! {
        <article class="card">
            <Slot name="header" />
            <section class="body">
                <Slot />
            </section>
        </article>
    }
}

#[component]
fn HomePage() -> View {
    match use_home_load() {
        LoadValue::Pending => view! {
            <PageShell>
                {stream_slot("home")}
            </PageShell>
        },
        LoadValue::Ok(data) => view! {
            <PageShell>
                {home_stream_view(&data)}
            </PageShell>
        },
        LoadValue::Err(err) => error_page(&FlowError::Loader(err)),
    }
}

const INLINE_CSS: &str = r#"<style>
* { box-sizing: border-box; }
body { font-family: ui-sans-serif, system-ui, sans-serif; background: #0b1020; color: #e6e8ee; margin: 0; min-height: 100vh; }
.shell { max-width: 42rem; margin: 0 auto; padding: 2rem 1rem; }
.nav { display: flex; gap: 1rem; margin-bottom: 1rem; }
.nav a { color: #b9bfd2; text-decoration: none; }
.nav a.active { color: #818cf8; font-weight: 600; }
.card { background: #14182b; border: 1px solid #2a2f4a; padding: 2rem; border-radius: 16px; }
.card h1 { margin: 0 0 1rem; }
.card p, .card label { color: #b9bfd2; display: block; margin: .75rem 0; }
.card input { width: 100%; margin-top: .35rem; padding: .5rem .65rem; border-radius: 8px; border: 1px solid #2a2f4a; background: #0b1020; color: inherit; }
.card button { background: #6366f1; color: white; border: 0; border-radius: 8px; padding: .55rem 1rem; font-weight: 600; cursor: pointer; margin-top: .5rem; }
.resuma-field-error { color: #f87171; font-size: .85rem; display: block; margin-top: .25rem; }
.resuma-stream-loading { color: #b9bfd2; font-style: italic; padding: 1rem 0; }
code { background: #0b1020; padding: .1rem .35rem; border-radius: 4px; }
</style>"#;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    FlowApp::new()
        .with_title("Resuma · Flow Demo")
        .with_head(INLINE_CSS)
        .streaming(true)
        .page_with_layouts("/", vec!["/".into()], |_req| {
            HomePage::render(HomePageProps::default())
        })
        .serve(FlowServeOptions::default())
        .await
}
