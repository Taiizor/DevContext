use thiserror::Error;

/// Core error types for DevContext operations.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Data integrity error: {0}")]
    Integrity(String),

    #[error("Concurrency error: {0}")]
    Concurrency(String),

    #[error("Project error: {0}")]
    Project(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
