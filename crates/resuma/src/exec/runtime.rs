//! Runtime targets — backend (Tokio/Axum), node (Resuma cluster), browser.

use super::types::RuntimeChoice;

/// Where execution runs in the Resuma ecosystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeTarget {
    /// Tokio + Axum — primary server (Fly.io, Docker, `resuma dev`).
    Backend,
    /// Resuma Node — isolated in-process worker with resource limits.
    Node,
    /// Browser resumability runtime (`@resuma/flow`).
    Browser,
}

impl From<RuntimeChoice> for RuntimeTarget {
    fn from(c: RuntimeChoice) -> Self {
        match c {
            RuntimeChoice::Backend => Self::Backend,
            RuntimeChoice::Node => Self::Node,
            RuntimeChoice::Browser => Self::Browser,
            RuntimeChoice::Hybrid | RuntimeChoice::Auto => Self::Backend,
        }
    }
}

/// Pick runtime from plan + resource profile.
pub fn route(
    plan: &super::types::ExecutionPlan,
    profile: &super::resources::ResourceProfile,
) -> RuntimeTarget {
    match profile.runtime {
        RuntimeChoice::Node => RuntimeTarget::Node,
        RuntimeChoice::Browser => RuntimeTarget::Browser,
        RuntimeChoice::Hybrid => RuntimeTarget::Backend,
        RuntimeChoice::Backend | RuntimeChoice::Auto => {
            if plan.chunks > 8 || plan.use_ai {
                RuntimeTarget::Backend
            } else {
                RuntimeTarget::from(plan.runtime)
            }
        }
    }
}
