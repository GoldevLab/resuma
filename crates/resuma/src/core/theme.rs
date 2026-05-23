//! Theme management via context.

use serde::{Deserialize, Serialize};

use super::app_context::{provide_context, use_context, ContextId};

/// Application theme tokens (cookbook pattern).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub mode: String,
    pub primary: String,
    pub background: String,
    pub foreground: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            mode: "dark".into(),
            primary: "#6366f1".into(),
            background: "#0b1020".into(),
            foreground: "#e6e8ee".into(),
        }
    }
}

static THEME_CTX: ContextId<Theme> = ContextId::new();

/// Provide theme for descendant components (serialized in resumability payload).
pub fn provide_theme(theme: Theme) {
    provide_context(&THEME_CTX, theme);
}

/// Read the active theme.
pub fn use_theme() -> Theme {
    use_context(&THEME_CTX)
}

/// CSS variables string for inline `<style>` or `class`.
pub fn theme_css_vars(theme: &Theme) -> String {
    format!(
        "--resuma-primary:{p};--resuma-bg:{bg};--resuma-fg:{fg};",
        p = theme.primary,
        bg = theme.background,
        fg = theme.foreground,
    )
}
