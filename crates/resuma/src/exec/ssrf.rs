//! SSRF protections for the built-in `fetch` tool and outbound webhooks.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

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

/// Validate a URL before outbound fetch (hostname / scheme checks, no DNS yet).
pub fn validate_fetch_url(url_str: &str) -> Result<Url> {
    let parsed = Url::parse(url_str).map_err(|_| ResumaError::validation("invalid URL"))?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(ResumaError::validation(
            "only http and https URLs are allowed",
        ));
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
        if !allowlist
            .iter()
            .any(|allowed| host_matches_allowlist(host, allowed))
        {
            return Err(ResumaError::Forbidden(
                "host not in RESUMA_FETCH_ALLOWLIST".into(),
            ));
        }
    }
    Ok(parsed)
}

/// Resolve DNS and validate all returned addresses (blocks rebinding to private IPs).
pub async fn validate_fetch_url_resolved(url_str: &str) -> Result<(Url, IpAddr)> {
    let url = validate_fetch_url(url_str)?;
    let host = url
        .host_str()
        .ok_or_else(|| ResumaError::validation("URL must have a host"))?;

    if let Some(ip) = parse_host_as_ip(host) {
        if is_blocked_ip(ip) {
            return Err(ResumaError::Forbidden(
                "fetch to private or reserved hosts is not allowed".into(),
            ));
        }
        return Ok((url, ip));
    }

    let port = url
        .port_or_known_default()
        .ok_or_else(|| ResumaError::validation("URL must have a port"))?;
    let mut addrs = tokio::net::lookup_host((host, port))
        .await
        .map_err(|_| ResumaError::validation("DNS lookup failed"))?;

    let first = addrs
        .next()
        .ok_or_else(|| ResumaError::validation("DNS lookup returned no addresses"))?;

    for addr in addrs {
        if is_blocked_ip(addr.ip()) {
            return Err(ResumaError::Forbidden(
                "fetch host resolves to a private or reserved address".into(),
            ));
        }
    }

    let ip = first.ip();
    if is_blocked_ip(ip) {
        return Err(ResumaError::Forbidden(
            "fetch to private or reserved hosts is not allowed".into(),
        ));
    }

    Ok((url, ip))
}

/// Build an HTTP client pinned to the IP validated at DNS resolution time.
pub fn pinned_fetch_client(url: &Url, ip: IpAddr) -> Result<reqwest::Client> {
    use std::time::Duration;
    let host = url
        .host_str()
        .ok_or_else(|| ResumaError::validation("URL must have a host"))?;
    let port = url
        .port_or_known_default()
        .ok_or_else(|| ResumaError::validation("URL must have a port"))?;
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(60))
        .user_agent("resuma-exec/1.0")
        .resolve(host, SocketAddr::new(ip, port))
        .build()
        .map_err(|e| ResumaError::Other(format!("HTTP client build failed: {e}")))
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
        || lower.ends_with(".internal")
        || lower == "metadata.google.internal"
        || lower.contains("metadata.google")
    {
        return true;
    }
    if let Some(ip) = parse_host_as_ip(host) {
        return is_blocked_ip(ip);
    }
    false
}

/// Parse a host string as an IP using multiple encodings (decimal, hex, shorthand, mapped IPv6).
pub fn parse_host_as_ip(host: &str) -> Option<IpAddr> {
    let host = host.trim();
    if host.is_empty() {
        return None;
    }

    // Bracketed IPv6: [::1]
    if host.starts_with('[') && host.ends_with(']') {
        return host[1..host.len() - 1].parse().ok();
    }

    // Standard forms
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Some(ip);
    }

    // Decimal IPv4: 2130706433
    if host.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(n) = host.parse::<u32>() {
            return Some(IpAddr::V4(Ipv4Addr::from(n)));
        }
    }

    // Hex IPv4: 0x7f000001 or 7f.0.0.1 style
    if host.starts_with("0x") || host.starts_with("0X") {
        if let Ok(n) = u32::from_str_radix(&host[2..], 16) {
            return Some(IpAddr::V4(Ipv4Addr::from(n)));
        }
    }

    // Dotted hex octets: 0x7f.0x00.0x00.0x01
    if host.contains('.')
        && host
            .split('.')
            .all(|p| p.starts_with("0x") || p.starts_with("0X"))
    {
        let octets: Option<Vec<u8>> = host
            .split('.')
            .map(|p| u8::from_str_radix(&p[2..], 16).ok())
            .collect();
        if let Some(octets) = octets {
            if octets.len() == 4 {
                return Some(IpAddr::V4(Ipv4Addr::new(
                    octets[0], octets[1], octets[2], octets[3],
                )));
            }
        }
    }

    // Shorthand IPv4: 127.1 → 127.0.0.1 (Rust accepts some forms)
    if host.contains('.') && !host.chars().any(|c| c.is_ascii_alphabetic()) {
        return host.parse::<Ipv4Addr>().ok().map(IpAddr::V4);
    }

    None
}

fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_ipv4(v4),
        IpAddr::V6(v6) => is_blocked_ipv6(v6),
    }
}

fn is_blocked_ipv4(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || o[0] == 0
        || ip == Ipv4Addr::new(169, 254, 169, 254)
        || ip == Ipv4Addr::new(127, 0, 0, 1)
        // Carrier-grade NAT (100.64.0.0/10) — internal in many cloud/VPC setups.
        || (o[0] == 100 && (64..=127).contains(&o[1]))
        // IETF protocol assignments (192.0.0.0/24), incl. 192.0.0.0/29 NAT64.
        || (o[0] == 192 && o[1] == 0 && o[2] == 0)
        // Benchmarking range 198.18.0.0/15.
        || (o[0] == 198 && (o[1] == 18 || o[1] == 19))
        // TEST-NET ranges sometimes used to reach internal proxies.
        || (o[0] == 192 && o[1] == 0 && o[2] == 2) // 192.0.2.0/24
        || (o[0] == 198 && o[1] == 51 && o[2] == 100) // 198.51.100.0/24
        || (o[0] == 203 && o[1] == 0 && o[2] == 113) // 203.0.113.0/24
        // Multicast + reserved/future use (224.0.0.0/3 covers 224–255).
        || o[0] >= 224
}

fn is_blocked_ipv6(ip: Ipv6Addr) -> bool {
    if ip.is_loopback() || ip.is_unspecified() {
        return true;
    }
    // IPv4-mapped: ::ffff:127.0.0.1
    if let Some(v4) = ip.to_ipv4_mapped() {
        return is_blocked_ipv4(v4);
    }
    let seg = ip.segments();
    // IPv4-compatible (deprecated ::a.b.c.d): first six segments all zero —
    // apply the IPv4 rules to the embedded address.
    if seg[..6].iter().all(|&s| s == 0) {
        let v4 = Ipv4Addr::new(
            (seg[6] >> 8) as u8,
            (seg[6] & 0xff) as u8,
            (seg[7] >> 8) as u8,
            (seg[7] & 0xff) as u8,
        );
        return is_blocked_ipv4(v4);
    }
    (seg[0] & 0xff00) == 0xff00 // multicast ff00::/8
        || (seg[0] & 0xfe00) == 0xfc00 // unique local
        || (seg[0] & 0xffc0) == 0xfe80 // link-local
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
    fn blocks_cgnat_and_reserved_ranges() {
        assert!(is_blocked_ipv4("100.64.0.1".parse().unwrap()), "CGNAT");
        assert!(
            is_blocked_ipv4("100.127.255.254".parse().unwrap()),
            "CGNAT edge"
        );
        assert!(
            is_blocked_ipv4("198.18.0.1".parse().unwrap()),
            "benchmark range"
        );
        assert!(
            is_blocked_ipv4("192.0.0.1".parse().unwrap()),
            "IETF assignments"
        );
        assert!(is_blocked_ipv4("224.0.0.1".parse().unwrap()), "multicast");
        // Public CGNAT-adjacent addresses stay allowed.
        assert!(!is_blocked_ipv4("100.63.255.255".parse().unwrap()));
        assert!(!is_blocked_ipv4("101.0.0.1".parse().unwrap()));
    }

    #[test]
    fn allows_public_https() {
        assert!(validate_fetch_url("https://example.com/path").is_ok());
    }

    #[test]
    fn blocks_decimal_ip_encoding() {
        assert!(is_blocked_host("2130706433"));
        assert!(parse_host_as_ip("2130706433").is_some());
    }

    #[test]
    fn blocks_hex_ip_encoding() {
        assert!(is_blocked_host("0x7f000001"));
    }

    #[test]
    fn blocks_ipv4_mapped_loopback() {
        assert!(is_blocked_host("[::ffff:127.0.0.1]"));
    }

    #[test]
    fn blocks_ipv4_compatible_and_ipv6_multicast() {
        assert!(is_blocked_ipv6("::127.0.0.1".parse().unwrap()));
        assert!(is_blocked_ipv6("::10.0.0.5".parse().unwrap()));
        assert!(is_blocked_ipv6("ff02::1".parse().unwrap()));
        // Public global unicast stays allowed.
        assert!(!is_blocked_ipv6("2606:4700:4700::1111".parse().unwrap()));
    }

    #[test]
    fn rejects_blocked_fetch_headers() {
        assert!(BLOCKED_HEADERS.contains(&"host"));
    }
}
