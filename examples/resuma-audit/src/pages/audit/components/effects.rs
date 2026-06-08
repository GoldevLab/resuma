use crate::page_impls::components;
use resuma::prelude::*;

pub fn page(req: FlowRequest) -> View {
    components::effects(req)
}
