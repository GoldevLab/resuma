//! Session middleware for production apps.
//!
//! **Important:** replace this stub before deploying. Set `RESUMA_OPS_SESSION` to a
//! long random secret; only clients presenting that cookie value get ops access.

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
        let req = resuma::flow::run_middleware(req).await?;
        let session = req
            .header("cookie")
            .and_then(|c| cookie_value(c, "resuma_session"));
        let ops_secret = std::env::var("RESUMA_OPS_SESSION")
            .ok()
            .filter(|s| !s.is_empty());
        let authenticated = match (&session, &ops_secret) {
            (Some(cookie), Some(secret)) => cookie == secret,
            _ => false,
        };
        let mut req = req;
        req.set_extension("user_id", serde_json::json!(session.unwrap_or_else(|| "guest".into())));
        req.set_extension("authenticated", serde_json::json!(authenticated));
        if authenticated {
            req.set_extension("roles", serde_json::json!(["admin"]));
        }
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
