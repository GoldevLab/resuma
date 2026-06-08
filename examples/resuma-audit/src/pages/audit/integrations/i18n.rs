use crate::page_impls::integrations;
use resuma::prelude::*;

pub fn page(req: FlowRequest) -> View {
    integrations::i18n(req)
}
