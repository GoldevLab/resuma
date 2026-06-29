use resuma::prelude::*;
use resuma::FlowPageRegistry;

pub struct PagesRegistry;

impl FlowPageRegistry for PagesRegistry {
    fn render(&self, module: &str, req: FlowRequest) -> Option<View> {
        match module {
            "index" => Some(super::index::page(req)),
            "ops" => Some(super::ops::page(req)),
            _ => None,
        }
    }
}
