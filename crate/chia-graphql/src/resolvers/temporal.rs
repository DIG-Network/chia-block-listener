use async_graphql::*;
use sqlx::{AnyPool, Row};
use std::sync::Arc;

use crate::database::DatabaseType;
use crate::error::GraphQLResult;
use crate::schema::types::*;

/// Temporal analytics queries for time-based blockchain analysis
pub struct TemporalQueries {
    pool: Arc<AnyPool>,
    db_type: DatabaseType,
}

impl TemporalQueries {
    pub fn new(pool: Arc<AnyPool>, db_type: DatabaseType) -> Self {
        Self { pool, db_type }
    }
}

#[Object]
impl TemporalQueries {
    /// Get blockchain growth trends over time
    async fn growth_trends(
        &self,
        #[graphql(default = 30)] days_back: i32,
    ) -> GraphQLResult<Vec<BlockchainGrowthTrend>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH daily_stats AS (
                  SELECT 
                    DATE(b.timestamp) as trend_date,
                    COUNT(DISTINCT b.height) as new_blocks,
                    COUNT(DISTINCT c.coin_id) as new_coins,
                    COUNT(DISTINCT c.puzzle_hash) as new_addresses,
                    COALESCE(SUM(c.amount), 0) as value_created
                  FROM blocks b
                  LEFT JOIN coins c ON b.height = c.created_block
                  WHERE b.timestamp >= NOW() - INTERVAL '%d days'
                  GROUP BY DATE(b.timestamp)
                  ORDER BY trend_date
                ),
                cumulative_stats AS (
                  SELECT 
                    ds.trend_date,
                    SUM(ds.new_blocks) OVER (ORDER BY ds.trend_date ROWS UNBOUNDED PRECEDING) as total_blocks,
                    SUM(ds.new_coins) OVER (ORDER BY ds.trend_date ROWS UNBOUNDED PRECEDING) as total_coins,
                    ds.new_coins,
                    SUM(ds.new_addresses) OVER (ORDER BY ds.trend_date ROWS UNBOUNDED PRECEDING) as total_addresses,
                    ds.new_addresses,
                    SUM(ds.value_created) OVER (ORDER BY ds.trend_date ROWS UNBOUNDED PRECEDING) as total_value,
                    LAG(SUM(ds.new_coins) OVER (ORDER BY ds.trend_date ROWS UNBOUNDED PRECEDING)) OVER (ORDER BY ds.trend_date) as prev_total_coins
                  FROM daily_stats ds
                )
                SELECT 
                  trend_date::text,
                  total_blocks,
                  total_coins,
                  new_coins,
                  total_addresses,
                  new_addresses,
                  total_value,
                  CASE 
                    WHEN prev_total_coins > 0 AND prev_total_coins IS NOT NULL 
                    THEN ROUND(((total_coins::NUMERIC - prev_total_coins::NUMERIC) / prev_total_coins::NUMERIC) * 100, 4)
                    ELSE 0
                  END as daily_growth_rate
                FROM cumulative_stats
                ORDER BY trend_date DESC
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH daily_stats AS (
                  SELECT 
                    DATE(b.timestamp) as trend_date,
                    COUNT(DISTINCT b.height) as new_blocks,
                    COUNT(DISTINCT c.coin_id) as new_coins,
                    COUNT(DISTINCT c.puzzle_hash) as new_addresses,
                    COALESCE(SUM(c.amount), 0) as value_created
                  FROM blocks b
                  LEFT JOIN coins c ON b.height = c.created_block
                  WHERE datetime(b.timestamp) >= datetime('now', '-%d days')
                  GROUP BY DATE(b.timestamp)
                  ORDER BY trend_date
                ),
                cumulative_stats AS (
                  SELECT 
                    ds.trend_date,
                    SUM(ds.new_blocks) OVER (ORDER BY ds.trend_date ROWS UNBOUNDED PRECEDING) as total_blocks,
                    SUM(ds.new_coins) OVER (ORDER BY ds.trend_date ROWS UNBOUNDED PRECEDING) as total_coins,
                    ds.new_coins,
                    SUM(ds.new_addresses) OVER (ORDER BY ds.trend_date ROWS UNBOUNDED PRECEDING) as total_addresses,
                    ds.new_addresses,
                    SUM(ds.value_created) OVER (ORDER BY ds.trend_date ROWS UNBOUNDED PRECEDING) as total_value,
                    LAG(SUM(ds.new_coins) OVER (ORDER BY ds.trend_date ROWS UNBOUNDED PRECEDING)) OVER (ORDER BY ds.trend_date) as prev_total_coins
                  FROM daily_stats ds
                )
                SELECT 
                  trend_date,
                  total_blocks,
                  total_coins,
                  new_coins,
                  total_addresses,
                  new_addresses,
                  total_value,
                  CASE 
                    WHEN prev_total_coins > 0 AND prev_total_coins IS NOT NULL 
                    THEN ROUND(((CAST(total_coins AS REAL) - CAST(prev_total_coins AS REAL)) / CAST(prev_total_coins AS REAL)) * 100, 4)
                    ELSE 0
                  END as daily_growth_rate
                FROM cumulative_stats
                ORDER BY trend_date DESC
                "#
            }
        };

        let formatted_query = query.replace("%d", &days_back.to_string());
        
        let rows = sqlx::query(&formatted_query)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(BlockchainGrowthTrend {
                trend_date: row.get::<String, _>("trend_date"),
                total_blocks: row.get::<i64, _>("total_blocks"),
                total_coins: row.get::<i64, _>("total_coins"),
                new_coins: row.get::<i64, _>("new_coins"),
                total_addresses: row.get::<i64, _>("total_addresses"),
                new_addresses: row.get::<i64, _>("new_addresses"),
                total_value: row.get::<i64, _>("total_value") as u64,
                daily_growth_rate: row.get::<f64, _>("daily_growth_rate"),
            });
        }

        Ok(results)
    }

    /// Get activity heatmap showing when the blockchain is most active
    async fn activity_heatmap(
        &self,
        #[graphql(default = 7)] days_back: i32,
    ) -> GraphQLResult<Vec<ActivityHeatmap>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH hourly_activity AS (
                  SELECT 
                    EXTRACT(HOUR FROM b.timestamp) as hour_of_day,
                    EXTRACT(DOW FROM b.timestamp) as day_of_week,
                    COUNT(DISTINCT s.coin_id) as total_transactions,
                    COUNT(DISTINCT c.coin_id) as total_coins_moved,
                    COALESCE(SUM(c.amount), 0) as total_value_moved
                  FROM blocks b
                  LEFT JOIN coins c ON b.height = c.created_block
                  LEFT JOIN spends s ON c.coin_id = s.coin_id AND s.spent_block = b.height
                  WHERE b.timestamp >= NOW() - INTERVAL '%d days'
                  GROUP BY EXTRACT(HOUR FROM b.timestamp), EXTRACT(DOW FROM b.timestamp)
                ),
                max_activity AS (
                  SELECT MAX(total_transactions) as max_transactions FROM hourly_activity
                )
                SELECT 
                  ha.hour_of_day,
                  ha.day_of_week,
                  ha.total_transactions,
                  ha.total_coins_moved,
                  ha.total_value_moved,
                  CASE 
                    WHEN ha.total_transactions > 0 THEN ROUND(ha.total_value_moved::NUMERIC / ha.total_transactions::NUMERIC, 2)
                    ELSE 0
                  END as avg_transaction_size,
                  CASE 
                    WHEN ma.max_transactions > 0 THEN ROUND((ha.total_transactions::NUMERIC / ma.max_transactions::NUMERIC) * 100, 2)
                    ELSE 0
                  END as activity_score
                FROM hourly_activity ha, max_activity ma
                ORDER BY ha.day_of_week, ha.hour_of_day
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH hourly_activity AS (
                  SELECT 
                    CAST(strftime('%H', b.timestamp) AS INTEGER) as hour_of_day,
                    CAST(strftime('%w', b.timestamp) AS INTEGER) as day_of_week,
                    COUNT(DISTINCT s.coin_id) as total_transactions,
                    COUNT(DISTINCT c.coin_id) as total_coins_moved,
                    COALESCE(SUM(c.amount), 0) as total_value_moved
                  FROM blocks b
                  LEFT JOIN coins c ON b.height = c.created_block
                  LEFT JOIN spends s ON c.coin_id = s.coin_id AND s.spent_block = b.height
                  WHERE datetime(b.timestamp) >= datetime('now', '-%d days')
                  GROUP BY CAST(strftime('%H', b.timestamp) AS INTEGER), CAST(strftime('%w', b.timestamp) AS INTEGER)
                ),
                max_activity AS (
                  SELECT MAX(total_transactions) as max_transactions FROM hourly_activity
                )
                SELECT 
                  ha.hour_of_day,
                  ha.day_of_week,
                  ha.total_transactions,
                  ha.total_coins_moved,
                  ha.total_value_moved,
                  CASE 
                    WHEN ha.total_transactions > 0 THEN ROUND(CAST(ha.total_value_moved AS REAL) / CAST(ha.total_transactions AS REAL), 2)
                    ELSE 0
                  END as avg_transaction_size,
                  CASE 
                    WHEN ma.max_transactions > 0 THEN ROUND((CAST(ha.total_transactions AS REAL) / CAST(ma.max_transactions AS REAL)) * 100, 2)
                    ELSE 0
                  END as activity_score
                FROM hourly_activity ha, max_activity ma
                ORDER BY ha.day_of_week, ha.hour_of_day
                "#
            }
        };

        let formatted_query = query.replace("%d", &days_back.to_string());
        
        let rows = sqlx::query(&formatted_query)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(ActivityHeatmap {
                hour_of_day: row.get::<i64, _>("hour_of_day") as i32,
                day_of_week: row.get::<i64, _>("day_of_week") as i32,
                total_transactions: row.get::<i64, _>("total_transactions"),
                total_coins_moved: row.get::<i64, _>("total_coins_moved"),
                total_value_moved: row.get::<i64, _>("total_value_moved") as u64,
                avg_transaction_size: row.get::<f64, _>("avg_transaction_size"),
                activity_score: row.get::<f64, _>("activity_score"),
            });
        }

        Ok(results)
    }

    /// Get time-based coin aggregation for analyzing coin flow patterns
    async fn coin_aggregation(
        &self,
        #[graphql(default = "hour")] time_bucket: String,
        #[graphql(default = 24)] periods_back: i32,
    ) -> GraphQLResult<Vec<TimeBasedCoinAggregation>> {
        let bucket_format = match time_bucket.as_str() {
            "hour" => match self.db_type {
                DatabaseType::Postgres => "date_trunc('hour', b.timestamp)",
                DatabaseType::Sqlite => "strftime('%Y-%m-%d %H:00:00', b.timestamp)",
            },
            "day" => match self.db_type {
                DatabaseType::Postgres => "date_trunc('day', b.timestamp)",
                DatabaseType::Sqlite => "date(b.timestamp)",
            },
            _ => match self.db_type {
                DatabaseType::Postgres => "date_trunc('hour', b.timestamp)",
                DatabaseType::Sqlite => "strftime('%Y-%m-%d %H:00:00', b.timestamp)",
            },
        };

        let interval_condition = match (time_bucket.as_str(), self.db_type) {
            ("hour", DatabaseType::Postgres) => format!("b.timestamp >= NOW() - INTERVAL '{} hours'", periods_back),
            ("hour", DatabaseType::Sqlite) => format!("datetime(b.timestamp) >= datetime('now', '-{} hours')", periods_back),
            ("day", DatabaseType::Postgres) => format!("b.timestamp >= NOW() - INTERVAL '{} days'", periods_back),
            ("day", DatabaseType::Sqlite) => format!("datetime(b.timestamp) >= datetime('now', '-{} days')", periods_back),
            (_, DatabaseType::Postgres) => format!("b.timestamp >= NOW() - INTERVAL '{} hours'", periods_back),
            (_, DatabaseType::Sqlite) => format!("datetime(b.timestamp) >= datetime('now', '-{} hours')", periods_back),
        };

        let query = format!(
            r#"
            WITH time_buckets AS (
              SELECT 
                {} as time_bucket,
                COUNT(DISTINCT CASE WHEN c.coin_id IS NOT NULL THEN c.coin_id END) as coins_created,
                COUNT(DISTINCT CASE WHEN s.coin_id IS NOT NULL THEN s.coin_id END) as coins_spent,
                COALESCE(SUM(CASE WHEN c.coin_id IS NOT NULL THEN c.amount ELSE 0 END), 0) as value_created,
                COALESCE(SUM(CASE WHEN s.coin_id IS NOT NULL THEN c2.amount ELSE 0 END), 0) as value_spent,
                COUNT(DISTINCT CASE WHEN c.coin_id IS NOT NULL THEN c.puzzle_hash END) as active_addresses
              FROM blocks b
              LEFT JOIN coins c ON b.height = c.created_block
              LEFT JOIN spends s ON b.height = s.spent_block
              LEFT JOIN coins c2 ON s.coin_id = c2.coin_id
              WHERE {}
              GROUP BY {}
              ORDER BY time_bucket DESC
            )
            SELECT 
              time_bucket,
              coins_created,
              coins_spent,
              (coins_created - coins_spent) as net_coin_change,
              value_created,
              value_spent,
              (value_created - value_spent) as net_value_change,
              active_addresses
            FROM time_buckets
            "#,
            bucket_format, interval_condition, bucket_format
        );

        let rows = sqlx::query(&query)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            let value_created: i64 = row.get("value_created");
            let value_spent: i64 = row.get("value_spent");
            
            results.push(TimeBasedCoinAggregation {
                time_bucket: row.get::<String, _>("time_bucket"),
                coins_created: row.get::<i64, _>("coins_created"),
                coins_spent: row.get::<i64, _>("coins_spent"),
                net_coin_change: row.get::<i64, _>("net_coin_change"),
                value_created: value_created as u64,
                value_spent: value_spent as u64,
                net_value_change: value_created - value_spent,
                active_addresses: row.get::<i64, _>("active_addresses"),
            });
        }

        Ok(results)
    }

    /// Get seasonal patterns in blockchain activity
    async fn seasonal_patterns(
        &self,
        period_type: String, // "daily", "weekly", "monthly"
    ) -> GraphQLResult<Vec<SeasonalPattern>> {
        let (period_extract, period_name) = match (period_type.as_str(), self.db_type) {
            ("daily", DatabaseType::Postgres) => ("EXTRACT(HOUR FROM b.timestamp)", "hour_of_day"),
            ("daily", DatabaseType::Sqlite) => ("CAST(strftime('%H', b.timestamp) AS INTEGER)", "hour_of_day"),
            ("weekly", DatabaseType::Postgres) => ("EXTRACT(DOW FROM b.timestamp)", "day_of_week"),
            ("weekly", DatabaseType::Sqlite) => ("CAST(strftime('%w', b.timestamp) AS INTEGER)", "day_of_week"),
            ("monthly", DatabaseType::Postgres) => ("EXTRACT(DAY FROM b.timestamp)", "day_of_month"),
            ("monthly", DatabaseType::Sqlite) => ("CAST(strftime('%d', b.timestamp) AS INTEGER)", "day_of_month"),
            (_, DatabaseType::Postgres) => ("EXTRACT(HOUR FROM b.timestamp)", "hour_of_day"),
            (_, DatabaseType::Sqlite) => ("CAST(strftime('%H', b.timestamp) AS INTEGER)", "hour_of_day"),
        };

        let query = format!(
            r#"
            WITH period_activity AS (
              SELECT 
                {} as period_value,
                COUNT(DISTINCT b.height) as activity_count,
                COUNT(DISTINCT c.coin_id) as coins_created,
                COUNT(DISTINCT s.coin_id) as coins_spent
              FROM blocks b
              LEFT JOIN coins c ON b.height = c.created_block
              LEFT JOIN spends s ON b.height = s.spent_block
              WHERE b.timestamp >= {} - INTERVAL '30 days'
              GROUP BY {}
            ),
            pattern_stats AS (
              SELECT 
                period_value,
                activity_count,
                AVG(activity_count) OVER () as avg_activity,
                MAX(activity_count) OVER () as peak_activity,
                MIN(activity_count) OVER () as low_activity,
                CASE 
                  WHEN LAG(activity_count) OVER (ORDER BY period_value) < activity_count THEN 'increasing'
                  WHEN LAG(activity_count) OVER (ORDER BY period_value) > activity_count THEN 'decreasing'
                  ELSE 'stable'
                END as trend_direction
              FROM period_activity
            )
            SELECT 
              '{}' as period_type,
              period_value,
              avg_activity,
              peak_activity,
              low_activity,
              COALESCE(POWER(activity_count - avg_activity, 2), 0) as variance,
              trend_direction
            FROM pattern_stats
            ORDER BY period_value
            "#,
            period_extract,
            match self.db_type {
                DatabaseType::Postgres => "NOW()",
                DatabaseType::Sqlite => "datetime('now')",
            },
            period_extract,
            period_type
        );

        let rows = sqlx::query(&query)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(SeasonalPattern {
                period_type: row.get::<String, _>("period_type"),
                period_value: row.get::<i64, _>("period_value").to_string(),
                avg_activity: row.get::<f64, _>("avg_activity"),
                peak_activity: row.get::<f64, _>("peak_activity"),
                low_activity: row.get::<f64, _>("low_activity"),
                variance: row.get::<f64, _>("variance"),
                trend_direction: row.get::<String, _>("trend_direction"),
            });
        }

        Ok(results)
    }
} 