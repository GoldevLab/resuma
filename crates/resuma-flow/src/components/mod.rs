//! Observable execution UI — live graph + event stream + ops dashboard.

mod event_stream;
mod flow_dashboard;
mod flow_execution;
mod flow_graph;
mod styles;
mod worker_panel;

pub use event_stream::{event_stream, event_stream_auth};
pub use flow_dashboard::{flow_dashboard, flow_dashboard_live, flow_dashboard_poll};
pub use flow_execution::{
    flow_execution, flow_execution_auth, flow_execution_panel_auth, flow_ops_page,
};
pub use flow_graph::{flow_graph, flow_graph_auth};
pub use styles::{flow_styles, flow_styles_link, FLOW_CSS};
pub use worker_panel::{worker_panel, worker_panel_auth};
