use resuma::prelude::*;
use resuma_flow::{flow_dashboard_poll, flow_styles};

pub fn page(req: FlowRequest) -> View {
    if !req.is_authenticated() && !req.has_role("admin") {
        return view! {
            <main class="card">
                <h1>"Ops dashboard"</h1>
                <p>"Sign in to view Resuma OS metrics."</p>
            </main>
        };
    }
    view! {
        <main>
            {flow_styles()}
            {flow_dashboard_poll(5000, Some(resuma::exec::exec_status()))}
        </main>
    }
}
