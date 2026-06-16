//! Session + action middleware stub for production apps.

use resuma::prelude::*;

pub fn install() {
    set_action_middleware(action_pipeline);
}

fn action_pipeline(
    req: FlowRequest,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = std::result::Result<FlowRequest, ResumaError>> + Send>,
> {
    Box::pin(async move {
        let user = req
            .header("cookie")
            .and_then(|c| cookie_value(c, "resuma_session"))
            .unwrap_or_else(|| "guest".into());
        let mut req = req;
        req.set_extension("user_id", serde_json::json!(user));
        Ok(req)
    })
}

fn cookie_value(raw: &str, key: &str) -> Option<String> {
    raw.split(';').find_map(|part| {
        let (k, v) = part.trim().split_once('=')?;
        if k == key {
            Some(v.to_string())
        } else {
            None
        }
    })
}
