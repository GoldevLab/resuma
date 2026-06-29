//! Resource profiles — invisible limits managed by the execution layer.

use serde::{Deserialize, Serialize};

use super::types::RuntimeChoice;

/// Per-dimension resource setting (`"auto"` or explicit).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResourceLevel {
    Auto,
    Named(String),
}

impl Default for ResourceLevel {
    fn default() -> Self {
        Self::Auto
    }
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
}

/// Resolved profile after the planner + resource manager run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceProfile {
    pub runtime: RuntimeChoice,
    pub timeout_secs: u64,
    pub memory_mb: u32,
    pub parallel_limit: u32,
}

impl Default for ResourceProfile {
    fn default() -> Self {
        Self {
            runtime: RuntimeChoice::Backend,
            timeout_secs: 30,
            memory_mb: 256,
            parallel_limit: 4,
        }
    }
}

/// Assign concrete resources from intent + plan heuristics.
pub fn resolve(resources: &Resources, plan: &super::types::ExecutionPlan) -> ResourceProfile {
    let mut profile = ResourceProfile::default();

    profile.runtime = match plan.runtime {
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

    if matches!(resources.timeout, ResourceLevel::Named(ref s) if s == "extended") {
        profile.timeout_secs = 300;
    }
    if matches!(resources.memory, ResourceLevel::Named(ref s) if s == "high") {
        profile.memory_mb = 1024;
    }
    if plan.parallel {
        profile.parallel_limit = plan.chunks.max(1);
    }

    profile
}
