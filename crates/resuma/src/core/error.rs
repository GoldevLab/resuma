use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResumaError {
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("render error: {0}")]
    Render(String),

    #[error("server action `{0}` not found")]
    UnknownAction(String),

    #[error("loader `{0}` not found")]
    UnknownLoader(String),

    #[error("submit `{0}` not found")]
    UnknownSubmit(String),

    #[error("worker `{0}` not found")]
    UnknownWorker(String),

    #[error("tool `{0}` not found")]
    UnknownTool(String),

    #[error("graph `{0}` not found")]
    UnknownGraph(String),

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

    #[error("validation error: {0}")]
    Validation(String),

    #[error("execution cancelled")]
    Cancelled,

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
            Self::Cancelled => 409,
            Self::UnknownAction(_)
            | Self::UnknownLoader(_)
            | Self::UnknownSubmit(_)
            | Self::UnknownWorker(_)
            | Self::UnknownTool(_)
            | Self::UnknownGraph(_)
            | Self::UnknownIsland(_) => 404,
            Self::Serde(_) | Self::PayloadTooLarge | Self::Validation(_) => 422,
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
                Self::Cancelled => "Execution cancelled".into(),
                Self::UnknownAction(_)
                | Self::UnknownLoader(_)
                | Self::UnknownSubmit(_)
                | Self::UnknownWorker(_)
                | Self::UnknownTool(_)
                | Self::UnknownGraph(_) => "Not found".into(),
                Self::Serde(_) | Self::PayloadTooLarge | Self::Validation(_) => {
                    "Invalid request".into()
                }
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

    /// Convenience constructor for typed validation failures.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }
}

pub type Result<T> = std::result::Result<T, ResumaError>;
