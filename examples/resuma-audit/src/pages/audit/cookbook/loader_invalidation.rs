use crate::page_impls::cookbook;
use resuma::prelude::*;

pub fn page(req: FlowRequest) -> View {
    cookbook::loader_invalidation(req)
}
