//! E2E showcase worker — registered via `#[worker]` for browser exec tests.

use resuma::exec::WorkerContext;
use resuma::prelude::*;
use resuma::worker;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct E2eShowcaseInput {
    pub topic: String,
}

#[worker(intent = "fast showcase worker for browser E2E")]
pub async fn e2e_showcase(input: E2eShowcaseInput, ctx: WorkerContext) -> Result<Value> {
    let topic = input.topic.trim();
    if topic.is_empty() {
        return Err(ResumaError::Validation("topic is required".into()));
    }

    ctx.log(format!("e2e_showcase: topic=\"{topic}\""));
    ctx.progress(10);
    tokio::time::sleep(std::time::Duration::from_millis(80)).await;
    ctx.log("streaming progress");
    ctx.progress(80);
    tokio::time::sleep(std::time::Duration::from_millis(80)).await;
    ctx.progress(100);

    Ok(json!({
        "topic": topic,
        "ok": true,
    }))
}
