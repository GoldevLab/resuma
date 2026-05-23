//! Embeds the compiled JS runtime into the binary so apps don't need a
//! separate static asset server.
//!
//! When the runtime crate is rebuilt (`pnpm --filter @resuma/runtime build`),
//! the resulting `runtime.js` is copied into this directory. Until that
//! happens we ship a tiny inline fallback that warns the developer.

pub const RUNTIME_JS: &str = include_str!("../assets/runtime.js");
