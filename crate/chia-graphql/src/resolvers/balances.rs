use async_graphql::*;
use sqlx::{AnyPool, Row};
use std::sync::Arc;

use crate::database::DatabaseType;
use crate::error::{GraphQLError, GraphQLResult};
use crate::schema::types::*;

/// Balance-related queries
pub struct BalanceQueries {
    pool: Arc<AnyPool>,
    db_type: DatabaseType,
}

impl BalanceQueries {
    pub fn new(pool: Arc<AnyPool>, db_type: DatabaseType) -> Self {
        Self { pool, db_type }
    }
}

#[Object]
impl BalanceQueries {
    /// Get balance distribution statistics
    async fn distribution_stats(&self) -> GraphQLResult<BalanceDistributionStats> {
        let query = r#"
        SELECT 
          COUNT(*) as total_addresses,
          SUM(total_mojos) as total_supply,
          AVG(total_mojos) as avg_balance,
          MIN(total_mojos) as min_balance,
          MAX(total_mojos) as max_balance,
          COUNT(CASE WHEN total_mojos >= 1000000000000 THEN 1 END) as addresses_with_1_xch_plus,
          COUNT(CASE WHEN total_mojos >= 1000000000000000 THEN 1 END) as addresses_with_1000_xch_plus,
          COUNT(CASE WHEN total_mojos >= 1000000000000000000 THEN 1 END) as addresses_with_1m_xch_plus
        FROM xch_balances
        "#;

        let row = sqlx::query(query)
            .fetch_one(&*self.pool)
            .await?;

        Ok(BalanceDistributionStats {
            total_addresses: row.get::<i64, _>("total_addresses"),
            total_supply: row.get::<i64, _>("total_supply") as u64,
            avg_balance: row.get("avg_balance"),
            min_balance: row.get::<i64, _>("min_balance") as u64,
            max_balance: row.get::<i64, _>("max_balance") as u64,
            addresses_with_1_xch_plus: row.get::<i64, _>("addresses_with_1_xch_plus"),
            addresses_with_1000_xch_plus: row.get::<i64, _>("addresses_with_1000_xch_plus"),
            addresses_with_1m_xch_plus: row.get::<i64, _>("addresses_with_1m_xch_plus"),
        })
    }

    /// Get top richest addresses with pagination
    async fn richest_addresses(
        &self,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<RichestAddress>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                  puzzle_hash_hex,
                  total_mojos,
                  total_xch,
                  coin_count,
                  min_coin_amount,
                  max_coin_amount,
                  avg_coin_amount,
                  RANK() OVER (ORDER BY total_mojos DESC) as rank_position
                FROM xch_balances
                ORDER BY total_mojos DESC
                LIMIT $1 OFFSET $2
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                  puzzle_hash_hex,
                  total_mojos,
                  total_xch,
                  coin_count,
                  min_coin_amount,
                  max_coin_amount,
                  avg_coin_amount,
                  RANK() OVER (ORDER BY total_mojos DESC) as rank_position
                FROM xch_balances
                ORDER BY total_mojos DESC
                LIMIT ? OFFSET ?
                "#
            }
        };

        let offset = (page - 1) * page_size;
        
        let rows = sqlx::query(query)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(RichestAddress {
                puzzle_hash_hex: row.get("puzzle_hash_hex"),
                total_mojos: row.get::<i64, _>("total_mojos") as u64,
                total_xch: row.get("total_xch"),
                coin_count: row.get::<i64, _>("coin_count"),
                min_coin_amount: row.get::<i64, _>("min_coin_amount") as u64,
                max_coin_amount: row.get::<i64, _>("max_coin_amount") as u64,
                avg_coin_amount: row.get::<i64, _>("avg_coin_amount") as u64,
                rank_position: row.get::<i64, _>("rank_position"),
            });
        }

        Ok(results)
    }

    /// Get balance changes for an address within a block range
    async fn balance_changes(
        &self,
        puzzle_hash: String,
        start_block: i64,
        end_block: i64,
    ) -> GraphQLResult<Vec<BalanceChange>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH balance_changes AS (
                  -- Coins received
                  SELECT 
                    c.created_block as block_height,
                    c.amount as net_change,
                    1 as coins_added,
                    0 as coins_spent
                  FROM coins c
                  WHERE encode(c.puzzle_hash, 'hex') = $1
                    AND c.created_block >= $2 
                    AND c.created_block <= $3
                  
                  UNION ALL
                  
                  -- Coins spent
                  SELECT 
                    s.spent_block as block_height,
                    -c.amount as net_change,
                    0 as coins_added,
                    1 as coins_spent
                  FROM spends s
                  JOIN coins c ON s.coin_id = c.coin_id
                  WHERE encode(c.puzzle_hash, 'hex') = $1
                    AND s.spent_block >= $2
                    AND s.spent_block <= $3
                )
                SELECT 
                  block_height,
                  SUM(net_change) as net_change,
                  SUM(coins_added) as coins_added,
                  SUM(coins_spent) as coins_spent
                FROM balance_changes
                GROUP BY block_height
                ORDER BY block_height
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH balance_changes AS (
                  -- Coins received
                  SELECT 
                    c.created_block as block_height,
                    c.amount as net_change,
                    1 as coins_added,
                    0 as coins_spent
                  FROM coins c
                  WHERE hex(c.puzzle_hash) = ?
                    AND c.created_block >= ? 
                    AND c.created_block <= ?
                  
                  UNION ALL
                  
                  -- Coins spent
                  SELECT 
                    s.spent_block as block_height,
                    -c.amount as net_change,
                    0 as coins_added,
                    1 as coins_spent
                  FROM spends s
                  JOIN coins c ON s.coin_id = c.coin_id
                  WHERE hex(c.puzzle_hash) = ?
                    AND s.spent_block >= ?
                    AND s.spent_block <= ?
                )
                SELECT 
                  block_height,
                  SUM(net_change) as net_change,
                  SUM(coins_added) as coins_added,
                  SUM(coins_spent) as coins_spent
                FROM balance_changes
                GROUP BY block_height
                ORDER BY block_height
                "#
            }
        };

        let rows = match self.db_type {
            DatabaseType::Postgres => {
                sqlx::query(query)
                    .bind(&puzzle_hash)
                    .bind(start_block)
                    .bind(end_block)
                    .fetch_all(&*self.pool)
                    .await?
            }
            DatabaseType::Sqlite => {
                sqlx::query(query)
                    .bind(&puzzle_hash)
                    .bind(start_block)
                    .bind(end_block)
                    .bind(&puzzle_hash)
                    .bind(start_block)
                    .bind(end_block)
                    .fetch_all(&*self.pool)
                    .await?
            }
        };

        let mut results = Vec::new();
        for row in rows {
            results.push(BalanceChange {
                block_height: row.get("block_height"),
                net_change: row.get("net_change"),
                coins_added: row.get("coins_added"),
                coins_spent: row.get("coins_spent"),
            });
        }

        Ok(results)
    }

    /// Get addresses within a balance range
    async fn addresses_by_balance_range(
        &self,
        min_balance: u64,
        max_balance: u64,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<AddressInRange>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                  puzzle_hash_hex,
                  total_mojos,
                  total_xch,
                  coin_count
                FROM xch_balances
                WHERE total_mojos >= $1 AND total_mojos <= $2
                ORDER BY total_mojos DESC
                LIMIT $3 OFFSET $4
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                  puzzle_hash_hex,
                  total_mojos,
                  total_xch,
                  coin_count
                FROM xch_balances
                WHERE total_mojos >= ? AND total_mojos <= ?
                ORDER BY total_mojos DESC
                LIMIT ? OFFSET ?
                "#
            }
        };

        let offset = (page - 1) * page_size;
        
        let rows = sqlx::query(query)
            .bind(min_balance as i64)
            .bind(max_balance as i64)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(AddressInRange {
                puzzle_hash_hex: row.get("puzzle_hash_hex"),
                total_mojos: row.get::<i64, _>("total_mojos") as u64,
                total_xch: row.get("total_xch"),
                coin_count: row.get::<i64, _>("coin_count"),
            });
        }

        Ok(results)
    }

    /// Get coin size distribution
    async fn coin_size_distribution(&self) -> GraphQLResult<Vec<CoinSizeDistribution>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH size_categories AS (
                  SELECT 
                    CASE 
                      WHEN amount < 1000000000 THEN 'dust'
                      WHEN amount < 10000000000 THEN 'tiny'
                      WHEN amount < 100000000000 THEN 'small'
                      WHEN amount < 1000000000000 THEN 'medium'
                      WHEN amount < 10000000000000 THEN 'large'
                      WHEN amount < 100000000000000 THEN 'huge'
                      ELSE 'whale'
                    END as size_category,
                    amount
                  FROM coins
                  WHERE NOT EXISTS (SELECT 1 FROM spends WHERE coin_id = coins.coin_id)
                )
                SELECT 
                  size_category,
                  COUNT(*) as coin_count,
                  SUM(amount) as total_amount,
                  MIN(amount) as min_amount,
                  MAX(amount) as max_amount,
                  AVG(amount) as avg_amount
                FROM size_categories
                GROUP BY size_category
                ORDER BY MIN(amount)
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH size_categories AS (
                  SELECT 
                    CASE 
                      WHEN amount < 1000000000 THEN 'dust'
                      WHEN amount < 10000000000 THEN 'tiny'
                      WHEN amount < 100000000000 THEN 'small'
                      WHEN amount < 1000000000000 THEN 'medium'
                      WHEN amount < 10000000000000 THEN 'large'
                      WHEN amount < 100000000000000 THEN 'huge'
                      ELSE 'whale'
                    END as size_category,
                    amount
                  FROM coins
                  WHERE NOT EXISTS (SELECT 1 FROM spends WHERE coin_id = coins.coin_id)
                )
                SELECT 
                  size_category,
                  COUNT(*) as coin_count,
                  SUM(amount) as total_amount,
                  MIN(amount) as min_amount,
                  MAX(amount) as max_amount,
                  AVG(amount) as avg_amount
                FROM size_categories
                GROUP BY size_category
                ORDER BY MIN(amount)
                "#
            }
        };

        let rows = sqlx::query(query)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(CoinSizeDistribution {
                size_category: row.get("size_category"),
                coin_count: row.get::<i64, _>("coin_count"),
                total_amount: row.get::<i64, _>("total_amount") as u64,
                min_amount: row.get::<i64, _>("min_amount") as u64,
                max_amount: row.get::<i64, _>("max_amount") as u64,
                avg_amount: row.get::<i64, _>("avg_amount") as u64,
            });
        }

        Ok(results)
    }

    /// Get balance percentiles
    async fn percentiles(&self) -> GraphQLResult<BalancePercentiles> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                  PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY total_mojos) as median,
                  PERCENTILE_CONT(0.75) WITHIN GROUP (ORDER BY total_mojos) as percentile_75,
                  PERCENTILE_CONT(0.9) WITHIN GROUP (ORDER BY total_mojos) as percentile_90,
                  PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY total_mojos) as percentile_95,
                  PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY total_mojos) as percentile_99
                FROM xch_balances
                "#
            }
            DatabaseType::Sqlite => {
                // SQLite doesn't have PERCENTILE_CONT, so we approximate
                r#"
                WITH sorted_balances AS (
                  SELECT 
                    total_mojos,
                    ROW_NUMBER() OVER (ORDER BY total_mojos) as row_num,
                    COUNT(*) OVER () as total_count
                  FROM xch_balances
                )
                SELECT 
                  (SELECT total_mojos FROM sorted_balances WHERE row_num = CAST(total_count * 0.5 AS INTEGER)) as median,
                  (SELECT total_mojos FROM sorted_balances WHERE row_num = CAST(total_count * 0.75 AS INTEGER)) as percentile_75,
                  (SELECT total_mojos FROM sorted_balances WHERE row_num = CAST(total_count * 0.9 AS INTEGER)) as percentile_90,
                  (SELECT total_mojos FROM sorted_balances WHERE row_num = CAST(total_count * 0.95 AS INTEGER)) as percentile_95,
                  (SELECT total_mojos FROM sorted_balances WHERE row_num = CAST(total_count * 0.99 AS INTEGER)) as percentile_99
                FROM sorted_balances
                LIMIT 1
                "#
            }
        };

        let row = sqlx::query(query)
            .fetch_one(&*self.pool)
            .await?;

        Ok(BalancePercentiles {
            median: row.get::<i64, _>("median") as u64,
            percentile_75: row.get::<i64, _>("percentile_75") as u64,
            percentile_90: row.get::<i64, _>("percentile_90") as u64,
            percentile_95: row.get::<i64, _>("percentile_95") as u64,
            percentile_99: row.get::<i64, _>("percentile_99") as u64,
        })
    }
} 