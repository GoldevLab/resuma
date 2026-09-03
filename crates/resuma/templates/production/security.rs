//! Session middleware for production apps.
//!
//! **Important:** replace this stub before deploying. Set `RESUMA_OPS_SESSION` to a
//! long random secret; only clients presenting that cookie value get ops access.
//!
//! Uses `#[middleware]` so the session is attached on **pages and actions**,
//! not only `#[server]` calls.

use resuma::prelude::*;

pub fn install() {
    // `#[middleware] attach_ops_session` registers itself at startup.
}

#[middleware]
async fn attach_ops_session(req: FlowRequest) -> resuma::Result<FlowRequest> {
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
    req.set_extension(
        "user_id",
        serde_json::json!(session.unwrap_or_else(|| "guest".into())),
    );
    req.set_extension("authenticated", serde_json::json!(authenticated));
    if authenticated {
        req.set_extension("roles", serde_json::json!(["admin"]));
    }
    Ok(req)
}
