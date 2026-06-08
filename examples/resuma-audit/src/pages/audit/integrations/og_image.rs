use crate::page_impls::integrations;
use resuma::prelude::*;

pub fn page(req: FlowRequest) -> View {
    integrations::og_image(req)
}
