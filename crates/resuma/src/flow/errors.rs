//! Error pages and boundary helpers for Resuma Flow.

use crate::core::view::{Attr, AttrValue, Child, Element, View};
use crate::core::ResumaError;

use super::load::LoaderError;

/// Unified page-level error for Flow renders.
#[derive(Debug)]
pub enum FlowError {
    NotFound,
    Loader(LoaderError),
    Render(String),
    Middleware(ResumaError),
}

impl FlowError {
    pub fn status(&self) -> u16 {
        match self {
            FlowError::NotFound => 404,
            FlowError::Loader(e) => e.status,
            FlowError::Render(_) => 500,
            FlowError::Middleware(e) => e.status_code(),
        }
    }

    pub fn message(&self) -> &str {
        match self {
            FlowError::NotFound => "Page not found",
            FlowError::Loader(e) => &e.message,
            FlowError::Render(m) => m,
            FlowError::Middleware(e) => match e {
                ResumaError::Unauthorized => "Unauthorized",
                ResumaError::Forbidden(_) => "Forbidden",
                ResumaError::InvalidCsrf => "Invalid CSRF token",
                ResumaError::RateLimited => "Too many requests",
                ResumaError::Validation(message) => message,
                _ => "Request blocked",
            },
        }
    }

    pub fn from_resuma(err: ResumaError) -> Self {
        Self::Middleware(err)
    }
}

/// Default error page view.
///
/// Stages the error's HTTP status so the response is returned with the correct
/// code (401/403/404/500…) instead of a soft `200 OK`.
pub fn error_page(err: &FlowError) -> View {
    crate::server::page_cache::stage_response_status(err.status());
    view_error_shell(err.status(), err.message())
}

/// 404 page.
pub fn not_found_page() -> View {
    error_page(&FlowError::NotFound)
}

/// 500 when a route is registered but its page module is missing from the registry.
pub fn registry_miss_page(module: &str) -> View {
    error_page(&FlowError::Render(format!(
        "page module `{module}` missing from registry — run `resuma routes --generate`"
    )))
}

fn view_error_shell(status: u16, message: &str) -> View {
    View::Element(Element {
        tag: "main".into(),
        attrs: vec![
            Attr {
                name: "class".into(),
                value: AttrValue::Static("resuma-error".into()),
            },
            Attr {
                name: "data-r-error".into(),
                value: AttrValue::Static(status.to_string()),
            },
        ],
        children: vec![
            Child::View(View::Element(Element {
                tag: "h1".into(),
                attrs: vec![],
                children: vec![Child::Text(format!("Error {status}"))],
                dom_id: None,
            })),
            Child::View(View::Element(Element {
                tag: "p".into(),
                attrs: vec![],
                children: vec![Child::Text(message.to_string())],
                dom_id: None,
            })),
        ],
        dom_id: None,
    })
}
