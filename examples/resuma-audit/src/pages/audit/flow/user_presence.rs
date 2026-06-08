use crate::page_impls::flow;
use resuma::prelude::*;

pub fn page(req: FlowRequest) -> View {
    flow::user_presence(req)
}
