//! Installs Resuma Flow middleware on `/_resuma/action/*` requests.

use std::future::Future;
use std::pin::Pin;

use crate::core::{FlowRequest, Result};

use super::middleware::run_middleware;

type ActionMiddlewareFuture = Pin<Box<dyn Future<Output = Result<FlowRequest>> + Send>>;

fn run_action_middleware(req: FlowRequest) -> ActionMiddlewareFuture {
    Box::pin(async move { run_middleware(req).await })
}

/// Called from a `#[ctor]` when the flow crate is linked.
pub fn install_action_middleware() {
    crate::server::set_action_middleware(run_action_middleware);
}

#[ctor::ctor(unsafe)]
fn auto_install_action_middleware() {
    install_action_middleware();
}
