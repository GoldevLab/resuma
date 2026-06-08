use crate::page_impls::intro;
use resuma::prelude::*;

pub fn page(req: FlowRequest) -> View {
    intro::benchmark(req)
}
