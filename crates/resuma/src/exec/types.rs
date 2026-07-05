//! Shared execution types — wire protocol between `resuma` (back) and `resuma-flow` (front).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Stable identifier for a running execution graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GraphId(pub String);

impl GraphId {
    pub fn new() -> Self {
        Self(format!("g_{}", crate::server::security::random_token()))
    }
}

impl Default for GraphId {
    fn default() -> Self {
        Self::new()
    }
}

/// Node within an [`ExecutionGraph`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new(label: impl Into<String>) -> Self {
        Self(label.into())
    }
}

/// Lifecycle of a graph execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphStatus {
    Running,
    Paused,
    Done,
    Failed,
}

/// Kind of executable node in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Worker,
    Tool,
    Ai,
    Transform,
    Merge,
    Checkpoint,
}

/// Per-node execution status (mirrors UI states).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Pending,
    Running,
    Done,
    Failed,
    Paused,
}

/// Execution strategy chosen by the planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStrategy {
    Single,
    Pipeline,
    Parallel,
    MapReduce,
    Hybrid,
}

/// Where the graph (or subgraph) runs — Resuma's own runtimes (no external edge vendor).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeChoice {
    #[default]
    Auto,
    Backend,
    /// Lightweight Resuma node (same cluster, isolated task + limits).
    #[serde(alias = "edge")]
    Node,
    Browser,
    Hybrid,
}

/// Plan produced before execution starts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub runtime: RuntimeChoice,
    pub strategy: ExecutionStrategy,
    pub chunks: u32,
    pub parallel: bool,
    pub use_ai: bool,
    pub tools: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_cost: Option<String>,
}

impl Default for ExecutionPlan {
    fn default() -> Self {
        Self {
            runtime: RuntimeChoice::Auto,
            strategy: ExecutionStrategy::Single,
            chunks: 1,
            parallel: false,
            use_ai: false,
            tools: Vec::new(),
            estimated_cost: None,
        }
    }
}

/// Directed edge in the execution graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GraphEdge {
    DataFlow {
        from: NodeId,
        to: NodeId,
        key: Option<String>,
    },
    ControlFlow {
        from: NodeId,
        to: NodeId,
    },
}

/// Snapshot of a node for API / UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNodeSnapshot {
    pub id: NodeId,
    pub kind: NodeKind,
    pub label: String,
    pub status: NodeStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// Full graph snapshot returned by `GET /_resuma/graph/:id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSnapshot {
    pub id: GraphId,
    pub worker: String,
    pub intent: String,
    pub plan: ExecutionPlan,
    pub nodes: Vec<GraphNodeSnapshot>,
    pub edges: Vec<GraphEdge>,
    pub status: GraphStatus,
}

/// Domain events emitted during worker execution (SSE + replay).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkerEvent {
    Log {
        message: String,
        node: NodeId,
        timestamp_ms: u64,
    },
    Progress {
        value: u8,
        node: NodeId,
        timestamp_ms: u64,
    },
    AiThinking {
        content: String,
        node: NodeId,
        timestamp_ms: u64,
    },
    ToolCall {
        tool: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<Value>,
        node: NodeId,
        timestamp_ms: u64,
    },
    NodeStart {
        node: NodeId,
        kind: NodeKind,
        timestamp_ms: u64,
    },
    NodeDone {
        node: NodeId,
        duration_ms: u64,
        timestamp_ms: u64,
    },
    NodeFailed {
        node: NodeId,
        error: String,
        timestamp_ms: u64,
    },
    Result {
        data: Value,
        timestamp_ms: u64,
    },
    GraphDone {
        graph_id: GraphId,
        timestamp_ms: u64,
    },
}

impl WorkerEvent {
    pub fn timestamp_ms(&self) -> u64 {
        match self {
            Self::Log { timestamp_ms, .. }
            | Self::Progress { timestamp_ms, .. }
            | Self::AiThinking { timestamp_ms, .. }
            | Self::ToolCall { timestamp_ms, .. }
            | Self::NodeStart { timestamp_ms, .. }
            | Self::NodeDone { timestamp_ms, .. }
            | Self::NodeFailed { timestamp_ms, .. }
            | Self::Result { timestamp_ms, .. }
            | Self::GraphDone { timestamp_ms, .. } => *timestamp_ms,
        }
    }

    pub fn node_id(&self) -> Option<&NodeId> {
        match self {
            Self::Log { node, .. }
            | Self::Progress { node, .. }
            | Self::AiThinking { node, .. }
            | Self::ToolCall { node, .. }
            | Self::NodeStart { node, .. }
            | Self::NodeDone { node, .. }
            | Self::NodeFailed { node, .. } => Some(node),
            Self::Result { .. } | Self::GraphDone { .. } => None,
        }
    }
}

/// Response from `POST /_resuma/worker/:name`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWorkerResponse {
    pub graph_id: GraphId,
    pub plan: ExecutionPlan,
    /// Scoped token for graph UI routes (SSE, pause, replay). Pass to `flow_graph(..., token)`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
}
