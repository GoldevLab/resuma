//! Parallel graph execution — map-reduce and parallel strategies.

use std::sync::Arc;

use futures_util::stream::{self, StreamExt, TryStreamExt};
use parking_lot::RwLock;
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

use crate::core::{Result, ResumaError};

use super::cancel;
use super::events::{emit, SharedEventBus};
use super::graph;
use super::node;
use super::resources::ResourceProfile;
use super::state::StateStore;
use super::tools;
use super::types::{ExecutionPlan, ExecutionStrategy, GraphId, NodeId, NodeKind};
use super::workers::{WorkerContext, WorkerFn};

/// Run execution according to the planner strategy.
pub async fn run_with_plan(
    plan: &ExecutionPlan,
    input: Value,
    run: WorkerFn,
    graph_id: GraphId,
    bus: SharedEventBus,
    state: Arc<StateStore>,
    snapshot: Arc<RwLock<super::types::GraphSnapshot>>,
    profile: ResourceProfile,
    cancel: CancellationToken,
) -> Result<Value> {
    cancel::check(&cancel)?;
    match plan.strategy {
        ExecutionStrategy::MapReduce | ExecutionStrategy::Parallel => {
            run_map_reduce(
                plan, input, run, graph_id, bus, state, snapshot, profile, cancel,
            )
            .await
        }
        _ => {
            let ctx = WorkerContext::new(graph_id, bus, state, snapshot, cancel.clone());
            node::run_on_node(&profile, input, ctx, run, &cancel).await
        }
    }
}

async fn run_map_reduce(
    plan: &ExecutionPlan,
    input: Value,
    run: WorkerFn,
    graph_id: GraphId,
    bus: SharedEventBus,
    state: Arc<StateStore>,
    snapshot: Arc<RwLock<super::types::GraphSnapshot>>,
    profile: ResourceProfile,
    cancel: CancellationToken,
) -> Result<Value> {
    let ctx = WorkerContext::new(
        graph_id.clone(),
        bus.clone(),
        state.clone(),
        snapshot.clone(),
        cancel.clone(),
    );
    ctx.log("map-reduce: preparing chunks");

    let mut working_data = input.clone();
    if let Some(query) = input.get("query").and_then(|v| v.as_str()) {
        cancel::check(&cancel)?;
        ctx.log(format!("map-reduce: scrape `{query}`"));
        let scraped = ctx
            .tool("scrape", json!({ "query": query }))
            .await?;
        working_data = scraped;
    }

    let prompt = input
        .get("prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("Analyze this chunk and return structured JSON.");

    let chunks = split_chunks(&working_data, plan.chunks);
    ctx.log(format!(
        "map-reduce: {} chunks, parallel={}",
        chunks.len(),
        plan.parallel
    ));

    cancel::check(&cancel)?;

    let parallel = profile.parallel_limit.max(1) as usize;
    let sem = Arc::new(tokio::sync::Semaphore::new(parallel));

    let chunk_results: Vec<Value> = stream::iter(
        chunks
            .into_iter()
            .enumerate()
            .map(|(i, chunk)| {
                let node_id = NodeId::new(format!("ai-{i}"));
                let bus = bus.clone();
                let snap = snapshot.clone();
                let prompt = prompt.to_string();
                let token = cancel.clone();
                let sem = sem.clone();
                async move {
                    let _permit = sem
                        .acquire_owned()
                        .await
                        .map_err(|_| ResumaError::Cancelled)?;
                    cancel::check(&token)?;
                    let start_evt = emit::node_start(node_id.clone(), NodeKind::Ai);
                    graph::apply_event(&mut snap.write(), &start_evt);
                    bus.emit(start_evt);

                    let started = super::id::now_ms();
                    let out = cancel::run_cancellable(
                        &token,
                        tools::dispatch("ai", json!({ "prompt": prompt, "data": chunk })),
                    )
                    .await;
                    let duration = super::id::now_ms().saturating_sub(started);

                    match &out {
                        Ok(_) => {
                            let done = emit::node_done(node_id.clone(), duration);
                            graph::apply_event(&mut snap.write(), &done);
                            bus.emit(done);
                        }
                        Err(ResumaError::Cancelled) => return Err(ResumaError::Cancelled),
                        Err(e) => {
                            let fail = emit::node_failed(node_id.clone(), e.to_string());
                            graph::apply_event(&mut snap.write(), &fail);
                            bus.emit(fail);
                        }
                    }
                    out
                }
            }),
    )
    .buffer_unordered(parallel)
    .try_collect()
    .await?;

    cancel::check(&cancel)?;

    let merge_id = NodeId::new("merge");
    let merge_start = emit::node_start(merge_id.clone(), NodeKind::Merge);
    graph::apply_event(&mut snapshot.write(), &merge_start);
    bus.emit(merge_start);

    let merged = json!({
        "chunks": chunk_results,
        "count": chunk_results.len(),
    });
    state.set("map_reduce_merged", merged.clone());

    let merge_done = emit::node_done(merge_id.clone(), 0);
    graph::apply_event(&mut snapshot.write(), &merge_done);
    bus.emit(merge_done);

    ctx.progress(90);
    ctx.log("map-reduce: running worker on merged result");

    let final_input = json!({
        "merged": merged,
        "original": input,
    });

    node::run_on_node(
        &profile,
        final_input,
        WorkerContext::new(graph_id, bus, state, snapshot, cancel.clone()),
        run,
        &cancel,
    )
    .await
}

/// Split input into `n` balanced chunks for parallel map steps.
pub fn split_chunks(input: &Value, n: u32) -> Vec<Value> {
    let n = n.max(1) as usize;

    let items: Vec<Value> = input
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .or_else(|| input.as_array().cloned())
        .unwrap_or_else(|| vec![input.clone()]);

    if items.is_empty() {
        return vec![input.clone()];
    }

    if items.len() <= n {
        return items;
    }

    let chunk_size = (items.len() + n - 1) / n;
    items
        .chunks(chunk_size)
        .map(|slice| Value::Array(slice.to_vec()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_chunks_balanced() {
        let input = json!({ "items": [1, 2, 3, 4, 5] });
        let chunks = split_chunks(&input, 2);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], json!([1, 2, 3]));
        assert_eq!(chunks[1], json!([4, 5]));
    }
}
