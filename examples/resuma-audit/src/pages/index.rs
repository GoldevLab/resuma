//! Home page — audit dashboard.

use crate::audit_shell::{self, AuditSection, SECTIONS};
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <div class="hero">
            <h1>"Resuma Full Audit"</h1>
            <p>
                "Interactive verification of every section documented at "
                <a href="https://resuma-docs.fly.dev/docs" target="_blank">"resuma-docs.fly.dev"</a>
                ". Each route tests one docs topic."
            </p>
            <p class="pill">"907 B loader · zero hydration · resumable SSR"</p>
            <p><NavLink href="/audit/reference/matrix" activeClass="active">"View full audit matrix →"</NavLink></p>
        </div>
        <div class="grid">
        {SECTIONS.iter().map(|(section, items)| {
            view! {
                <div class="section-card">
                    <h2>{(*section).to_string()}</h2>
                    <ul>
                        {items.iter().map(|AuditSection { title, href }| {
                            view! {
                                <li><NavLink href={href.to_string()} activeClass="active">{(*title).to_string()}</NavLink></li>
                            }
                        }).collect::<Vec<_>>()}
                    </ul>
                </div>
            }
        }).collect::<Vec<_>>()}
        </div>
    }
}
