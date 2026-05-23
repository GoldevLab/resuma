//! Shared UI for the Resuma documentation site.

mod css;
mod pwa;
mod seo;
mod sidebar;

pub use css::SITE_CSS;
pub use pwa::config as pwa_config;
pub use seo::{json_ld, site_description, site_url};
pub use sidebar::doc_sidebar;

use resuma::prelude::*;

pub fn code_block(code: &str) -> View {
    view! {
        <pre class="code"><code>{code.to_string()}</code></pre>
    }
}

pub fn playground_card(title: &str, body: &str, command: &str) -> View {
    view! {
        <article class="playground-card">
            <h3>{title.to_string()}</h3>
            <p>{body.to_string()}</p>
            <code>{command.to_string()}</code>
        </article>
    }
}

pub fn feature_card(icon: &str, title: &str, body: &str) -> View {
    view! {
        <article class="card">
            <div class="card-icon">{icon.to_string()}</div>
            <h3>{title.to_string()}</h3>
            <p>{body.to_string()}</p>
        </article>
    }
}
