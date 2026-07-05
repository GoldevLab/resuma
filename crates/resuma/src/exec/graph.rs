//! Graph materialization + node status updates from events.

use super::id;
use super::types::{
    ExecutionPlan, GraphEdge, GraphId, GraphNodeSnapshot, GraphSnapshot, GraphStatus, NodeId,
    NodeKind, NodeStatus, WorkerEvent,
};

/// Build an executable graph snapshot from plan (templates).
pub fn materialize(
    graph_id: GraphId,
    worker: &str,
    intent: &str,
    plan: &ExecutionPlan,
) -> GraphSnapshot {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    let root = NodeId::new("worker");
    nodes.push(GraphNodeSnapshot {
        id: root.clone(),
        kind: NodeKind::Worker,
        label: worker.to_string(),
        status: NodeStatus::Pending,
        duration_ms: None,
    });

    match plan.strategy {
        super::types::ExecutionStrategy::Single => {}
        super::types::ExecutionStrategy::Pipeline => {
            let scrape = NodeId::new("scrape");
            let ai = NodeId::new("ai");
            nodes.push(GraphNodeSnapshot {
                id: scrape.clone(),
                kind: NodeKind::Tool,
                label: "scrape".into(),
                status: NodeStatus::Pending,
                duration_ms: None,
            });
            nodes.push(GraphNodeSnapshot {
                id: ai.clone(),
                kind: NodeKind::Ai,
                label: "ai".into(),
                status: NodeStatus::Pending,
                duration_ms: None,
            });
            edges.push(GraphEdge::ControlFlow {
                from: root.clone(),
                to: scrape.clone(),
            });
            edges.push(GraphEdge::ControlFlow {
                from: scrape,
                to: ai,
            });
        }
        super::types::ExecutionStrategy::Parallel | super::types::ExecutionStrategy::MapReduce => {
            let scrape = NodeId::new("scrape");
            nodes.push(GraphNodeSnapshot {
                id: scrape.clone(),
                kind: NodeKind::Tool,
                label: "scrape".into(),
                status: NodeStatus::Pending,
                duration_ms: None,
            });
            edges.push(GraphEdge::ControlFlow {
                from: root.clone(),
                to: scrape.clone(),
            });
            for i in 0..plan.chunks {
                let nid = NodeId::new(format!("ai-{i}"));
                nodes.push(GraphNodeSnapshot {
                    id: nid.clone(),
                    kind: NodeKind::Ai,
                    label: format!("ai chunk {i}"),
                    status: NodeStatus::Pending,
                    duration_ms: None,
                });
                edges.push(GraphEdge::ControlFlow {
                    from: scrape.clone(),
                    to: nid.clone(),
                });
            }
            let merge = NodeId::new("merge");
            nodes.push(GraphNodeSnapshot {
                id: merge.clone(),
                kind: NodeKind::Merge,
                label: "merge".into(),
                status: NodeStatus::Pending,
                duration_ms: None,
            });
            for i in 0..plan.chunks {
                edges.push(GraphEdge::ControlFlow {
                    from: NodeId::new(format!("ai-{i}")),
                    to: merge.clone(),
                });
            }
        }
        super::types::ExecutionStrategy::Hybrid => {
            let tool = NodeId::new("tool-node");
            nodes.push(GraphNodeSnapshot {
                id: tool.clone(),
                kind: NodeKind::Tool,
                label: "node tool".into(),
                status: NodeStatus::Pending,
                duration_ms: None,
            });
            edges.push(GraphEdge::ControlFlow {
                from: root.clone(),
                to: tool,
            });
        }
    }

    GraphSnapshot {
        id: graph_id,
        worker: worker.to_string(),
        intent: intent.to_string(),
        plan: plan.clone(),
        nodes,
        edges,
        status: GraphStatus::Running,
    }
}

/// Apply a worker event to node statuses in the snapshot.
pub fn apply_event(snapshot: &mut GraphSnapshot, event: &WorkerEvent) {
    let update_node =
        |snap: &mut GraphSnapshot, id: &NodeId, status: NodeStatus, duration_ms: Option<u64>| {
            if let Some(n) = snap.nodes.iter_mut().find(|n| n.id == *id) {
                n.status = status;
                if duration_ms.is_some() {
                    n.duration_ms = duration_ms;
                }
            }
        };

    match event {
        WorkerEvent::NodeStart { node, .. } => {
            update_node(snapshot, node, NodeStatus::Running, None);
        }
        WorkerEvent::NodeDone {
            node, duration_ms, ..
        } => {
            update_node(snapshot, node, NodeStatus::Done, Some(*duration_ms));
        }
        WorkerEvent::NodeFailed { node, .. } => {
            update_node(snapshot, node, NodeStatus::Failed, None);
            snapshot.status = GraphStatus::Failed;
        }
        WorkerEvent::ToolCall { node, .. } => {
            update_node(snapshot, node, NodeStatus::Running, None);
        }
        WorkerEvent::AiThinking { node, .. } => {
            update_node(snapshot, node, NodeStatus::Running, None);
        }
        WorkerEvent::GraphDone { .. } => {
            snapshot.status = GraphStatus::Done;
        }
        _ => {}
    }
}

/// Mark root worker node running at start.
pub fn mark_running(snapshot: &mut GraphSnapshot) {
    if let Some(n) = snapshot.nodes.first_mut() {
        n.status = NodeStatus::Running;
    }
}

pub fn mark_failed(snapshot: &mut GraphSnapshot) {
    snapshot.status = GraphStatus::Failed;
    if let Some(n) = snapshot.nodes.first_mut() {
        if n.status == NodeStatus::Running {
            n.status = NodeStatus::Failed;
        }
    }
}

pub fn mark_done(snapshot: &mut GraphSnapshot) {
    snapshot.status = GraphStatus::Done;
    if let Some(n) = snapshot.nodes.first_mut() {
        if n.status == NodeStatus::Running {
            n.status = NodeStatus::Done;
            n.duration_ms = Some(id::now_ms());
        }
    }
}
