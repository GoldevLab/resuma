//! Booking template — query-driven `#[load]` + `loader_refresh_input`.

mod booking_store;
mod pages;

use pages::PagesRegistry;
use resuma::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookFormData {
    pub fecha: String,
    pub servicio: String,
    pub slots: Vec<String>,
}

#[load]
async fn book_form(req: &FlowRequest) -> BookFormData {
    let fecha = req.query_param("fecha").unwrap_or_default().to_string();
    let servicio = req.query_param("servicio").unwrap_or_default().to_string();
    let slots = booking_store::available_slots(&fecha)
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    BookFormData {
        fecha,
        servicio,
        slots,
    }
}

#[data]
struct BookInput {
    name: String,
    phone: String,
    service: String,
    date: String,
    time: String,
}

#[submit]
async fn book_slot(form: BookInput, _req: &FlowRequest) -> Result<Redirect, SubmitError> {
    match booking_store::book(
        form.name,
        form.phone,
        form.service,
        form.date,
        form.time,
    ) {
        Ok(()) => Ok(redirect(&format!("/gracias?fecha={}", form.date))),
        Err((field, msg)) => Err(SubmitError::new("Fix the highlighted fields.").field(field, msg)),
    }
}

#[layout("/")]
fn RootLayout() -> View {
    let theme = Theme {
        mode: "light".into(),
        primary: "#4f46e5".into(),
        background: "#f8fafc".into(),
        foreground: "#0f172a".into(),
    };
    provide_theme(theme.clone());

    view! {
        <div style={theme_css_vars(&use_theme())}>
            <nav>
                <NavLink href="/" activeClass="active">"Home"</NavLink>
                <NavLink href="/book" activeClass="active">"Book"</NavLink>
            </nav>
            <Slot />
        </div>
    }
}

const CSS: &str = r#"<style>
body { font-family: system-ui, sans-serif; margin: 0; background: var(--resuma-bg); color: var(--resuma-fg); }
nav { display: flex; gap: 1rem; padding: 1rem 1.5rem; }
nav a.active { color: var(--resuma-primary); font-weight: 600; }
main { max-width: 36rem; margin: 0 auto; padding: 1.5rem; }
.slot-grid { display: flex; flex-wrap: wrap; gap: .5rem; }
</style>"#;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let theme = Theme {
        primary: "#4f46e5".into(),
        background: "#f8fafc".into(),
        ..Default::default()
    };

    FlowApp::new()
        .with_title("%NAME% · Booking")
        .with_head(CSS)
        .with_theme_pwa(theme)
        .not_found(|| not_found_page())
        .auto_pages(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pages"),
            PagesRegistry,
        )
        .serve(FlowServeOptions::default())
        .await
}
