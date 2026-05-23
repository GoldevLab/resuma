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

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, ResumaError>;
