use crate::page_impls::flow;
use resuma::prelude::*;

pub fn page(req: FlowRequest) -> View {
    flow::gestures(req)
}
