use crate::page_impls::reference;
use resuma::prelude::*;

pub fn page(req: FlowRequest) -> View {
    reference::cli(req)
}
