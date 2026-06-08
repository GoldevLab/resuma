//! Target-aware helpers for web / mobile-web / PWA variants.

use resuma::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformTarget {
    DesktopWeb,
    MobileWeb,
    Pwa,
}

impl PlatformTarget {
    pub fn label(self) -> &'static str {
        match self {
            Self::DesktopWeb => "desktop-web",
            Self::MobileWeb => "mobile-web",
            Self::Pwa => "pwa",
        }
    }

    pub fn css_class(self) -> &'static str {
        match self {
            Self::DesktopWeb => "platform-desktop",
            Self::MobileWeb => "platform-mobile",
            Self::Pwa => "platform-pwa",
        }
    }
}

/// Resolve target from query `?platform=mobile|pwa` or User-Agent heuristics.
pub fn detect(req: &FlowRequest) -> PlatformTarget {
    match req.query_param("platform").unwrap_or("") {
        "mobile" => return PlatformTarget::MobileWeb,
        "pwa" => return PlatformTarget::Pwa,
        _ => {}
    }
    let ua = req.header("user-agent").unwrap_or("").to_ascii_lowercase();
    if ua.contains("wv") || ua.contains("mobile") {
        PlatformTarget::MobileWeb
    } else if ua.contains("standalone") {
        PlatformTarget::Pwa
    } else {
        PlatformTarget::DesktopWeb
    }
}

/// Pick a value for the active platform; falls back to `default` when a slot is unset.
pub fn platform_select<T: Clone>(
    target: PlatformTarget,
    desktop: T,
    mobile: Option<T>,
    pwa: Option<T>,
    default: T,
) -> T {
    match target {
        PlatformTarget::DesktopWeb => desktop,
        PlatformTarget::MobileWeb => mobile.unwrap_or(default),
        PlatformTarget::Pwa => pwa.unwrap_or(mobile.unwrap_or(default)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn query_overrides_user_agent() {
        let req = FlowRequest {
            query: BTreeMap::from([("platform".into(), "pwa".into())]),
            ..Default::default()
        };
        assert_eq!(detect(&req), PlatformTarget::Pwa);
    }

    #[test]
    fn platform_select_prefers_specific_slot() {
        let out: String = platform_select(
            PlatformTarget::MobileWeb,
            "desktop".to_string(),
            Some("mobile".to_string()),
            Some("pwa".to_string()),
            "fallback".to_string(),
        );
        assert_eq!(out, "mobile");
    }
}
