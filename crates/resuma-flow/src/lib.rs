//! # Resuma Flow (frontend package)
//!
//! Full-stack app layer + observable execution UI. Depends on [`resuma`] for the
//! execution layer (planner, graph runtime, event bus).
//!
//! ```no_run
//! use resuma_flow::prelude::*;
//! use resuma::exec::exec_status;
//!
//! #[component]
//! fn OpsPage() {
//!     view! {
//!         {flow_ops_page(exec_status())}
//!     }
//! }
//! ```

pub mod components;

pub use components::{
    event_stream, event_stream_auth, flow_dashboard, flow_dashboard_live, flow_dashboard_poll,
    flow_execution, flow_execution_auth, flow_graph, flow_graph_auth, flow_ops_page, flow_styles,
    worker_panel, worker_panel_auth,
};
pub use resuma::flow::*;
pub use resuma::{
    exec::{
        enqueue, exec_status, FlowEngine, GraphId, GraphSnapshot, StartWorkerResponse,
        WorkerContext, WorkerEvent, WorkerMeta, WorkerRegistry,
    },
    FlowApp, FlowServeOptions,
};

pub mod prelude {
    pub use crate::components::{
        event_stream, event_stream_auth, flow_dashboard, flow_dashboard_live, flow_dashboard_poll,
        flow_execution, flow_execution_auth, flow_graph, flow_graph_auth, flow_ops_page,
        flow_styles, worker_panel, worker_panel_auth,
    };
    pub use crate::{
        enqueue, exec_status, FlowApp, FlowEngine, FlowServeOptions, GraphId, WorkerContext,
        WorkerMeta, WorkerRegistry,
    };
    pub use resuma::prelude::*;
    pub use resuma::worker;
}
