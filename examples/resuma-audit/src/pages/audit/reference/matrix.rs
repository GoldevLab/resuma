use crate::page_impls::matrix;
use resuma::prelude::*;

pub fn page(req: FlowRequest) -> View {
    matrix::matrix(req)
}
