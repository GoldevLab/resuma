//! Shared listen address resolution for `ResumaApp` and `FlowApp`.

use std::net::SocketAddr;

/// Max ports to try when the preferred port is taken (local dev only).
const AUTO_PORT_ATTEMPTS: u16 = 50;

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

/// Platform injects `PORT` — must bind exactly that port (no auto-scan).
fn platform_port_fixed() -> bool {
    std::env::var("PORT").is_ok()
}

/// Pick the first bindable address starting at `preferred`.
///
/// When `PORT` is set (Fly, Docker, etc.), only `preferred` is tried.
/// Otherwise, if the port is in use, tries `port + 1`, `port + 2`, …
pub fn resolve_listen_addr(preferred: SocketAddr) -> std::io::Result<SocketAddr> {
    use std::io::ErrorKind;
    use std::net::TcpListener;

    if platform_port_fixed() {
        TcpListener::bind(preferred)?;
        return Ok(preferred);
    }

    let ip = preferred.ip();
    let start = preferred.port();
    // Port 0 = OS picks an ephemeral port (single attempt).
    if start == 0 {
        let listener = TcpListener::bind(preferred)?;
        return listener.local_addr();
    }
    for offset in 0..AUTO_PORT_ATTEMPTS {
        let port = start.saturating_add(offset);
        let addr = SocketAddr::new(ip, port);
        match TcpListener::bind(addr) {
            Ok(_) => return Ok(addr),
            Err(e) if e.kind() == ErrorKind::AddrInUse => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(
        ErrorKind::AddrInUse,
        format!(
            "no free port in range {}–{}",
            start,
            start.saturating_add(AUTO_PORT_ATTEMPTS - 1)
        ),
    ))
}

/// Bind a tokio listener, with the same port-scan behavior as [`resolve_listen_addr`].
pub async fn bind_listener(
    preferred: SocketAddr,
) -> std::io::Result<(tokio::net::TcpListener, SocketAddr)> {
    use std::io::ErrorKind;

    if platform_port_fixed() {
        let listener = tokio::net::TcpListener::bind(preferred).await?;
        let bound = listener.local_addr()?;
        return Ok((listener, bound));
    }

    let ip = preferred.ip();
    let start = preferred.port();
    if start == 0 {
        let listener = tokio::net::TcpListener::bind(preferred).await?;
        let bound = listener.local_addr()?;
        return Ok((listener, bound));
    }
    for offset in 0..AUTO_PORT_ATTEMPTS {
        let port = start.saturating_add(offset);
        let addr = SocketAddr::new(ip, port);
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                let bound = listener.local_addr()?;
                if offset > 0 {
                    eprintln!(
                        "[resuma] port {} in use, listening on http://{}",
                        start, bound
                    );
                }
                return Ok((listener, bound));
            }
            Err(e) if e.kind() == ErrorKind::AddrInUse => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(
        ErrorKind::AddrInUse,
        format!(
            "no free port in range {}–{}",
            start,
            start.saturating_add(AUTO_PORT_ATTEMPTS - 1)
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_listen_addr_skips_taken_port() {
        use std::net::TcpListener;
        let preferred: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let holder = TcpListener::bind(preferred).unwrap();
        let taken = holder.local_addr().unwrap();
        let resolved = resolve_listen_addr(taken).unwrap();
        assert_eq!(resolved.ip(), taken.ip());
        assert_eq!(resolved.port(), taken.port() + 1);
    }
}
