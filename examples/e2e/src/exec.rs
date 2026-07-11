//! Resuma OS browser E2E — dynamic Flow widget mount (same pattern as resuma-docs).

use resuma::prelude::*;
use resuma_flow::flow_styles;
use serde_json::{json, Value};

#[server]
async fn start_e2e_worker(topic: String) -> Result<Value> {
    let started =
        resuma::exec::FlowEngine::start("e2e_showcase", json!({ "topic": topic })).await?;
    Ok(json!({
        "graph_id": started.graph_id.0,
        "access_token": started.access_token.unwrap_or_default(),
    }))
}

#[component]
pub fn ExecPage() {
    view! {
        <main>
            {flow_styles()}
            <h1>"Resuma OS E2E"</h1>
            <p data-testid="exec-lead">"Live worker + execution graph"</p>
            <label>
                "Topic"
                <input id="exec-topic" type="text" value="E2E worker" />
            </label>
            <button
                type="button"
                data-testid="exec-start"
                onClick={js!(async (_event, _state, __resuma) => {
                    const topic = document.getElementById("exec-topic").value;
                    const errEl = document.getElementById("exec-err");
                    const slot = document.getElementById("exec-flow-slot");
                    const btn = document.querySelector("[data-testid=\"exec-start\"]");
                    errEl.hidden = true;
                    btn.disabled = true;
                    const res = await __resuma.safeAction("start_e2e_worker", [topic]);
                    btn.disabled = false;
                    if (!res.ok) {
                        errEl.textContent = res.error;
                        errEl.hidden = false;
                        return;
                    }
                    const graphId = res.value.graph_id;
                    const token = res.value.access_token || "";
                    if (window.__resumaCoreReady) await window.__resumaCoreReady;
                    let flow;
                    try {
                        flow = await import("/_resuma/flow.js");
                    } catch (e) {
                        errEl.textContent = "Could not load Flow widgets: " + String(e);
                        errEl.hidden = false;
                        return;
                    }
                    const prev = slot.querySelector("[data-r-flow-execution]");
                    if (prev) flow.disconnectFlowWidgets(prev);
                    slot.innerHTML = "";
                    const panel = document.createElement("div");
                    panel.className = "r-flow-exec";
                    panel.setAttribute("data-r-flow-execution", graphId);
                    panel.innerHTML =
                        "<div class=\"r-flow-exec__panel\">" +
                        "<h3>Execution graph</h3>" +
                        "<div class=\"r-flow-graph\" data-r-flow-graph=\"" + graphId + "\" data-r-flow-graph-live=\"true\" data-r-graph-token=\"" + token + "\">" +
                        "<div class=\"r-flow-graph__track\" data-r-flow-graph-track=\"true\">...</div>" +
                        "<p class=\"r-flow-graph__status\" data-r-flow-graph-status=\"true\">Loading graph...</p>" +
                        "</div></div>" +
                        "<aside class=\"r-flow-exec__side\">" +
                        "<div class=\"r-flow-exec__panel\"><h3>Event stream</h3>" +
                        "<div class=\"r-event-stream\" data-r-event-stream=\"" + graphId + "\" data-r-graph-token=\"" + token + "\">" +
                        "<ul class=\"r-event-stream-list\">Waiting for events...</ul>" +
                        "</div></div></aside>";
                    slot.appendChild(panel);
                    slot.hidden = false;
                    flow.initFlowWidgets(slot, { flush: false });
                })}
            >
                "Run worker"
            </button>
            <p id="exec-err" role="alert" hidden></p>
            <div id="exec-flow-slot" hidden></div>
        </main>
    }
}
