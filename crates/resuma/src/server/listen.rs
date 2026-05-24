//! Shared listen address resolution for `ResumaApp` and `FlowApp`.

use std::net::SocketAddr;

/// Bind address from `RESUMA_ADDR`, or `HOST` + `PORT` (Fly.io / Docker), else `127.0.0.1:3000`.
pub fn listen_addr_from_env() -> SocketAddr {
    if let Ok(raw) = std::env::var("RESUMA_ADDR") {
        if let Ok(addr) = raw.parse() {
            return addr;
        }
    }

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    format!("{host}:{port}")
        .parse()
        .unwrap_or_else(|_| ([127, 0, 0, 1], 3000).into())
}
