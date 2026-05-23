//! Combine multiple event-handler JS bodies into one handler.

/// Merge JS handler bodies (for `onClick={ combine_js(&[a, b]) }` patterns).
pub fn combine_js(handlers: &[&str]) -> String {
    let body = handlers.join("\n");
    format!("(async (event, state, __resuma) => {{ {body} }})")
}
