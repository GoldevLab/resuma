use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResumaError {
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("render error: {0}")]
    Render(String),

    #[error("server action `{0}` not found")]
    UnknownAction(String),

    #[error("island `{0}` not found")]
    UnknownIsland(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("invalid or missing CSRF token")]
    InvalidCsrf,

    #[error("rate limit exceeded")]
    RateLimited,

    #[error("request payload too large")]
    PayloadTooLarge,

    #[error("{0}")]
    Other(String),
}

impl ResumaError {
    /// Suggested HTTP status for API/action responses.
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Unauthorized => 401,
            Self::Forbidden(_) | Self::InvalidCsrf => 403,
            Self::RateLimited => 429,
            Self::UnknownAction(_) | Self::UnknownIsland(_) => 404,
            Self::Serde(_) | Self::PayloadTooLarge => 422,
            Self::Render(_) | Self::Io(_) => 500,
            Self::Other(msg) if msg.eq_ignore_ascii_case("unauthorized") => 401,
            Self::Other(_) => 400,
        }
    }

    /// Safe client-facing message (no internal details in production).
    pub fn client_message(&self, production: bool) -> String {
        if production {
            match self {
                Self::Unauthorized => "Unauthorized".into(),
                Self::Forbidden(_) | Self::InvalidCsrf => "Forbidden".into(),
                Self::RateLimited => "Too many requests".into(),
                Self::UnknownAction(_) => "Not found".into(),
                Self::Serde(_) | Self::PayloadTooLarge => "Invalid request".into(),
                Self::Render(_) | Self::Io(_) => "Internal server error".into(),
                Self::Other(msg) if msg.eq_ignore_ascii_case("unauthorized") => {
                    "Unauthorized".into()
                }
                Self::Other(_) => "Bad request".into(),
                Self::UnknownIsland(_) => "Not found".into(),
            }
        } else {
            self.to_string()
        }
    }
}

pub type Result<T> = std::result::Result<T, ResumaError>;
