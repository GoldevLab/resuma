//! Page registry helpers — bridges `resuma-router` discovery with `FlowApp`.

use std::path::Path;

use crate::router::{discover, layout_chain_for};

use super::request::FlowRequest;
use crate::core::view::View;

pub use crate::router::DiscoveredRoute;

/// Metadata for a discovered page (from `src/pages/`).
#[derive(Debug, Clone)]
pub struct DiscoveredPage {
    pub pattern: String,
    pub module: String,
    pub layouts: Vec<String>,
    pub is_dynamic: bool,
}

/// Scan `src/pages/` and return page metadata including layout chains.
pub fn discover_pages(root: impl AsRef<Path>) -> Vec<DiscoveredPage> {
    let routes = discover(root);
    let layouts: Vec<_> = routes
        .iter()
        .filter(|r| r.is_layout)
        .map(|r| (r.pattern.clone(), r.file.clone()))
        .collect();

    routes
        .into_iter()
        .filter(|r| !r.is_layout)
        .map(|r| {
            let layouts = layout_chain_for(&r.pattern, &layouts);
            let is_dynamic = r.pattern.contains(':') || r.pattern.contains('*');
            DiscoveredPage {
                pattern: r.pattern,
                module: r.module,
                layouts,
                is_dynamic,
            }
        })
        .collect()
}

/// Trait implemented by generated or hand-written page registries.
pub trait FlowPageRegistry: Send + Sync {
    fn render(&self, module: &str, req: FlowRequest) -> Option<View>;
}

/// Look up a discovered route by URL pattern.
pub fn find_page<'a>(pages: &'a [DiscoveredPage], pattern: &str) -> Option<&'a DiscoveredPage> {
    pages.iter().find(|p| p.pattern == pattern)
}
