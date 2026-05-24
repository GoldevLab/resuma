//! Shared UI for the Resuma documentation site.

mod css;
mod docs_search;
mod pwa;
mod seo;
mod sidebar;

pub use css::SITE_CSS;
pub use pwa::config as pwa_config;
pub use docs_search::search;
pub use seo::{json_ld, site_description, site_title, site_url};
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

pub fn pillar_card(icon: &str, title: &str, body: &str) -> View {
    view! {
        <article class="pillar">
            <div class="pillar-icon">{icon.to_string()}</div>
            <h3>{title.to_string()}</h3>
            <p>{body.to_string()}</p>
        </article>
    }
}

pub fn pipeline_step(num: &str, title: &str, body: &str) -> View {
    view! {
        <article class="pipeline-step">
            <span class="pipeline-num">{num.to_string()}</span>
            <h3>{title.to_string()}</h3>
            <p>{body.to_string()}</p>
        </article>
    }
}

pub fn metric_item(value: &str, label: &str) -> View {
    view! {
        <div class="metric-item">
            <strong>{value.to_string()}</strong>
            <span>{label.to_string()}</span>
        </div>
    }
}
