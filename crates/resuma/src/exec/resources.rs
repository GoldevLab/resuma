//! Resource profiles — invisible limits managed by the execution layer.

use serde::{Deserialize, Serialize};

use super::types::RuntimeChoice;

/// Per-dimension resource setting (`"auto"` or explicit).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResourceLevel {
    #[default]
    Auto,
    Named(String),
}

/// Resource declaration on a worker (user never tunes CPU/RAM manually).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Resources {
    #[serde(default)]
    pub cpu: ResourceLevel,
    #[serde(default)]
    pub memory: ResourceLevel,
    #[serde(default)]
    pub timeout: ResourceLevel,
    #[serde(default)]
    pub network: ResourceLevel,
    #[serde(default)]
    pub persistence: ResourceLevel,
}

impl Resources {
    pub fn auto() -> Self {
        Self::default()
    }

    /// Long CPU jobs (mesh gen, builds): 5 minutes by default.
    pub fn extended() -> Self {
        Self {
            timeout: ResourceLevel::Named("extended".into()),
            ..Self::default()
        }
    }

    /// No wall-clock timeout (`timeout_secs = 0` → cooperative cancel only).
    pub fn unlimited() -> Self {
        Self {
            timeout: ResourceLevel::Named("none".into()),
            ..Self::default()
        }
    }
}

/// Resolved profile after the planner + resource manager run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceProfile {
    pub runtime: RuntimeChoice,
    /// Wall-clock seconds; **0** means no timeout (cancel token only).
    pub timeout_secs: u64,
    pub memory_mb: u32,
    pub parallel_limit: u32,
}

impl Default for ResourceProfile {
    fn default() -> Self {
        let timeout_secs = std::env::var("RESUMA_WORKER_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        Self {
            runtime: RuntimeChoice::Backend,
            timeout_secs,
            memory_mb: 256,
            parallel_limit: 4,
        }
    }
}

/// Assign concrete resources from intent + plan heuristics.
pub fn resolve(resources: &Resources, plan: &super::types::ExecutionPlan) -> ResourceProfile {
    let runtime = match plan.runtime {
        RuntimeChoice::Auto => {
            if plan.strategy == super::types::ExecutionStrategy::Hybrid {
                RuntimeChoice::Hybrid
            } else if plan.use_ai || plan.chunks > 1 {
                RuntimeChoice::Backend
            } else {
                RuntimeChoice::Browser
            }
        }
        other => other,
    };

    let mut profile = ResourceProfile {
        runtime,
        ..ResourceProfile::default()
    };

    match &resources.timeout {
        ResourceLevel::Named(s) if s == "extended" => {
            profile.timeout_secs = 300;
        }
        ResourceLevel::Named(s) if s == "none" || s == "unlimited" || s == "0" => {
            profile.timeout_secs = 0;
        }
        ResourceLevel::Named(s) => {
            if let Ok(secs) = s.parse::<u64>() {
                profile.timeout_secs = secs;
            }
        }
        ResourceLevel::Auto => {}
    }
    if matches!(resources.memory, ResourceLevel::Named(ref s) if s == "high") {
        profile.memory_mb = 1024;
    }
    if plan.parallel {
        profile.parallel_limit = plan.chunks.max(1);
    }

    profile
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::types::{ExecutionPlan, ExecutionStrategy, RuntimeChoice};

    fn empty_plan() -> ExecutionPlan {
        ExecutionPlan {
            strategy: ExecutionStrategy::Single,
            runtime: RuntimeChoice::Backend,
            use_ai: false,
            chunks: 1,
            parallel: false,
            ..Default::default()
        }
    }

    #[test]
    fn extended_and_none_timeouts() {
        let plan = empty_plan();
        let ext = resolve(&Resources::extended(), &plan);
        assert_eq!(ext.timeout_secs, 300);
        let none = resolve(&Resources::unlimited(), &plan);
        assert_eq!(none.timeout_secs, 0);
        let custom = resolve(
            &Resources {
                timeout: ResourceLevel::Named("120".into()),
                ..Resources::auto()
            },
            &plan,
        );
        assert_eq!(custom.timeout_secs, 120);
    }
}
