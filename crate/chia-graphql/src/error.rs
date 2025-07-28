use thiserror::Error;

/// GraphQL-specific error type
#[derive(Error, Debug, Clone)]
pub enum GraphQLError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for GraphQLError {
    fn from(err: sqlx::Error) -> Self {
        GraphQLError::Database(err.to_string())
    }
}

/// Result type for GraphQL operations
pub type GraphQLResult<T> = Result<T, GraphQLError>; 