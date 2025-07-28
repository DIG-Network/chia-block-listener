use crate::database::DatabaseType;
use crate::error::{DatabaseError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{AnyPool, Row};

use std::fs;
use std::path::Path;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub id: String,
    pub name: String,
    pub sql: String,
    pub checksum: String,
}

#[derive(Debug, Clone)]
pub struct AppliedMigration {
    pub id: String,
    pub name: String,
    pub checksum: String,
    pub applied_at: DateTime<Utc>,
}

pub struct MigrationRunner {
    db_type: DatabaseType,
}

impl MigrationRunner {
    pub fn new(db_type: DatabaseType) -> Self {
        Self { db_type }
    }

    /// Run all pending migrations
    pub async fn run_migrations(&self, pool: &AnyPool) -> Result<()> {
        // Ensure migration history table exists
        self.create_migration_history_table(pool).await?;

        // Load available migrations
        let available_migrations = self.load_migrations().await?;
        
        // Get applied migrations
        let applied_migrations = self.get_applied_migrations(pool).await?;
        let applied_ids: std::collections::HashSet<String> = applied_migrations
            .iter()
            .map(|m| m.id.clone())
            .collect();

        // Find pending migrations
        let mut pending_migrations: Vec<_> = available_migrations
            .into_iter()
            .filter(|m| !applied_ids.contains(&m.id))
            .collect();
        
        // Sort by migration ID to ensure consistent order
        pending_migrations.sort_by(|a, b| a.id.cmp(&b.id));

        if pending_migrations.is_empty() {
            info!("No pending migrations");
            return Ok(());
        }

        info!("Running {} pending migrations", pending_migrations.len());

        // Apply each migration in a transaction
        for migration in pending_migrations {
            info!("Applying migration: {} - {}", migration.id, migration.name);
            
            let mut tx = pool.begin().await?;
            
            // Execute the migration SQL
            for statement in self.split_sql_statements(&migration.sql) {
                if !statement.trim().is_empty() {
                    sqlx::query(&statement)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| DatabaseError::Migration(format!(
                            "Failed to execute migration {}: {}",
                            migration.id, e
                        )))?;
                }
            }

            // Record the migration as applied
            self.record_migration(&mut *tx, &migration).await?;
            
            // Commit the transaction
            tx.commit().await?;
            
            info!("Successfully applied migration: {}", migration.id);
        }

        info!("All migrations completed successfully");
        Ok(())
    }

    /// Create the migration history table if it doesn't exist
    async fn create_migration_history_table(&self, pool: &AnyPool) -> Result<()> {
        let sql = match self.db_type {
            DatabaseType::Sqlite => {
                r#"
                CREATE TABLE IF NOT EXISTS migration_history (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    checksum TEXT NOT NULL,
                    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
                )
                "#
            }
            DatabaseType::Postgres => {
                r#"
                CREATE TABLE IF NOT EXISTS migration_history (
                    id VARCHAR(255) PRIMARY KEY,
                    name TEXT NOT NULL,
                    checksum VARCHAR(64) NOT NULL,
                    applied_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
                )
                "#
            }
        };

        sqlx::query(sql).execute(pool).await?;
        Ok(())
    }

    /// Load available migrations from the appropriate directory
    async fn load_migrations(&self) -> Result<Vec<Migration>> {
        let migrations_dir = match self.db_type {
            DatabaseType::Sqlite => "crate/chia-block-database/migrations/sqlite",
            DatabaseType::Postgres => "crate/chia-block-database/migrations/postgres",
        };

        let migrations_path = Path::new(migrations_dir);
        if !migrations_path.exists() {
            return Err(DatabaseError::MigrationFileNotFound(format!(
                "Migrations directory not found: {}",
                migrations_dir
            )));
        }

        let mut migrations = Vec::new();
        let entries = fs::read_dir(migrations_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                    let sql_content = fs::read_to_string(&path)?;
                    let checksum = self.calculate_checksum(&sql_content);
                    
                    // Parse migration ID and name from filename (e.g., "001_core_tables.sql")
                    let parts: Vec<&str> = file_name.splitn(2, '_').collect();
                    if parts.len() >= 2 {
                        let id = parts[0].to_string();
                        let name = parts[1].replace('_', " ");
                        
                        migrations.push(Migration {
                            id,
                            name,
                            sql: sql_content,
                            checksum,
                        });
                    } else {
                        warn!("Skipping migration file with invalid name format: {}", file_name);
                    }
                }
            }
        }

        // Sort by ID to ensure consistent order
        migrations.sort_by(|a, b| a.id.cmp(&b.id));
        
        Ok(migrations)
    }

    /// Get list of applied migrations from the database
    async fn get_applied_migrations(&self, pool: &AnyPool) -> Result<Vec<AppliedMigration>> {
        let rows = sqlx::query("SELECT id, name, checksum, applied_at FROM migration_history ORDER BY id")
            .fetch_all(pool)
            .await?;

        let mut applied_migrations = Vec::new();
        for row in rows {
            let id: String = row.try_get("id")?;
            let name: String = row.try_get("name")?;
            let checksum: String = row.try_get("checksum")?;
            
            let applied_at = match self.db_type {
                DatabaseType::Sqlite => {
                    let datetime_str: String = row.try_get("applied_at")?;
                    // Parse SQLite datetime string format
                    chrono::NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S")
                        .map_err(|e| DatabaseError::SqlExecution(format!("Failed to parse SQLite datetime: {}", e)))?
                        .and_utc()
                }
                DatabaseType::Postgres => {
                    // For PostgreSQL, we need to get the string and parse it
                    let datetime_str: String = row.try_get("applied_at")?;
                    DateTime::parse_from_rfc3339(&datetime_str)
                        .map_err(|e| DatabaseError::SqlExecution(format!("Failed to parse PostgreSQL datetime: {}", e)))?
                        .with_timezone(&Utc)
                }
            };

            applied_migrations.push(AppliedMigration {
                id,
                name,
                checksum,
                applied_at,
            });
        }

        Ok(applied_migrations)
    }

    /// Record a migration as applied
    async fn record_migration<'a>(&self, executor: impl sqlx::Executor<'a, Database = sqlx::Any>, migration: &Migration) -> Result<()> {
        let sql = match self.db_type {
            DatabaseType::Sqlite => {
                "INSERT INTO migration_history (id, name, checksum) VALUES (?1, ?2, ?3)"
            }
            DatabaseType::Postgres => {
                "INSERT INTO migration_history (id, name, checksum) VALUES ($1, $2, $3)"
            }
        };

        sqlx::query(sql)
            .bind(&migration.id)
            .bind(&migration.name)
            .bind(&migration.checksum)
            .execute(executor)
            .await?;

        Ok(())
    }

    /// Calculate checksum for migration content
    fn calculate_checksum(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Split SQL content into individual statements
    fn split_sql_statements(&self, sql: &str) -> Vec<String> {
        // Simple SQL statement splitter - splits on semicolons not inside quotes
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut chars = sql.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '\'' if !in_double_quote => {
                    in_single_quote = !in_single_quote;
                    current_statement.push(ch);
                }
                '"' if !in_single_quote => {
                    in_double_quote = !in_double_quote;
                    current_statement.push(ch);
                }
                ';' if !in_single_quote && !in_double_quote => {
                    current_statement.push(ch);
                    let trimmed = current_statement.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with("--") {
                        statements.push(current_statement.trim().to_string());
                    }
                    current_statement.clear();
                }
                _ => {
                    current_statement.push(ch);
                }
            }
        }

        // Add the last statement if it doesn't end with semicolon
        let trimmed = current_statement.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("--") {
            statements.push(trimmed.to_string());
        }

        statements
    }
} 