use thiserror::Error;

/// Typed engine errors. Platform bindings map these onto native error types.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;
