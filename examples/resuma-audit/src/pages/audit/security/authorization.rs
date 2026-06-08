use crate::page_impls::security;
use resuma::prelude::*;

pub fn page(req: FlowRequest) -> View {
    security::authorization(req)
}
