use async_graphql::*;
use sqlx::{AnyPool, Row};
use std::sync::Arc;

use crate::database::DatabaseType;
use crate::error::GraphQLResult;
use crate::schema::types::*;

/// Network analytics queries
pub struct NetworkQueries {
    pool: Arc<AnyPool>,
    db_type: DatabaseType,
}

impl NetworkQueries {
    pub fn new(pool: Arc<AnyPool>, db_type: DatabaseType) -> Self {
        Self { pool, db_type }
    }
}

#[Object]
impl NetworkQueries {
    /// Get comprehensive network statistics
    async fn network_stats(&self) -> GraphQLResult<NetworkStats> {
        let query = r#"
        SELECT 
          COUNT(DISTINCT b.height) as total_blocks,
          COUNT(DISTINCT c.coin_id) as total_coins,
          COUNT(DISTINCT s.coin_id) as total_spends,
          COUNT(DISTINCT p.puzzle_hash) as unique_puzzles,
          COUNT(DISTINCT sol.solution_hash) as unique_solutions,
          COALESCE(MAX(b.height), 0) as current_height,
          COALESCE(SUM(c.amount), 0) as total_coin_value,
          COUNT(DISTINCT CASE WHEN c.coin_id NOT IN (SELECT coin_id FROM spends) THEN c.coin_id END) as unspent_coins,
          COUNT(DISTINCT CASE WHEN c.coin_id NOT IN (SELECT coin_id FROM spends) THEN c.puzzle_hash END) as total_addresses
        FROM blocks b
        LEFT JOIN coins c ON b.height = c.created_block
        LEFT JOIN spends s ON c.coin_id = s.coin_id
        LEFT JOIN puzzles p ON s.puzzle_id = p.id
        LEFT JOIN solutions sol ON s.solution_id = sol.id
        "#;

        let row = sqlx::query(query)
            .fetch_one(&*self.pool)
            .await?;

        let total_blocks: i64 = row.get("total_blocks");
        let total_coins: i64 = row.get("total_coins");
        let total_coin_value: i64 = row.get("total_coin_value");

        Ok(NetworkStats {
            total_blocks,
            total_coins,
            total_addresses: row.get("total_addresses"),
            total_value: total_coin_value as u64,
            avg_block_time_seconds: 18.75, // Chia's target block time
            avg_coins_per_block: if total_blocks > 0 { total_coins as f64 / total_blocks as f64 } else { 0.0 },
            avg_value_per_block: if total_blocks > 0 { total_coin_value as f64 / total_blocks as f64 } else { 0.0 },
            unique_puzzles: row.get("unique_puzzles"),
            unique_solutions: row.get("unique_solutions"),
            current_height: row.get("current_height"),
        })
    }

    /// Get block production statistics
    async fn block_production_stats(&self) -> GraphQLResult<BlockProductionStats> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH block_intervals AS (
                  SELECT 
                    height,
                    timestamp,
                    LAG(timestamp) OVER (ORDER BY height) as prev_timestamp,
                    EXTRACT(EPOCH FROM (timestamp - LAG(timestamp) OVER (ORDER BY height))) as interval_seconds
                  FROM blocks
                  WHERE height > 0
                  ORDER BY height
                ),
                time_periods AS (
                  SELECT 
                    COUNT(CASE WHEN DATE(timestamp) = CURRENT_DATE THEN 1 END) as blocks_today,
                    COUNT(CASE WHEN timestamp >= CURRENT_DATE - INTERVAL '7 days' THEN 1 END) as blocks_this_week,
                    COUNT(CASE WHEN timestamp >= CURRENT_DATE - INTERVAL '30 days' THEN 1 END) as blocks_this_month
                  FROM blocks
                )
                SELECT 
                  ROUND(AVG(interval_seconds), 2) as avg_block_time_seconds,
                  ROUND(STDDEV(interval_seconds), 2) as block_time_stddev,
                  MIN(interval_seconds) as min_block_time,
                  MAX(interval_seconds) as max_block_time,
                  PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY interval_seconds) as median_block_time,
                  tp.blocks_today,
                  tp.blocks_this_week,
                  tp.blocks_this_month
                FROM block_intervals bi, time_periods tp
                WHERE bi.interval_seconds IS NOT NULL
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH block_intervals AS (
                  SELECT 
                    height,
                    timestamp,
                    LAG(timestamp) OVER (ORDER BY height) as prev_timestamp,
                    (julianday(timestamp) - julianday(LAG(timestamp) OVER (ORDER BY height))) * 86400 as interval_seconds
                  FROM blocks
                  WHERE height > 0
                  ORDER BY height
                ),
                time_periods AS (
                  SELECT 
                    COUNT(CASE WHEN DATE(timestamp) = DATE('now') THEN 1 END) as blocks_today,
                    COUNT(CASE WHEN datetime(timestamp) >= datetime('now', '-7 days') THEN 1 END) as blocks_this_week,
                    COUNT(CASE WHEN datetime(timestamp) >= datetime('now', '-30 days') THEN 1 END) as blocks_this_month
                  FROM blocks
                )
                SELECT 
                  ROUND(AVG(interval_seconds), 2) as avg_block_time_seconds,
                  MIN(interval_seconds) as min_block_time,
                  MAX(interval_seconds) as max_block_time,
                  NULL as block_time_stddev,
                  NULL as median_block_time,
                  tp.blocks_today,
                  tp.blocks_this_week,
                  tp.blocks_this_month
                FROM block_intervals bi, time_periods tp
                WHERE bi.interval_seconds IS NOT NULL
                "#
            }
        };

        let row = sqlx::query(query)
            .fetch_one(&*self.pool)
            .await?;

        Ok(BlockProductionStats {
            avg_block_time_seconds: row.get::<f64, _>("avg_block_time_seconds"),
            min_block_time_seconds: row.get::<f64, _>("min_block_time") as i64,
            max_block_time_seconds: row.get::<f64, _>("max_block_time") as i64,
            median_block_time: row.get("median_block_time"),
            block_time_stddev: row.get("block_time_stddev"),
            blocks_today: row.get::<i64, _>("blocks_today"),
            blocks_this_week: row.get::<i64, _>("blocks_this_week"),
            blocks_this_month: row.get::<i64, _>("blocks_this_month"),
        })
    }

    /// Get network growth metrics over time
    async fn growth_metrics(
        &self,
        #[graphql(default = 30)] days_back: i32,
    ) -> GraphQLResult<Vec<NetworkGrowthMetrics>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH daily_metrics AS (
                  SELECT 
                    DATE(b.timestamp) as metric_date,
                    COUNT(DISTINCT b.height) as blocks_created,
                    COUNT(DISTINCT c.coin_id) as coins_created,
                    COUNT(DISTINCT s.coin_id) as coins_spent,
                    COUNT(DISTINCT c.puzzle_hash) as unique_addresses,
                    COALESCE(SUM(c.amount), 0) as total_coin_value
                  FROM blocks b
                  LEFT JOIN coins c ON b.height = c.created_block
                  LEFT JOIN spends s ON c.coin_id = s.coin_id AND s.spent_block = b.height
                  WHERE b.timestamp >= NOW() - INTERVAL '%d days'
                  GROUP BY DATE(b.timestamp)
                  ORDER BY metric_date DESC
                ),
                cumulative_stats AS (
                  SELECT 
                    dm.metric_date,
                    dm.blocks_created,
                    dm.coins_created,
                    dm.unique_addresses as new_addresses,
                    dm.total_coin_value as value_created,
                    SUM(dm.blocks_created) OVER (ORDER BY dm.metric_date ROWS UNBOUNDED PRECEDING) as total_blocks,
                    SUM(dm.coins_created) OVER (ORDER BY dm.metric_date ROWS UNBOUNDED PRECEDING) as total_coins,
                    SUM(dm.unique_addresses) OVER (ORDER BY dm.metric_date ROWS UNBOUNDED PRECEDING) as unique_addresses,
                    SUM(dm.total_coin_value) OVER (ORDER BY dm.metric_date ROWS UNBOUNDED PRECEDING) as total_value
                  FROM daily_metrics dm
                )
                SELECT 
                  metric_date::text,
                  total_blocks,
                  blocks_created,
                  unique_addresses,
                  new_addresses,
                  total_coins,
                  coins_created,
                  total_value,
                  value_created
                FROM cumulative_stats
                ORDER BY metric_date DESC
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH daily_metrics AS (
                  SELECT 
                    DATE(b.timestamp) as metric_date,
                    COUNT(DISTINCT b.height) as blocks_created,
                    COUNT(DISTINCT c.coin_id) as coins_created,
                    COUNT(DISTINCT s.coin_id) as coins_spent,
                    COUNT(DISTINCT c.puzzle_hash) as unique_addresses,
                    COALESCE(SUM(c.amount), 0) as total_coin_value
                  FROM blocks b
                  LEFT JOIN coins c ON b.height = c.created_block
                  LEFT JOIN spends s ON c.coin_id = s.coin_id AND s.spent_block = b.height
                  WHERE datetime(b.timestamp) >= datetime('now', '-%d days')
                  GROUP BY DATE(b.timestamp)
                  ORDER BY metric_date DESC
                ),
                cumulative_stats AS (
                  SELECT 
                    dm.metric_date,
                    dm.blocks_created,
                    dm.coins_created,
                    dm.unique_addresses as new_addresses,
                    dm.total_coin_value as value_created,
                    SUM(dm.blocks_created) OVER (ORDER BY dm.metric_date ROWS UNBOUNDED PRECEDING) as total_blocks,
                    SUM(dm.coins_created) OVER (ORDER BY dm.metric_date ROWS UNBOUNDED PRECEDING) as total_coins,
                    SUM(dm.unique_addresses) OVER (ORDER BY dm.metric_date ROWS UNBOUNDED PRECEDING) as unique_addresses,
                    SUM(dm.total_coin_value) OVER (ORDER BY dm.metric_date ROWS UNBOUNDED PRECEDING) as total_value
                  FROM daily_metrics dm
                )
                SELECT 
                  metric_date,
                  total_blocks,
                  blocks_created,
                  unique_addresses,
                  new_addresses,
                  total_coins,
                  coins_created,
                  total_value,
                  value_created
                FROM cumulative_stats
                ORDER BY metric_date DESC
                "#
            }
        };

        let formatted_query = query.replace("%d", &days_back.to_string());
        
        let rows = sqlx::query(&formatted_query)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(NetworkGrowthMetrics {
                metric_date: row.get::<String, _>("metric_date"),
                total_blocks: row.get::<i64, _>("total_blocks"),
                blocks_created: row.get::<i64, _>("blocks_created"),
                unique_addresses: row.get::<i64, _>("unique_addresses"),
                new_addresses: row.get::<i64, _>("new_addresses"),
                total_coins: row.get::<i64, _>("total_coins"),
                new_coins: row.get::<i64, _>("coins_created"),
                total_value: row.get::<i64, _>("total_value") as u64,
                value_created: row.get::<i64, _>("value_created") as u64,
            });
        }

        Ok(results)
    }

    /// Get unspent coin analysis (Chia coinset model)
    async fn unspent_coin_analysis(&self) -> GraphQLResult<UnspentCoinAnalysis> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH unspent_analysis AS (
                  SELECT 
                    c.amount,
                    c.created_block,
                    (SELECT MAX(height) FROM blocks) - c.created_block as coin_age_blocks
                  FROM coins c
                  WHERE NOT EXISTS (SELECT 1 FROM spends s WHERE s.coin_id = c.coin_id)
                )
                SELECT 
                  COUNT(*) as total_unspent_coins,
                  COALESCE(SUM(amount), 0) as total_unspent_value,
                  ROUND(AVG(amount), 2) as avg_coin_value,
                  PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY amount) as median_coin_value,
                  COUNT(CASE WHEN amount < 1 THEN 1 END) as dust_coins,
                  COUNT(CASE WHEN amount < 10000000000 THEN 1 END) as small_coins,
                  COUNT(CASE WHEN amount BETWEEN 10000000000 AND 1000000000000 THEN 1 END) as medium_coins,
                  COUNT(CASE WHEN amount > 1000000000000 THEN 1 END) as large_coins,
                  ROUND(AVG(coin_age_blocks), 2) as avg_coin_age_blocks
                FROM unspent_analysis
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH unspent_analysis AS (
                  SELECT 
                    c.amount,
                    c.created_block,
                    (SELECT MAX(height) FROM blocks) - c.created_block as coin_age_blocks
                  FROM coins c
                  WHERE NOT EXISTS (SELECT 1 FROM spends s WHERE s.coin_id = c.coin_id)
                )
                SELECT 
                  COUNT(*) as total_unspent_coins,
                  COALESCE(SUM(amount), 0) as total_unspent_value,
                  ROUND(AVG(amount), 2) as avg_coin_value,
                  NULL as median_coin_value,
                  COUNT(CASE WHEN amount < 1 THEN 1 END) as dust_coins,
                  COUNT(CASE WHEN amount < 10000000000 THEN 1 END) as small_coins,
                  COUNT(CASE WHEN amount BETWEEN 10000000000 AND 1000000000000 THEN 1 END) as medium_coins,
                  COUNT(CASE WHEN amount > 1000000000000 THEN 1 END) as large_coins,
                  ROUND(AVG(coin_age_blocks), 2) as avg_coin_age_blocks
                FROM unspent_analysis
                "#
            }
        };

        let row = sqlx::query(query)
            .fetch_one(&*self.pool)
            .await?;

        Ok(UnspentCoinAnalysis {
            total_unspent_coins: row.get::<i64, _>("total_unspent_coins"),
            total_unspent_value: row.get::<i64, _>("total_unspent_value") as u64,
            avg_coin_value: row.get::<f64, _>("avg_coin_value"),
            median_coin_value: row.get("median_coin_value"),
            dust_coins: row.get::<i64, _>("dust_coins"),
            small_coins: row.get::<i64, _>("small_coins"),
            medium_coins: row.get::<i64, _>("medium_coins"),
            large_coins: row.get::<i64, _>("large_coins"),
            avg_coin_age_blocks: row.get::<f64, _>("avg_coin_age_blocks"),
        })
    }

    /// Get throughput analysis
    async fn throughput_analysis(
        &self,
        #[graphql(default = 24)] hours_back: i32,
    ) -> GraphQLResult<ThroughputAnalysis> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH hourly_stats AS (
                  SELECT 
                    COUNT(DISTINCT b.height) as blocks_count,
                    COUNT(DISTINCT c.coin_id) as coins_count,
                    COALESCE(SUM(c.amount), 0) as value_moved,
                    COUNT(DISTINCT s.coin_id) as transactions_count
                  FROM blocks b
                  LEFT JOIN coins c ON b.height = c.created_block
                  LEFT JOIN spends s ON c.coin_id = s.coin_id AND s.spent_block = b.height
                  WHERE b.timestamp >= NOW() - INTERVAL '%d hours'
                )
                SELECT 
                  blocks_count,
                  coins_count,
                  value_moved,
                  transactions_count,
                  ROUND(transactions_count::NUMERIC / (%d * 3600), 6) as tps,
                  ROUND(coins_count::NUMERIC / (%d * 3600), 6) as coins_per_second,
                  ROUND(value_moved::NUMERIC / (%d * 3600), 2) as value_per_second,
                  ROUND(blocks_count::NUMERIC / %d, 2) as blocks_per_hour
                FROM hourly_stats
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH hourly_stats AS (
                  SELECT 
                    COUNT(DISTINCT b.height) as blocks_count,
                    COUNT(DISTINCT c.coin_id) as coins_count,
                    COALESCE(SUM(c.amount), 0) as value_moved,
                    COUNT(DISTINCT s.coin_id) as transactions_count
                  FROM blocks b
                  LEFT JOIN coins c ON b.height = c.created_block
                  LEFT JOIN spends s ON c.coin_id = s.coin_id AND s.spent_block = b.height
                  WHERE datetime(b.timestamp) >= datetime('now', '-%d hours')
                )
                SELECT 
                  blocks_count,
                  coins_count,
                  value_moved,
                  transactions_count,
                  ROUND(CAST(transactions_count AS REAL) / (%d * 3600), 6) as tps,
                  ROUND(CAST(coins_count AS REAL) / (%d * 3600), 6) as coins_per_second,
                  ROUND(CAST(value_moved AS REAL) / (%d * 3600), 2) as value_per_second,
                  ROUND(CAST(blocks_count AS REAL) / %d, 2) as blocks_per_hour
                FROM hourly_stats
                "#
            }
        };

        let formatted_query = query
            .replace("%d", &hours_back.to_string());
        
        let row = sqlx::query(&formatted_query)
            .fetch_one(&*self.pool)
            .await?;

        let tps = row.get::<f64, _>("tps");
        
        Ok(ThroughputAnalysis {
            time_period: format!("Last {} hours", hours_back),
            tps,
            coins_per_second: row.get::<f64, _>("coins_per_second"),
            value_per_second: row.get::<f64, _>("value_per_second"),
            blocks_per_hour: row.get::<f64, _>("blocks_per_hour"),
            peak_tps: tps, // Simplified - same as current TPS
            avg_tps: tps,  // Simplified - same as current TPS
        })
    }
} 