use crate::error::{DatabaseError, Result};
use crate::migrations::MigrationRunner;
use crate::namespaces::{BlockchainNamespace, AssetsNamespace, SystemNamespace};
use sqlx::{AnyPool, Row, Column};
use std::str::FromStr;
use tracing::{info, warn};

#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseType {
    Sqlite,
    Postgres,
}

impl FromStr for DatabaseType {
    type Err = DatabaseError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "sqlite" | "sqlite3" => Ok(DatabaseType::Sqlite),
            "postgres" | "postgresql" => Ok(DatabaseType::Postgres),
            _ => Err(DatabaseError::UnsupportedType(s.to_string())),
        }
    }
}

pub struct ChiaBlockDatabase {
    pool: AnyPool,
    db_type: DatabaseType,
    migration_runner: MigrationRunner,
    blockchain: BlockchainNamespace,
    assets: AssetsNamespace,
    system: SystemNamespace,
}

impl ChiaBlockDatabase {
    /// Create a new database instance with automatic type detection from URL
    pub async fn new(database_url: &str) -> Result<Self> {
        let db_type = Self::detect_database_type(database_url)?;
        Self::new_with_type(database_url, db_type).await
    }

    /// Create a new database instance with explicit type
    pub async fn new_with_type(database_url: &str, db_type: DatabaseType) -> Result<Self> {
        info!("Connecting to {:?} database...", db_type);
        
        let pool = sqlx::AnyPool::connect(database_url)
            .await
            .map_err(|e| DatabaseError::Connection(e))?;

        info!("Connected to database successfully");

        let migration_runner = MigrationRunner::new(db_type.clone());

        let mut database = Self {
            pool,
            blockchain: BlockchainNamespace::new(db_type.clone()),
            assets: AssetsNamespace::new(db_type.clone()),
            system: SystemNamespace::new(db_type.clone()),
            db_type,
            migration_runner,
        };

        // Run migrations
        database.migrate().await?;

        Ok(database)
    }

    /// Run all pending migrations
    pub async fn migrate(&mut self) -> Result<()> {
        info!("Running database migrations for {:?}", self.db_type);
        self.migration_runner.run_migrations(&self.pool).await
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &AnyPool {
        &self.pool
    }

    /// Get the database type
    pub fn database_type(&self) -> &DatabaseType {
        &self.db_type
    }

    /// Close the database connection
    pub async fn close(self) {
        self.pool.close().await;
        info!("Database connection closed");
    }

    /// Access to blockchain operations (blocks, coins, spends)
    pub fn blockchain(&self) -> &BlockchainNamespace {
        &self.blockchain
    }

    /// Access to asset operations (CATs, NFTs)
    pub fn assets(&self) -> &AssetsNamespace {
        &self.assets
    }

    /// Access to system operations (sync status, preferences)
    pub fn system(&self) -> &SystemNamespace {
        &self.system
    }

    /// Execute a raw SQL query with optional parameters
    pub async fn execute_raw_query(&self, query: &str, params: Option<Vec<serde_json::Value>>) -> Result<String> {
        info!("Executing raw query: {}", query);
        
        let mut sqlx_query = sqlx::query(query);
        
        // Bind parameters if provided
        if let Some(params) = params {
            for param in params {
                match param {
                    serde_json::Value::String(s) => {
                        sqlx_query = sqlx_query.bind(s);
                    }
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            sqlx_query = sqlx_query.bind(i);
                        } else if let Some(f) = n.as_f64() {
                            sqlx_query = sqlx_query.bind(f);
                        } else {
                            return Err(DatabaseError::InvalidInput("Invalid numeric parameter".to_string()));
                        }
                    }
                    serde_json::Value::Bool(b) => {
                        sqlx_query = sqlx_query.bind(b);
                    }
                    serde_json::Value::Null => {
                        sqlx_query = sqlx_query.bind(Option::<String>::None);
                    }
                    _ => {
                        // For complex types, bind as JSON string
                        sqlx_query = sqlx_query.bind(param.to_string());
                    }
                }
            }
        }
        
        // Execute query and collect results
        let rows = sqlx_query.fetch_all(&self.pool).await?;
        
        // Convert rows to JSON
        let mut results = Vec::new();
        for row in rows {
            let mut row_map = serde_json::Map::new();
            
            // Get column names and values
            let columns = row.columns();
            for (i, column) in columns.iter().enumerate() {
                let column_name = column.name();
                
                // Try to extract value based on type
                let value = if let Ok(val) = row.try_get::<String, _>(i) {
                    serde_json::Value::String(val)
                } else if let Ok(val) = row.try_get::<i64, _>(i) {
                    serde_json::Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.try_get::<f64, _>(i) {
                    serde_json::Number::from_f64(val)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null)
                } else if let Ok(val) = row.try_get::<bool, _>(i) {
                    serde_json::Value::Bool(val)
                } else if let Ok(val) = row.try_get::<Vec<u8>, _>(i) {
                    // Convert binary data to hex string
                    serde_json::Value::String(hex::encode(val))
                } else {
                    serde_json::Value::Null
                };
                
                row_map.insert(column_name.to_string(), value);
            }
            
            results.push(serde_json::Value::Object(row_map));
        }
        
        // Return results as JSON string
        Ok(serde_json::to_string(&results)?)
    }

    /// Detect database type from URL
    fn detect_database_type(url: &str) -> Result<DatabaseType> {
        if url.starts_with("sqlite:") || url.starts_with("sqlite3:") || url.ends_with(".db") || url.ends_with(".sqlite") {
            Ok(DatabaseType::Sqlite)
        } else if url.starts_with("postgres:") || url.starts_with("postgresql:") {
            Ok(DatabaseType::Postgres)
        } else {
            Err(DatabaseError::InvalidUrl(format!(
                "Cannot detect database type from URL: {}",
                url
            )))
        }
    }

    /// Configure database URL with appropriate options
    fn configure_database_url(url: &str, db_type: &DatabaseType) -> Result<String> {
        match db_type {
            DatabaseType::Sqlite => {
                // Ensure SQLite URL has proper format and options
                if url.starts_with("sqlite:") {
                    // Add WAL mode and other SQLite optimizations to URL if not present
                    if url.contains('?') {
                        Ok(format!("{}&journal_mode=WAL&synchronous=NORMAL&cache_size=64000&foreign_keys=ON", url))
                    } else {
                        Ok(format!("{}?journal_mode=WAL&synchronous=NORMAL&cache_size=64000&foreign_keys=ON", url))
                    }
                } else {
                    // Assume it's a file path
                    Ok(format!("sqlite:{}?journal_mode=WAL&synchronous=NORMAL&cache_size=64000&foreign_keys=ON", url))
                }
            }
            DatabaseType::Postgres => {
                // PostgreSQL URL should be used as-is
                Ok(url.to_string())
            }
        }
    }

    /// Enable WAL mode for SQLite
    async fn enable_wal_mode(pool: &AnyPool) -> Result<()> {
        info!("Enabling WAL mode for SQLite");
        
        // Check current journal mode
        let row = sqlx::query("PRAGMA journal_mode")
            .fetch_one(pool)
            .await?;
        
        let current_mode: String = row.try_get(0)
            .map_err(|e| DatabaseError::SqlExecution(format!("Failed to get journal mode: {}", e)))?;
        
        if current_mode.to_uppercase() != "WAL" {
            warn!("Journal mode is '{}', setting to WAL", current_mode);
            
            sqlx::query("PRAGMA journal_mode=WAL")
                .execute(pool)
                .await?;
                
            info!("WAL mode enabled");
        } else {
            info!("WAL mode already enabled");
        }

        // Set other SQLite optimizations
        sqlx::query("PRAGMA synchronous=NORMAL").execute(pool).await?;
        sqlx::query("PRAGMA cache_size=64000").execute(pool).await?;
        sqlx::query("PRAGMA foreign_keys=ON").execute(pool).await?;
        
        Ok(())
    }
}

/* 
Example Usage:

```rust
// Initialize database
let db = ChiaBlockDatabase::new("sqlite:blockchain.db").await?;

// Access blockchain operations
let block = db.blockchain().blocks().get_by_height(&db.pool(), 1000).await?;
let coins = db.blockchain().coins().get_by_puzzle_hash(&db.pool(), &puzzle_hash, 1, 100).await?;

// Spends automatically handle puzzle/solution abstraction
let spend = Spend {
    coin_id: "abc123".to_string(),
    puzzle_hash: Some(puzzle_hash),
    puzzle_reveal: Some(puzzle_reveal),
    solution_hash: None, // Will be calculated
    solution: Some(solution_data),
    spent_block: 1000,
};
db.blockchain().spends().create(&db.pool(), &spend).await?;

// Access asset operations
let cat_balances = db.assets().cat_functions().get_balances_by_owner("puzzle_hash", 1, 100).await?;
let nfts = db.assets().nft_functions().get_by_owner("puzzle_hash", None, 1, 100).await?;

        // Analytics functionality has been moved to GraphQL API

// Access system functions
let sync_status = db.system().sync_status().get_current().await?;
let preferences = db.system().preferences().get_all().await?;
```
*/ 