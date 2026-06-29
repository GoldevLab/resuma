//! SSRF protections for the built-in `fetch` tool.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use url::Url;

use crate::core::{Result, ResumaError};

/// Blocked request headers (credential / routing smuggling).
pub const BLOCKED_HEADERS: &[&str] = &[
    "host",
    "authorization",
    "cookie",
    "proxy-authorization",
    "x-forwarded-for",
    "x-forwarded-host",
    "x-real-ip",
];

/// Validate a URL before outbound fetch.
pub fn validate_fetch_url(url_str: &str) -> Result<Url> {
    let parsed = Url::parse(url_str).map_err(|_| ResumaError::validation("invalid URL"))?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(ResumaError::validation("only http and https URLs are allowed"));
    }
    if parsed.username() != "" || parsed.password().is_some() {
        return Err(ResumaError::validation("URL credentials are not allowed"));
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| ResumaError::validation("URL must have a host"))?;
    if is_blocked_host(host) {
        return Err(ResumaError::Forbidden(
            "fetch to private or reserved hosts is not allowed".into(),
        ));
    }
    if let Some(allowlist) = fetch_allowlist() {
        if !allowlist.iter().any(|allowed| host_matches_allowlist(host, allowed)) {
            return Err(ResumaError::Forbidden(
                "host not in RESUMA_FETCH_ALLOWLIST".into(),
            ));
        }
    }
    Ok(parsed)
}

fn fetch_allowlist() -> Option<Vec<String>> {
    std::env::var("RESUMA_FETCH_ALLOWLIST").ok().map(|raw| {
        raw.split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect()
    })
}

fn host_matches_allowlist(host: &str, allowed: &str) -> bool {
    let host = host.to_lowercase();
    if allowed.starts_with("*.") {
        let suffix = &allowed[1..];
        host.ends_with(suffix) || host == allowed.trim_start_matches('*')
    } else {
        host == allowed
    }
}

fn is_blocked_host(host: &str) -> bool {
    let lower = host.to_lowercase();
    if lower == "localhost"
        || lower.ends_with(".localhost")
        || lower.ends_with(".local")
        || lower == "metadata.google.internal"
        || lower.contains("metadata.google")
    {
        return true;
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        return is_blocked_ip(ip);
    }
    // IPv6 in brackets: [::1]
    if host.starts_with('[') && host.ends_with(']') {
        if let Ok(ip) = host[1..host.len() - 1].parse::<IpAddr>() {
            return is_blocked_ip(ip);
        }
    }
    false
}

fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_ipv4(v4),
        IpAddr::V6(v6) => is_blocked_ipv6(v6),
    }
}

fn is_blocked_ipv4(ip: Ipv4Addr) -> bool {
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.octets()[0] == 0
        || ip == Ipv4Addr::new(169, 254, 169, 254)
        || ip == Ipv4Addr::new(127, 0, 0, 1)
}

fn is_blocked_ipv6(ip: Ipv6Addr) -> bool {
    ip.is_loopback()
        || ip.is_unspecified()
        || (ip.segments()[0] & 0xfe00) == 0xfc00 // unique local
        || (ip.segments()[0] & 0xffc0) == 0xfe80 // link-local
}

pub fn max_fetch_bytes() -> usize {
    std::env::var("RESUMA_FETCH_MAX_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5 * 1024 * 1024)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_localhost() {
        assert!(validate_fetch_url("http://localhost/admin").is_err());
        assert!(validate_fetch_url("http://127.0.0.1/").is_err());
    }

    #[test]
    fn blocks_private_ranges() {
        assert!(validate_fetch_url("http://192.168.1.1/").is_err());
        assert!(validate_fetch_url("http://10.0.0.5/").is_err());
    }

    #[test]
    fn allows_public_https() {
        assert!(validate_fetch_url("https://example.com/path").is_ok());
    }
}
