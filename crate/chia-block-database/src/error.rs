use thiserror::Error;

pub type Result<T> = std::result::Result<T, DatabaseError>;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("SQL execution error: {0}")]
    SqlExecution(String),

    #[error("Invalid database URL: {0}")]
    InvalidUrl(String),

    #[error("Database type not supported: {0}")]
    UnsupportedType(String),

    #[error("Migration file not found: {0}")]
    MigrationFileNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
} 