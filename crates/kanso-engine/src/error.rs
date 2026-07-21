use thiserror::Error;

/// Typed engine errors. Platform bindings map these onto native error types.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("decode error: {0}")]
    Decode(String),

    #[error("transport error: {0}")]
    Transport(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;
