use async_graphql::*;
use sqlx::{AnyPool, Row};
use std::sync::Arc;

use crate::database::DatabaseType;
use crate::error::GraphQLResult;
use crate::schema::types::*;

/// Transaction analytics queries
pub struct TransactionQueries {
    pool: Arc<AnyPool>,
    db_type: DatabaseType,
}

impl TransactionQueries {
    pub fn new(pool: Arc<AnyPool>, db_type: DatabaseType) -> Self {
        Self { pool, db_type }
    }
}

#[Object]
impl TransactionQueries {
    /// Get spend velocity analysis showing how quickly coins are spent
    async fn spend_velocity_analysis(&self) -> GraphQLResult<SpendVelocityAnalysis> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH spend_data AS (
                  SELECT 
                    c.coin_id,
                    c.amount,
                    c.created_block,
                    s.spent_block,
                    (s.spent_block - c.created_block) as blocks_to_spend
                  FROM coins c
                  JOIN spends s ON c.coin_id = s.coin_id
                  WHERE s.spent_block > c.created_block
                )
                SELECT 
                  ROUND(AVG(blocks_to_spend), 2) as avg_blocks_to_spend,
                  PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY blocks_to_spend) as median_blocks_to_spend,
                  COUNT(CASE WHEN blocks_to_spend = 0 THEN 1 END) as immediate_spends,
                  COUNT(CASE WHEN blocks_to_spend BETWEEN 1 AND 10 THEN 1 END) as quick_spends,
                  COUNT(CASE WHEN blocks_to_spend BETWEEN 11 AND 100 THEN 1 END) as medium_spends,
                  COUNT(CASE WHEN blocks_to_spend > 100 THEN 1 END) as slow_spends,
                  COUNT(*) as total_coins_analyzed,
                  COALESCE(SUM(amount), 0) as total_value_spent
                FROM spend_data
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH spend_data AS (
                  SELECT 
                    c.coin_id,
                    c.amount,
                    c.created_block,
                    s.spent_block,
                    (s.spent_block - c.created_block) as blocks_to_spend
                  FROM coins c
                  JOIN spends s ON c.coin_id = s.coin_id
                  WHERE s.spent_block > c.created_block
                )
                SELECT 
                  ROUND(AVG(blocks_to_spend), 2) as avg_blocks_to_spend,
                  NULL as median_blocks_to_spend,
                  COUNT(CASE WHEN blocks_to_spend = 0 THEN 1 END) as immediate_spends,
                  COUNT(CASE WHEN blocks_to_spend BETWEEN 1 AND 10 THEN 1 END) as quick_spends,
                  COUNT(CASE WHEN blocks_to_spend BETWEEN 11 AND 100 THEN 1 END) as medium_spends,
                  COUNT(CASE WHEN blocks_to_spend > 100 THEN 1 END) as slow_spends,
                  COUNT(*) as total_coins_analyzed,
                  COALESCE(SUM(amount), 0) as total_value_spent
                FROM spend_data
                "#
            }
        };

        let row = sqlx::query(query)
            .fetch_one(&*self.pool)
            .await?;

        Ok(SpendVelocityAnalysis {
            avg_blocks_to_spend: row.get::<f64, _>("avg_blocks_to_spend"),
            median_blocks_to_spend: row.get("median_blocks_to_spend"),
            immediate_spends: row.get::<i64, _>("immediate_spends"),
            quick_spends: row.get::<i64, _>("quick_spends"),
            medium_spends: row.get::<i64, _>("medium_spends"),
            slow_spends: row.get::<i64, _>("slow_spends"),
            total_coins_analyzed: row.get::<i64, _>("total_coins_analyzed"),
            total_value_spent: row.get::<i64, _>("total_value_spent") as u64,
        })
    }

    /// Get complex transaction patterns
    async fn complex_transaction_patterns(
        &self,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<ComplexTransactionPattern>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH transaction_data AS (
                  SELECT 
                    s.spent_block,
                    b.timestamp as block_timestamp,
                    COUNT(DISTINCT s.coin_id) as coins_spent,
                    COUNT(DISTINCT nc.coin_id) as coins_created,
                    COALESCE(SUM(c.amount), 0) as total_value_in,
                    COALESCE(SUM(nc.amount), 0) as total_value_out,
                    COUNT(DISTINCT s.puzzle_id) as unique_puzzles_used,
                    COUNT(DISTINCT s.solution_id) as unique_solutions_used
                  FROM spends s
                  JOIN coins c ON s.coin_id = c.coin_id
                  JOIN blocks b ON s.spent_block = b.height
                  LEFT JOIN coins nc ON nc.created_block = s.spent_block
                  GROUP BY s.spent_block, b.timestamp
                  HAVING COUNT(DISTINCT s.coin_id) > 5
                ),
                complexity_scores AS (
                  SELECT 
                    *,
                    (coins_spent * 0.3 + coins_created * 0.3 + unique_puzzles_used * 0.2 + unique_solutions_used * 0.2) as complexity_score,
                    (total_value_in - total_value_out) as fee_amount
                  FROM transaction_data
                )
                SELECT 
                  spent_block,
                  block_timestamp::text,
                  coins_spent,
                  coins_created,
                  total_value_in,
                  total_value_out,
                  GREATEST(fee_amount, 0) as fee_amount,
                  unique_puzzles_used,
                  unique_solutions_used,
                  complexity_score
                FROM complexity_scores
                ORDER BY complexity_score DESC
                LIMIT $1 OFFSET $2
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH transaction_data AS (
                  SELECT 
                    s.spent_block,
                    b.timestamp as block_timestamp,
                    COUNT(DISTINCT s.coin_id) as coins_spent,
                    COUNT(DISTINCT nc.coin_id) as coins_created,
                    COALESCE(SUM(c.amount), 0) as total_value_in,
                    COALESCE(SUM(nc.amount), 0) as total_value_out,
                    COUNT(DISTINCT s.puzzle_id) as unique_puzzles_used,
                    COUNT(DISTINCT s.solution_id) as unique_solutions_used
                  FROM spends s
                  JOIN coins c ON s.coin_id = c.coin_id
                  JOIN blocks b ON s.spent_block = b.height
                  LEFT JOIN coins nc ON nc.created_block = s.spent_block
                  GROUP BY s.spent_block, b.timestamp
                  HAVING COUNT(DISTINCT s.coin_id) > 5
                ),
                complexity_scores AS (
                  SELECT 
                    *,
                    (coins_spent * 0.3 + coins_created * 0.3 + unique_puzzles_used * 0.2 + unique_solutions_used * 0.2) as complexity_score,
                    (total_value_in - total_value_out) as fee_amount
                  FROM transaction_data
                )
                SELECT 
                  spent_block,
                  block_timestamp,
                  coins_spent,
                  coins_created,
                  total_value_in,
                  total_value_out,
                  MAX(fee_amount, 0) as fee_amount,
                  unique_puzzles_used,
                  unique_solutions_used,
                  complexity_score
                FROM complexity_scores
                ORDER BY complexity_score DESC
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
            results.push(ComplexTransactionPattern {
                spent_block: row.get::<i64, _>("spent_block"),
                block_timestamp: row.get::<String, _>("block_timestamp"),
                coins_spent: row.get::<i64, _>("coins_spent"),
                coins_created: row.get::<i64, _>("coins_created"),
                total_value_in: row.get::<i64, _>("total_value_in") as u64,
                total_value_out: row.get::<i64, _>("total_value_out") as u64,
                fee_amount: row.get::<i64, _>("fee_amount") as u64,
                unique_puzzles_used: row.get::<i64, _>("unique_puzzles_used"),
                unique_solutions_used: row.get::<i64, _>("unique_solutions_used"),
                complexity_score: row.get::<f64, _>("complexity_score"),
            });
        }

        Ok(results)
    }

    /// Get spending frequency analysis by address
    async fn spending_frequency_by_address(
        &self,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<SpendingFrequencyByAddress>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH address_spending AS (
                  SELECT 
                    encode(c.puzzle_hash, 'hex') as puzzle_hash_hex,
                    COUNT(DISTINCT s.coin_id) as total_coins_spent,
                    COALESCE(SUM(c.amount), 0) as total_value_spent,
                    MIN(s.spent_block) as first_spend_block,
                    MAX(s.spent_block) as last_spend_block,
                    COUNT(DISTINCT s.spent_block) as active_spending_blocks
                  FROM spends s
                  JOIN coins c ON s.coin_id = c.coin_id
                  GROUP BY c.puzzle_hash
                  HAVING COUNT(DISTINCT s.coin_id) >= 10
                ),
                frequency_scores AS (
                  SELECT 
                    *,
                    CASE 
                      WHEN (last_spend_block - first_spend_block) > 0 
                      THEN ROUND(total_coins_spent::NUMERIC / (last_spend_block - first_spend_block)::NUMERIC, 4)
                      ELSE 0
                    END as avg_spends_per_block,
                    CASE 
                      WHEN (last_spend_block - first_spend_block) > 0 
                      THEN ROUND((active_spending_blocks::NUMERIC / (last_spend_block - first_spend_block)::NUMERIC) * 100, 2)
                      ELSE 0
                    END as spending_frequency_score
                  FROM address_spending
                )
                SELECT * FROM frequency_scores
                ORDER BY spending_frequency_score DESC
                LIMIT $1 OFFSET $2
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH address_spending AS (
                  SELECT 
                    hex(c.puzzle_hash) as puzzle_hash_hex,
                    COUNT(DISTINCT s.coin_id) as total_coins_spent,
                    COALESCE(SUM(c.amount), 0) as total_value_spent,
                    MIN(s.spent_block) as first_spend_block,
                    MAX(s.spent_block) as last_spend_block,
                    COUNT(DISTINCT s.spent_block) as active_spending_blocks
                  FROM spends s
                  JOIN coins c ON s.coin_id = c.coin_id
                  GROUP BY c.puzzle_hash
                  HAVING COUNT(DISTINCT s.coin_id) >= 10
                ),
                frequency_scores AS (
                  SELECT 
                    *,
                    CASE 
                      WHEN (last_spend_block - first_spend_block) > 0 
                      THEN ROUND(CAST(total_coins_spent AS REAL) / CAST((last_spend_block - first_spend_block) AS REAL), 4)
                      ELSE 0
                    END as avg_spends_per_block,
                    CASE 
                      WHEN (last_spend_block - first_spend_block) > 0 
                      THEN ROUND((CAST(active_spending_blocks AS REAL) / CAST((last_spend_block - first_spend_block) AS REAL)) * 100, 2)
                      ELSE 0
                    END as spending_frequency_score
                  FROM address_spending
                )
                SELECT * FROM frequency_scores
                ORDER BY spending_frequency_score DESC
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
            results.push(SpendingFrequencyByAddress {
                puzzle_hash_hex: row.get::<String, _>("puzzle_hash_hex"),
                total_coins_spent: row.get::<i64, _>("total_coins_spent"),
                total_value_spent: row.get::<i64, _>("total_value_spent") as u64,
                first_spend_block: row.get::<i64, _>("first_spend_block"),
                last_spend_block: row.get::<i64, _>("last_spend_block"),
                active_spending_blocks: row.get::<i64, _>("active_spending_blocks"),
                avg_spends_per_block: row.get::<f64, _>("avg_spends_per_block"),
                spending_frequency_score: row.get::<f64, _>("spending_frequency_score"),
            });
        }

        Ok(results)
    }

    /// Get transaction fee analysis
    async fn fee_analysis(
        &self,
        #[graphql(default = 24)] hours_back: i32,
    ) -> GraphQLResult<TransactionFeeAnalysis> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH fee_data AS (
                  SELECT 
                    s.spent_block,
                    COALESCE(SUM(c.amount), 0) as total_value_in,
                    COALESCE(SUM(nc.amount), 0) as total_value_out,
                    (COALESCE(SUM(c.amount), 0) - COALESCE(SUM(nc.amount), 0)) as fee_amount
                  FROM spends s
                  JOIN coins c ON s.coin_id = c.coin_id
                  JOIN blocks b ON s.spent_block = b.height
                  LEFT JOIN coins nc ON nc.created_block = s.spent_block
                  WHERE b.timestamp >= NOW() - INTERVAL '%d hours'
                  GROUP BY s.spent_block
                  HAVING (COALESCE(SUM(c.amount), 0) - COALESCE(SUM(nc.amount), 0)) >= 0
                )
                SELECT 
                  COUNT(*) as transaction_count,
                  COALESCE(SUM(fee_amount), 0) as total_fees,
                  ROUND(AVG(fee_amount), 2) as avg_fee_per_transaction,
                  PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY fee_amount) as median_fee,
                  MIN(fee_amount) as min_fee,
                  MAX(fee_amount) as max_fee,
                  ROUND(STDDEV(fee_amount), 2) as fee_stddev,
                  COUNT(CASE WHEN fee_amount = 0 THEN 1 END) as zero_fee_transactions,
                  COUNT(CASE WHEN fee_amount > 1000000000000 THEN 1 END) as high_fee_transactions
                FROM fee_data
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH fee_data AS (
                  SELECT 
                    s.spent_block,
                    COALESCE(SUM(c.amount), 0) as total_value_in,
                    COALESCE(SUM(nc.amount), 0) as total_value_out,
                    (COALESCE(SUM(c.amount), 0) - COALESCE(SUM(nc.amount), 0)) as fee_amount
                  FROM spends s
                  JOIN coins c ON s.coin_id = c.coin_id
                  JOIN blocks b ON s.spent_block = b.height
                  LEFT JOIN coins nc ON nc.created_block = s.spent_block
                  WHERE datetime(b.timestamp) >= datetime('now', '-%d hours')
                  GROUP BY s.spent_block
                  HAVING (COALESCE(SUM(c.amount), 0) - COALESCE(SUM(nc.amount), 0)) >= 0
                )
                SELECT 
                  COUNT(*) as transaction_count,
                  COALESCE(SUM(fee_amount), 0) as total_fees,
                  ROUND(AVG(fee_amount), 2) as avg_fee_per_transaction,
                  NULL as median_fee,
                  MIN(fee_amount) as min_fee,
                  MAX(fee_amount) as max_fee,
                  NULL as fee_stddev,
                  COUNT(CASE WHEN fee_amount = 0 THEN 1 END) as zero_fee_transactions,
                  COUNT(CASE WHEN fee_amount > 1000000000000 THEN 1 END) as high_fee_transactions
                FROM fee_data
                "#
            }
        };

        let formatted_query = query.replace("%d", &hours_back.to_string());
        
        let row = sqlx::query(&formatted_query)
            .fetch_one(&*self.pool)
            .await?;

        Ok(TransactionFeeAnalysis {
            time_period: format!("Last {} hours", hours_back),
            total_fees: row.get::<i64, _>("total_fees") as u64,
            avg_fee_per_transaction: row.get::<f64, _>("avg_fee_per_transaction"),
            median_fee: row.get("median_fee"),
            min_fee: row.get::<i64, _>("min_fee") as u64,
            max_fee: row.get::<i64, _>("max_fee") as u64,
            fee_stddev: row.get("fee_stddev"),
            zero_fee_transactions: row.get::<i64, _>("zero_fee_transactions"),
            high_fee_transactions: row.get::<i64, _>("high_fee_transactions"),
        })
    }

    /// Get transaction patterns by type
    async fn patterns_by_type(&self) -> GraphQLResult<Vec<TransactionPatternByType>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH transaction_types AS (
                  SELECT 
                    CASE 
                      WHEN coins_spent = 1 AND coins_created = 1 THEN 'simple_transfer'
                      WHEN coins_spent = 1 AND coins_created = 2 THEN 'split_transfer'
                      WHEN coins_spent > 1 AND coins_created = 1 THEN 'consolidation'
                      WHEN coins_spent > 1 AND coins_created > 1 AND coins_spent = coins_created THEN 'complex_transfer'
                      WHEN coins_spent > coins_created THEN 'aggregation'
                      WHEN coins_created > coins_spent THEN 'distribution'
                      ELSE 'other'
                    END as transaction_type,
                    total_value_in,
                    total_value_out,
                    fee_amount
                  FROM (
                    SELECT 
                      s.spent_block,
                      COUNT(DISTINCT s.coin_id) as coins_spent,
                      COUNT(DISTINCT nc.coin_id) as coins_created,
                      COALESCE(SUM(c.amount), 0) as total_value_in,
                      COALESCE(SUM(nc.amount), 0) as total_value_out,
                      (COALESCE(SUM(c.amount), 0) - COALESCE(SUM(nc.amount), 0)) as fee_amount
                    FROM spends s
                    JOIN coins c ON s.coin_id = c.coin_id
                    LEFT JOIN coins nc ON nc.created_block = s.spent_block
                    GROUP BY s.spent_block
                  ) tx_stats
                )
                SELECT 
                  transaction_type,
                  COUNT(*) as count,
                  ROUND(AVG(total_value_in), 2) as avg_value_in,
                  ROUND(AVG(total_value_out), 2) as avg_value_out,
                  ROUND(AVG(fee_amount), 2) as avg_fee
                FROM transaction_types
                GROUP BY transaction_type
                ORDER BY count DESC
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH transaction_types AS (
                  SELECT 
                    CASE 
                      WHEN coins_spent = 1 AND coins_created = 1 THEN 'simple_transfer'
                      WHEN coins_spent = 1 AND coins_created = 2 THEN 'split_transfer'
                      WHEN coins_spent > 1 AND coins_created = 1 THEN 'consolidation'
                      WHEN coins_spent > 1 AND coins_created > 1 AND coins_spent = coins_created THEN 'complex_transfer'
                      WHEN coins_spent > coins_created THEN 'aggregation'
                      WHEN coins_created > coins_spent THEN 'distribution'
                      ELSE 'other'
                    END as transaction_type,
                    total_value_in,
                    total_value_out,
                    fee_amount
                  FROM (
                    SELECT 
                      s.spent_block,
                      COUNT(DISTINCT s.coin_id) as coins_spent,
                      COUNT(DISTINCT nc.coin_id) as coins_created,
                      COALESCE(SUM(c.amount), 0) as total_value_in,
                      COALESCE(SUM(nc.amount), 0) as total_value_out,
                      (COALESCE(SUM(c.amount), 0) - COALESCE(SUM(nc.amount), 0)) as fee_amount
                    FROM spends s
                    JOIN coins c ON s.coin_id = c.coin_id
                    LEFT JOIN coins nc ON nc.created_block = s.spent_block
                    GROUP BY s.spent_block
                  ) tx_stats
                )
                SELECT 
                  transaction_type,
                  COUNT(*) as count,
                  ROUND(AVG(total_value_in), 2) as avg_value_in,
                  ROUND(AVG(total_value_out), 2) as avg_value_out,
                  ROUND(AVG(fee_amount), 2) as avg_fee
                FROM transaction_types
                GROUP BY transaction_type
                ORDER BY count DESC
                "#
            }
        };

        let rows = sqlx::query(query)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(TransactionPatternByType {
                transaction_type: row.get::<String, _>("transaction_type"),
                count: row.get::<i64, _>("count"),
                avg_value_in: row.get::<f64, _>("avg_value_in"),
                avg_value_out: row.get::<f64, _>("avg_value_out"),
                avg_fee: row.get::<f64, _>("avg_fee"),
            });
        }

        Ok(results)
    }
} 