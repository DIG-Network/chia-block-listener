use async_graphql::*;
use sqlx::{AnyPool, Row};
use std::sync::Arc;

use crate::database::DatabaseType;
use crate::error::{GraphQLError, GraphQLResult};
use crate::schema::types::*;

/// Core blockchain queries
pub struct CoreQueries {
    pool: Arc<AnyPool>,
    db_type: DatabaseType,
}

impl CoreQueries {
    pub fn new(pool: Arc<AnyPool>, db_type: DatabaseType) -> Self {
        Self { pool, db_type }
    }
}

#[Object]
impl CoreQueries {
    /// Get coin by ID
    async fn coin_by_id(&self, coin_id: String) -> GraphQLResult<Option<CoinInfo>> {
        let coin_id_bytes = hex::decode(&coin_id)
            .map_err(|_| GraphQLError::InvalidInput("Invalid coin ID hex".to_string()))?;

        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                    encode(c.coin_id, 'hex') as coin_id,
                    encode(c.parent_coin_id, 'hex') as parent_coin_id,
                    encode(c.puzzle_hash, 'hex') as puzzle_hash,
                    c.amount,
                    c.created_block,
                    s.spent_block
                FROM coins c
                LEFT JOIN spends s ON c.coin_id = s.coin_id
                WHERE c.coin_id = $1
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                    hex(c.coin_id) as coin_id,
                    hex(c.parent_coin_id) as parent_coin_id,
                    hex(c.puzzle_hash) as puzzle_hash,
                    c.amount,
                    c.created_block,
                    s.spent_block
                FROM coins c
                LEFT JOIN spends s ON c.coin_id = s.coin_id
                WHERE c.coin_id = ?
                "#
            }
        };

        let row = sqlx::query(query)
            .bind(&coin_id_bytes)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(row.map(|r| CoinInfo {
            coin_id: r.get("coin_id"),
            parent_coin_id: r.get("parent_coin_id"),
            puzzle_hash: r.get("puzzle_hash"),
            amount: r.get::<i64, _>("amount") as u64,
            created_block: r.get("created_block"),
            spent_block: r.get("spent_block"),
        }))
    }

    /// Get coins by puzzle hash
    async fn coins_by_puzzle_hash(
        &self,
        puzzle_hash_hex: String,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<CoinInfo>> {
        let puzzle_hash = hex::decode(&puzzle_hash_hex)
            .map_err(|_| GraphQLError::InvalidInput("Invalid puzzle hash hex".to_string()))?;

        let offset = (page - 1) * page_size;

        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                    encode(c.coin_id, 'hex') as coin_id,
                    encode(c.parent_coin_id, 'hex') as parent_coin_id,
                    encode(c.puzzle_hash, 'hex') as puzzle_hash,
                    c.amount,
                    c.created_block,
                    s.spent_block
                FROM coins c
                LEFT JOIN spends s ON c.coin_id = s.coin_id
                WHERE c.puzzle_hash = $1
                ORDER BY c.created_block DESC
                LIMIT $2 OFFSET $3
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                    hex(c.coin_id) as coin_id,
                    hex(c.parent_coin_id) as parent_coin_id,
                    hex(c.puzzle_hash) as puzzle_hash,
                    c.amount,
                    c.created_block,
                    s.spent_block
                FROM coins c
                LEFT JOIN spends s ON c.coin_id = s.coin_id
                WHERE c.puzzle_hash = ?
                ORDER BY c.created_block DESC
                LIMIT ? OFFSET ?
                "#
            }
        };

        let rows = sqlx::query(query)
            .bind(&puzzle_hash)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(CoinInfo {
                coin_id: row.get("coin_id"),
                parent_coin_id: row.get("parent_coin_id"),
                puzzle_hash: row.get("puzzle_hash"),
                amount: row.get::<i64, _>("amount") as u64,
                created_block: row.get("created_block"),
                spent_block: row.get("spent_block"),
            });
        }

        Ok(results)
    }

    /// Get block by height
    async fn block_by_height(&self, height: i64) -> GraphQLResult<Option<BlockInfo>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                    b.height,
                    encode(b.block_hash, 'hex') as block_hash,
                    encode(b.prev_hash, 'hex') as prev_hash,
                    b.timestamp::text,
                    COUNT(DISTINCT c.coin_id) as coin_count,
                    COUNT(DISTINCT s.coin_id) as spend_count
                FROM blocks b
                LEFT JOIN coins c ON b.height = c.created_block
                LEFT JOIN spends s ON b.height = s.spent_block
                WHERE b.height = $1
                GROUP BY b.height, b.block_hash, b.prev_hash, b.timestamp
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                    b.height,
                    hex(b.block_hash) as block_hash,
                    hex(b.prev_hash) as prev_hash,
                    b.timestamp,
                    COUNT(DISTINCT c.coin_id) as coin_count,
                    COUNT(DISTINCT s.coin_id) as spend_count
                FROM blocks b
                LEFT JOIN coins c ON b.height = c.created_block
                LEFT JOIN spends s ON b.height = s.spent_block
                WHERE b.height = ?
                GROUP BY b.height, b.block_hash, b.prev_hash, b.timestamp
                "#
            }
        };

        let row = sqlx::query(query)
            .bind(height)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(row.map(|r| BlockInfo {
            height: r.get("height"),
            block_hash: r.get("block_hash"),
            prev_hash: r.get("prev_hash"),
            timestamp: r.get("timestamp"),
            coin_count: r.get("coin_count"),
            spend_count: r.get("spend_count"),
        }))
    }

    /// Get recent blocks
    async fn recent_blocks(
        &self,
        #[graphql(default = 10)] limit: i32,
    ) -> GraphQLResult<Vec<BlockInfo>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                    b.height,
                    encode(b.block_hash, 'hex') as block_hash,
                    encode(b.prev_hash, 'hex') as prev_hash,
                    b.timestamp::text,
                    COUNT(DISTINCT c.coin_id) as coin_count,
                    COUNT(DISTINCT s.coin_id) as spend_count
                FROM blocks b
                LEFT JOIN coins c ON b.height = c.created_block
                LEFT JOIN spends s ON b.height = s.spent_block
                GROUP BY b.height, b.block_hash, b.prev_hash, b.timestamp
                ORDER BY b.height DESC
                LIMIT $1
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                    b.height,
                    hex(b.block_hash) as block_hash,
                    hex(b.prev_hash) as prev_hash,
                    b.timestamp,
                    COUNT(DISTINCT c.coin_id) as coin_count,
                    COUNT(DISTINCT s.coin_id) as spend_count
                FROM blocks b
                LEFT JOIN coins c ON b.height = c.created_block
                LEFT JOIN spends s ON b.height = s.spent_block
                GROUP BY b.height, b.block_hash, b.prev_hash, b.timestamp
                ORDER BY b.height DESC
                LIMIT ?
                "#
            }
        };

        let rows = sqlx::query(query)
            .bind(limit)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(BlockInfo {
                height: row.get("height"),
                block_hash: row.get("block_hash"),
                prev_hash: row.get("prev_hash"),
                timestamp: row.get("timestamp"),
                coin_count: row.get("coin_count"),
                spend_count: row.get("spend_count"),
            });
        }

        Ok(results)
    }

    /// Get balance for puzzle hash
    async fn balance_by_puzzle_hash(&self, puzzle_hash_hex: String) -> GraphQLResult<Option<BalanceInfo>> {
        let puzzle_hash = hex::decode(&puzzle_hash_hex)
            .map_err(|_| GraphQLError::InvalidInput("Invalid puzzle hash hex".to_string()))?;

        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                    encode(c.puzzle_hash, 'hex') as puzzle_hash,
                    COALESCE(SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END), 0) as confirmed_balance,
                    COUNT(CASE WHEN s.coin_id IS NULL THEN 1 END) as unspent_coin_count,
                    MIN(c.created_block) as first_activity_block,
                    MAX(COALESCE(s.spent_block, c.created_block)) as last_activity_block
                FROM coins c
                LEFT JOIN spends s ON c.coin_id = s.coin_id
                WHERE c.puzzle_hash = $1
                GROUP BY c.puzzle_hash
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                    hex(c.puzzle_hash) as puzzle_hash,
                    COALESCE(SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END), 0) as confirmed_balance,
                    COUNT(CASE WHEN s.coin_id IS NULL THEN 1 END) as unspent_coin_count,
                    MIN(c.created_block) as first_activity_block,
                    MAX(COALESCE(s.spent_block, c.created_block)) as last_activity_block
                FROM coins c
                LEFT JOIN spends s ON c.coin_id = s.coin_id
                WHERE c.puzzle_hash = ?
                GROUP BY c.puzzle_hash
                "#
            }
        };

        let row = sqlx::query(query)
            .bind(&puzzle_hash)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(row.map(|r| BalanceInfo {
            puzzle_hash: r.get("puzzle_hash"),
            confirmed_balance: r.get::<i64, _>("confirmed_balance") as u64,
            unconfirmed_balance: 0, // For now, assuming no unconfirmed transactions
            unspent_coin_count: r.get("unspent_coin_count"),
            first_activity_block: r.get("first_activity_block"),
            last_activity_block: r.get("last_activity_block"),
        }))
    }
} 