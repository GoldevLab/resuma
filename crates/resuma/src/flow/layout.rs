//! Layout registry and composition for Resuma Flow pages.

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::view::View;
use once_cell::sync::Lazy;
use parking_lot::RwLock;

use super::request::FlowRequest;

pub type LayoutFn = Arc<dyn Fn(FlowRequest, View) -> View + Send + Sync>;

static LAYOUTS: Lazy<RwLock<HashMap<String, LayoutFn>>> = Lazy::new(|| RwLock::new(HashMap::new()));

/// Register a layout for a URL prefix (`/`, `/users`, …).
pub fn register_layout(pattern: impl Into<String>, f: LayoutFn) {
    LAYOUTS.write().insert(pattern.into(), f);
}

/// Wrap `page` with layouts from most specific prefix to root.
pub fn apply_layouts(req: &FlowRequest, page: View, chain: &[String]) -> View {
    let layouts = LAYOUTS.read();
    let mut sorted: Vec<&String> = chain.iter().collect();
    sorted.sort_by_key(|p| std::cmp::Reverse(p.len()));

    let mut view = page;
    for pat in sorted {
        if let Some(layout) = layouts.get(pat.as_str()) {
            view = layout(req.clone(), view);
        }
    }
    view
}
