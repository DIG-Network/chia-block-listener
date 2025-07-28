use serde::{Deserialize, Serialize};

/// Database configuration for connecting to either Postgres or SQLite
#[derive(Debug, Clone)]
pub enum DatabaseConfig {
    Postgres { connection_string: String },
    Sqlite { file_path: String },
}

/// Type of database being used
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseType {
    Postgres,
    Sqlite,
} 