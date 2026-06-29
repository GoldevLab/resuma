//! Execution planner — intent → [`ExecutionPlan`].

use super::types::{ExecutionPlan, ExecutionStrategy, RuntimeChoice};

/// Heuristic planner (static analysis + intent keywords).
///
/// Future: AST pass over worker body to detect `ctx.tool` / `ctx.ai` calls.
pub fn plan(intent: &str, hints: PlannerHints) -> ExecutionPlan {
    let lower = intent.to_lowercase();
    let words = lower.split_whitespace().count();

    let use_ai = hints.use_ai
        || lower.contains("ai")
        || lower.contains("analiz")
        || lower.contains("classif")
        || lower.contains("analyze");

    let heavy = lower.contains("1m")
        || lower.contains("1000")
        || lower.contains("millón")
        || lower.contains("million")
        || words > 12;

    let strategy = if heavy && use_ai {
        ExecutionStrategy::MapReduce
    } else if lower.contains(" || ") || lower.contains(" parallel") {
        ExecutionStrategy::Parallel
    } else if lower.contains(" → ") || lower.contains(" then ") || lower.contains(" luego ") {
        ExecutionStrategy::Pipeline
    } else if hints.tools.len() > 2 {
        ExecutionStrategy::Hybrid
    } else {
        ExecutionStrategy::Single
    };

    let chunks = match strategy {
        ExecutionStrategy::MapReduce => 5,
        ExecutionStrategy::Parallel => 3,
        ExecutionStrategy::Hybrid => 2,
        _ => 1,
    };

    let parallel = matches!(
        strategy,
        ExecutionStrategy::Parallel | ExecutionStrategy::MapReduce
    );

    let runtime = if heavy || use_ai {
        RuntimeChoice::Backend
    } else {
        RuntimeChoice::Auto
    };

    let estimated_cost = if heavy {
        Some("high".into())
    } else if use_ai {
        Some("medium".into())
    } else {
        Some("low".into())
    };

    ExecutionPlan {
        runtime,
        strategy,
        chunks,
        parallel,
        use_ai,
        tools: hints.tools,
        estimated_cost,
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlannerHints {
    pub use_ai: bool,
    pub tools: Vec<String>,
}
