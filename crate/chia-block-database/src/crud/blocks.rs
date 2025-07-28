use crate::database::DatabaseType;
use crate::error::{DatabaseError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{AnyPool, Row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub height: i64,
    pub weight: i64,
    pub header_hash: Vec<u8>,
    pub timestamp: DateTime<Utc>,
}

pub struct BlocksCrud {
    db_type: DatabaseType,
}

impl BlocksCrud {
    pub fn new(db_type: DatabaseType) -> Self {
        Self { db_type }
    }

    /// Create a new block
    pub async fn create(&self, pool: &AnyPool, block: &Block) -> Result<()> {
        let timestamp_str = match self.db_type {
            DatabaseType::Postgres => block.timestamp.to_rfc3339(),
            DatabaseType::Sqlite => block.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        match self.db_type {
            DatabaseType::Postgres => {
                sqlx::query(
                    "INSERT INTO blocks (height, weight, header_hash, timestamp) VALUES ($1, $2, $3, $4)
                     ON CONFLICT (height) DO UPDATE SET 
                     weight = EXCLUDED.weight,
                     header_hash = EXCLUDED.header_hash,
                     timestamp = EXCLUDED.timestamp"
                )
                .bind(block.height)
                .bind(block.weight)
                .bind(&block.header_hash)
                .bind(timestamp_str)
                .execute(pool)
                .await?;
            }
            DatabaseType::Sqlite => {
                sqlx::query(
                    "INSERT OR REPLACE INTO blocks (height, weight, header_hash, timestamp) VALUES (?1, ?2, ?3, ?4)"
                )
                .bind(block.height)
                .bind(block.weight)
                .bind(&block.header_hash)
                .bind(timestamp_str)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }

    /// Get a block by height
    pub async fn get_by_height(&self, pool: &AnyPool, height: i64) -> Result<Option<Block>> {
        let query = match self.db_type {
            DatabaseType::Postgres => "SELECT height, weight, header_hash, timestamp FROM blocks WHERE height = $1",
            DatabaseType::Sqlite => "SELECT height, weight, header_hash, timestamp FROM blocks WHERE height = ?1",
        };

        let row = sqlx::query(query)
            .bind(height)
            .fetch_optional(pool)
            .await?;

        if let Some(row) = row {
            let timestamp_str: String = row.try_get("timestamp")?;
            let timestamp = match self.db_type {
                DatabaseType::Postgres => {
                    DateTime::parse_from_rfc3339(&timestamp_str)
                        .map_err(|e| DatabaseError::SqlExecution(format!("Failed to parse PostgreSQL timestamp: {}", e)))?
                        .with_timezone(&Utc)
                }
                DatabaseType::Sqlite => {
                    chrono::NaiveDateTime::parse_from_str(&timestamp_str, "%Y-%m-%d %H:%M:%S")
                        .map_err(|e| DatabaseError::SqlExecution(format!("Failed to parse SQLite timestamp: {}", e)))?
                        .and_utc()
                }
            };

            Ok(Some(Block {
                height: row.try_get("height")?,
                weight: row.try_get("weight")?,
                header_hash: row.try_get("header_hash")?,
                timestamp,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get blocks in a range with pagination
    pub async fn get_range(&self, pool: &AnyPool, start_height: i64, end_height: i64, page: i32, page_size: i32) -> Result<Vec<Block>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                "SELECT height, weight, header_hash, timestamp FROM blocks 
                 WHERE height >= $1 AND height <= $2 
                 ORDER BY height ASC 
                 LIMIT $3 OFFSET $4"
            }
            DatabaseType::Sqlite => {
                "SELECT height, weight, header_hash, timestamp FROM blocks 
                 WHERE height >= ?1 AND height <= ?2 
                 ORDER BY height ASC 
                 LIMIT ?3 OFFSET ?4"
            }
        };

        let rows = sqlx::query(query)
            .bind(start_height)
            .bind(end_height)
            .bind(page_size)
            .bind((page - 1) * page_size)
            .fetch_all(pool)
            .await?;

        let mut blocks = Vec::new();
        for row in rows {
            let timestamp_str: String = row.try_get("timestamp")?;
            let timestamp = match self.db_type {
                DatabaseType::Postgres => {
                    DateTime::parse_from_rfc3339(&timestamp_str)
                        .map_err(|e| DatabaseError::SqlExecution(format!("Failed to parse PostgreSQL timestamp: {}", e)))?
                        .with_timezone(&Utc)
                }
                DatabaseType::Sqlite => {
                    chrono::NaiveDateTime::parse_from_str(&timestamp_str, "%Y-%m-%d %H:%M:%S")
                        .map_err(|e| DatabaseError::SqlExecution(format!("Failed to parse SQLite timestamp: {}", e)))?
                        .and_utc()
                }
            };

            blocks.push(Block {
                height: row.try_get("height")?,
                weight: row.try_get("weight")?,
                header_hash: row.try_get("header_hash")?,
                timestamp,
            });
        }
        Ok(blocks)
    }

    /// Update a block (uses upsert)
    pub async fn update(&self, pool: &AnyPool, block: &Block) -> Result<()> {
        self.create(pool, block).await
    }

    /// Delete a block
    pub async fn delete(&self, pool: &AnyPool, height: i64) -> Result<()> {
        let query = match self.db_type {
            DatabaseType::Postgres => "DELETE FROM blocks WHERE height = $1",
            DatabaseType::Sqlite => "DELETE FROM blocks WHERE height = ?1",
        };

        sqlx::query(query)
            .bind(height)
            .execute(pool)
            .await?;
        Ok(())
    }
} 