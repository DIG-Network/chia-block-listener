use async_graphql::*;
use sqlx::{AnyPool, Row};
use std::sync::Arc;

use crate::database::DatabaseType;
use crate::error::{GraphQLError, GraphQLResult};
use crate::schema::types::*;

/// CAT (Chia Asset Token) queries
pub struct CatQueries {
    pool: Arc<AnyPool>,
    db_type: DatabaseType,
}

impl CatQueries {
    pub fn new(pool: Arc<AnyPool>, db_type: DatabaseType) -> Self {
        Self { pool, db_type }
    }
}

#[Object]
impl CatQueries {
    /// Get CAT asset information by asset ID
    async fn asset_by_id(&self, asset_id: String) -> GraphQLResult<Option<CatAsset>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                "SELECT *, metadata_json::text as metadata_json_str FROM cat_assets WHERE asset_id = $1"
            }
            DatabaseType::Sqlite => {
                "SELECT *, metadata_json as metadata_json_str FROM cat_assets WHERE asset_id = ?"
            }
        };

        let row = sqlx::query(query)
            .bind(&asset_id)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(row.map(|r| CatAsset {
            asset_id: r.get("asset_id"),
            symbol: r.get("symbol"),
            name: r.get("name"),
            description: r.get("description"),
            total_supply: r.get("total_supply"),
            decimals: r.get("decimals"),
            metadata_json: r.get("metadata_json_str"),
            first_seen_block: r.get("first_seen_block"),
            created_at: r.get::<String, _>("created_at"),
        }))
    }

    /// Get all CAT balances for a specific owner
    async fn balances_by_owner(
        &self,
        puzzle_hash: String,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<CatBalance>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                  cb.asset_id,
                  COALESCE(ca.symbol, '') as symbol,
                  COALESCE(ca.name, '') as asset_name,
                  cb.owner_puzzle_hash_hex,
                  cb.coin_count,
                  cb.total_amount,
                  cb.total_balance,
                  cb.min_coin_amount,
                  cb.max_coin_amount,
                  cb.avg_coin_amount,
                  cb.last_activity_block,
                  COALESCE(ca.decimals, 3) as decimals
                FROM cat_balances cb
                LEFT JOIN cat_assets ca ON cb.asset_id = ca.asset_id
                WHERE cb.owner_puzzle_hash_hex = $1
                ORDER BY cb.total_amount DESC
                LIMIT $2 OFFSET $3
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                  cb.asset_id,
                  COALESCE(ca.symbol, '') as symbol,
                  COALESCE(ca.name, '') as asset_name,
                  cb.owner_puzzle_hash_hex,
                  cb.coin_count,
                  cb.total_amount,
                  cb.total_balance,
                  cb.min_coin_amount,
                  cb.max_coin_amount,
                  cb.avg_coin_amount,
                  cb.last_activity_block,
                  COALESCE(ca.decimals, 3) as decimals
                FROM cat_balances cb
                LEFT JOIN cat_assets ca ON cb.asset_id = ca.asset_id
                WHERE cb.owner_puzzle_hash_hex = ?
                ORDER BY cb.total_amount DESC
                LIMIT ? OFFSET ?
                "#
            }
        };

        let offset = (page - 1) * page_size;
        
        let rows = sqlx::query(query)
            .bind(&puzzle_hash)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(CatBalance {
                asset_id: row.get("asset_id"),
                symbol: row.get::<Option<String>, _>("symbol"),
                asset_name: row.get::<Option<String>, _>("asset_name"),
                owner_puzzle_hash_hex: row.get("owner_puzzle_hash_hex"),
                coin_count: row.get::<i64, _>("coin_count"),
                total_amount: row.get::<i64, _>("total_amount") as u64,
                total_balance: row.get("total_balance"),
                min_coin_amount: row.get::<i64, _>("min_coin_amount") as u64,
                max_coin_amount: row.get::<i64, _>("max_coin_amount") as u64,
                avg_coin_amount: row.get::<i64, _>("avg_coin_amount") as u64,
                last_activity_block: row.get("last_activity_block"),
                decimals: row.get("decimals"),
            });
        }

        Ok(results)
    }

    /// Get top holders for a specific CAT asset
    async fn holders_by_asset(
        &self,
        asset_id: String,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<CatHolder>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                  cb.owner_puzzle_hash_hex,
                  cb.coin_count,
                  cb.total_amount,
                  cb.total_balance,
                  cb.min_coin_amount,
                  cb.max_coin_amount,
                  cb.avg_coin_amount,
                  cb.last_activity_block
                FROM cat_balances cb
                WHERE cb.asset_id = $1
                ORDER BY cb.total_amount DESC
                LIMIT $2 OFFSET $3
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                  cb.owner_puzzle_hash_hex,
                  cb.coin_count,
                  cb.total_amount,
                  cb.total_balance,
                  cb.min_coin_amount,
                  cb.max_coin_amount,
                  cb.avg_coin_amount,
                  cb.last_activity_block
                FROM cat_balances cb
                WHERE cb.asset_id = ?
                ORDER BY cb.total_amount DESC
                LIMIT ? OFFSET ?
                "#
            }
        };

        let offset = (page - 1) * page_size;
        
        let rows = sqlx::query(query)
            .bind(&asset_id)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(CatHolder {
                owner_puzzle_hash_hex: row.get("owner_puzzle_hash_hex"),
                coin_count: row.get::<i64, _>("coin_count"),
                total_amount: row.get::<i64, _>("total_amount") as u64,
                total_balance: row.get("total_balance"),
                min_coin_amount: row.get::<i64, _>("min_coin_amount") as u64,
                max_coin_amount: row.get::<i64, _>("max_coin_amount") as u64,
                avg_coin_amount: row.get::<i64, _>("avg_coin_amount") as u64,
                last_activity_block: row.get("last_activity_block"),
            });
        }

        Ok(results)
    }

    /// Get summary of all CAT assets
    async fn asset_summary(
        &self,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<CatAssetSummary>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                  cb.asset_id,
                  COALESCE(ca.symbol, '') as symbol,
                  COALESCE(ca.name, '') as asset_name,
                  COUNT(*) as holder_count,
                  SUM(cb.total_amount) as total_circulating_supply,
                  SUM(cb.total_balance) as total_circulating_balance,
                  MAX(cb.total_amount) as top_holder_amount,
                  ROUND(AVG(cb.total_balance)::numeric, COALESCE(MAX(ca.decimals), 3)) as avg_holder_balance,
                  MAX(ca.decimals) as decimals
                FROM cat_balances cb
                LEFT JOIN cat_assets ca ON cb.asset_id = ca.asset_id
                GROUP BY cb.asset_id, ca.symbol, ca.name
                ORDER BY total_circulating_supply DESC
                LIMIT $1 OFFSET $2
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                  cb.asset_id,
                  COALESCE(ca.symbol, '') as symbol,
                  COALESCE(ca.name, '') as asset_name,
                  COUNT(*) as holder_count,
                  SUM(cb.total_amount) as total_circulating_supply,
                  SUM(cb.total_balance) as total_circulating_balance,
                  MAX(cb.total_amount) as top_holder_amount,
                  ROUND(AVG(cb.total_balance), COALESCE(MAX(ca.decimals), 3)) as avg_holder_balance,
                  MAX(ca.decimals) as decimals
                FROM cat_balances cb
                LEFT JOIN cat_assets ca ON cb.asset_id = ca.asset_id
                GROUP BY cb.asset_id, ca.symbol, ca.name
                ORDER BY total_circulating_supply DESC
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
            results.push(CatAssetSummary {
                asset_id: row.get("asset_id"),
                symbol: row.get::<Option<String>, _>("symbol"),
                asset_name: row.get::<Option<String>, _>("asset_name"),
                holder_count: row.get::<i64, _>("holder_count"),
                total_circulating_supply: row.get::<i64, _>("total_circulating_supply") as u64,
                total_circulating_balance: row.get("total_circulating_balance"),
                top_holder_amount: row.get::<i64, _>("top_holder_amount") as u64,
                avg_holder_balance: row.get("avg_holder_balance"),
                decimals: row.get("decimals"),
            });
        }

        Ok(results)
    }

    /// Get comprehensive token analytics
    async fn token_analytics(&self, asset_id: String) -> GraphQLResult<Option<TokenAnalytics>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH token_stats AS (
                  SELECT 
                    c.asset_id,
                    ca.symbol,
                    ca.name as token_name,
                    ca.decimals,
                    ca.total_supply,
                    COUNT(DISTINCT c.coin_id) as total_coins,
                    COUNT(DISTINCT c.owner_puzzle_hash) as unique_holders,
                    COUNT(DISTINCT CASE WHEN EXISTS(SELECT 1 FROM spends WHERE coin_id = c.coin_id) THEN NULL ELSE c.owner_puzzle_hash END) as current_holders,
                    SUM(c.amount) as total_circulating,
                    SUM(CASE WHEN NOT EXISTS(SELECT 1 FROM spends WHERE coin_id = c.coin_id) THEN c.amount ELSE 0 END) as current_supply,
                    MIN(c.created_block) as first_mint_block,
                    MAX(c.created_block) as last_activity_block,
                    AVG(c.amount) as avg_coin_amount,
                    MIN(c.amount) as min_coin_amount,
                    MAX(c.amount) as max_coin_amount
                  FROM cats c
                  LEFT JOIN cat_assets ca ON c.asset_id = ca.asset_id
                  WHERE c.asset_id = $1
                  GROUP BY c.asset_id, ca.symbol, ca.name, ca.decimals, ca.total_supply
                ),
                current_height AS (
                  SELECT MAX(height) as max_height FROM blocks
                )
                SELECT 
                  ts.*,
                  (ch.max_height - ts.last_activity_block) as blocks_since_last_activity,
                  CASE 
                    WHEN ts.total_supply > 0 THEN ROUND((ts.current_supply::NUMERIC / ts.total_supply::NUMERIC) * 100, 2)
                    ELSE 0
                  END as supply_utilization_percent,
                  CASE 
                    WHEN ts.total_coins > 0 THEN ROUND((ts.unique_holders::NUMERIC / ts.total_coins::NUMERIC) * 100, 2)
                    ELSE 0
                  END as holder_coin_ratio
                FROM token_stats ts, current_height ch
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH token_stats AS (
                  SELECT 
                    c.asset_id,
                    ca.symbol,
                    ca.name as token_name,
                    ca.decimals,
                    ca.total_supply,
                    COUNT(DISTINCT c.coin_id) as total_coins,
                    COUNT(DISTINCT c.owner_puzzle_hash) as unique_holders,
                    COUNT(DISTINCT CASE WHEN EXISTS(SELECT 1 FROM spends WHERE coin_id = c.coin_id) THEN NULL ELSE c.owner_puzzle_hash END) as current_holders,
                    SUM(c.amount) as total_circulating,
                    SUM(CASE WHEN NOT EXISTS(SELECT 1 FROM spends WHERE coin_id = c.coin_id) THEN c.amount ELSE 0 END) as current_supply,
                    MIN(c.created_block) as first_mint_block,
                    MAX(c.created_block) as last_activity_block,
                    AVG(c.amount) as avg_coin_amount,
                    MIN(c.amount) as min_coin_amount,
                    MAX(c.amount) as max_coin_amount
                  FROM cats c
                  LEFT JOIN cat_assets ca ON c.asset_id = ca.asset_id
                  WHERE c.asset_id = ?
                  GROUP BY c.asset_id, ca.symbol, ca.name, ca.decimals, ca.total_supply
                ),
                current_height AS (
                  SELECT MAX(height) as max_height FROM blocks
                )
                SELECT 
                  ts.*,
                  (ch.max_height - ts.last_activity_block) as blocks_since_last_activity,
                  CASE 
                    WHEN ts.total_supply > 0 THEN ROUND((CAST(ts.current_supply AS REAL) / CAST(ts.total_supply AS REAL)) * 100, 2)
                    ELSE 0
                  END as supply_utilization_percent,
                  CASE 
                    WHEN ts.total_coins > 0 THEN ROUND((CAST(ts.unique_holders AS REAL) / CAST(ts.total_coins AS REAL)) * 100, 2)
                    ELSE 0
                  END as holder_coin_ratio
                FROM token_stats ts, current_height ch
                "#
            }
        };

        let row = sqlx::query(query)
            .bind(&asset_id)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(row.map(|r| TokenAnalytics {
            asset_id: r.get("asset_id"),
            symbol: r.get("symbol"),
            token_name: r.get("token_name"),
            decimals: r.get("decimals"),
            total_supply: r.get("total_supply"),
            total_coins: r.get::<i64, _>("total_coins"),
            unique_holders: r.get::<i64, _>("unique_holders"),
            current_holders: r.get::<i64, _>("current_holders"),
            total_circulating: r.get::<i64, _>("total_circulating") as u64,
            current_supply: r.get::<i64, _>("current_supply") as u64,
            first_mint_block: r.get::<i64, _>("first_mint_block"),
            last_activity_block: r.get::<i64, _>("last_activity_block"),
            blocks_since_last_activity: r.get::<i64, _>("blocks_since_last_activity"),
            avg_coin_amount: r.get::<i64, _>("avg_coin_amount") as u64,
            min_coin_amount: r.get::<i64, _>("min_coin_amount") as u64,
            max_coin_amount: r.get::<i64, _>("max_coin_amount") as u64,
            supply_utilization_percent: r.get("supply_utilization_percent"),
            holder_coin_ratio: r.get("holder_coin_ratio"),
        }))
    }

    /// Get distribution analysis for a CAT token
    async fn distribution_analysis(
        &self,
        asset_id: String,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<TokenDistribution>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH token_total AS (
                  SELECT SUM(total_amount) as total_supply
                  FROM cat_balances
                  WHERE asset_id = $1
                ),
                ranked_holders AS (
                  SELECT 
                    cb.owner_puzzle_hash_hex as holder_address,
                    cb.coin_count as coins_held,
                    cb.total_amount as total_balance,
                    ROUND((cb.total_amount::NUMERIC / tt.total_supply::NUMERIC) * 100, 4) as balance_percentage,
                    RANK() OVER (ORDER BY cb.total_amount DESC) as wealth_rank,
                    cb.last_activity_block as last_acquired,
                    (cb.total_amount / cb.coin_count) as avg_coin_size,
                    CASE 
                      WHEN ROUND((cb.total_amount::NUMERIC / tt.total_supply::NUMERIC) * 100, 4) >= 10 THEN 'whale'
                      WHEN ROUND((cb.total_amount::NUMERIC / tt.total_supply::NUMERIC) * 100, 4) >= 1 THEN 'large_holder'
                      WHEN ROUND((cb.total_amount::NUMERIC / tt.total_supply::NUMERIC) * 100, 4) >= 0.1 THEN 'medium_holder'
                      ELSE 'small_holder'
                    END as holder_category
                  FROM cat_balances cb, token_total tt
                  WHERE cb.asset_id = $1
                )
                SELECT 
                  holder_address,
                  coins_held,
                  total_balance,
                  balance_percentage,
                  wealth_rank,
                  0 as first_acquired,
                  last_acquired,
                  avg_coin_size,
                  holder_category
                FROM ranked_holders
                ORDER BY wealth_rank
                LIMIT $2 OFFSET $3
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH token_total AS (
                  SELECT SUM(total_amount) as total_supply
                  FROM cat_balances
                  WHERE asset_id = ?
                ),
                ranked_holders AS (
                  SELECT 
                    cb.owner_puzzle_hash_hex as holder_address,
                    cb.coin_count as coins_held,
                    cb.total_amount as total_balance,
                    ROUND((CAST(cb.total_amount AS REAL) / CAST(tt.total_supply AS REAL)) * 100, 4) as balance_percentage,
                    RANK() OVER (ORDER BY cb.total_amount DESC) as wealth_rank,
                    cb.last_activity_block as last_acquired,
                    (cb.total_amount / cb.coin_count) as avg_coin_size,
                    CASE 
                      WHEN ROUND((CAST(cb.total_amount AS REAL) / CAST(tt.total_supply AS REAL)) * 100, 4) >= 10 THEN 'whale'
                      WHEN ROUND((CAST(cb.total_amount AS REAL) / CAST(tt.total_supply AS REAL)) * 100, 4) >= 1 THEN 'large_holder'
                      WHEN ROUND((CAST(cb.total_amount AS REAL) / CAST(tt.total_supply AS REAL)) * 100, 4) >= 0.1 THEN 'medium_holder'
                      ELSE 'small_holder'
                    END as holder_category
                  FROM cat_balances cb, token_total tt
                  WHERE cb.asset_id = ?
                )
                SELECT 
                  holder_address,
                  coins_held,
                  total_balance,
                  balance_percentage,
                  wealth_rank,
                  0 as first_acquired,
                  last_acquired,
                  avg_coin_size,
                  holder_category
                FROM ranked_holders
                ORDER BY wealth_rank
                LIMIT ? OFFSET ?
                "#
            }
        };

        let offset = (page - 1) * page_size;
        
        let rows = match self.db_type {
            DatabaseType::Postgres => {
                sqlx::query(query)
                    .bind(&asset_id)
                    .bind(page_size)
                    .bind(offset)
                    .fetch_all(&*self.pool)
                    .await?
            }
            DatabaseType::Sqlite => {
                sqlx::query(query)
                    .bind(&asset_id)
                    .bind(&asset_id)
                    .bind(page_size)
                    .bind(offset)
                    .fetch_all(&*self.pool)
                    .await?
            }
        };

        let mut results = Vec::new();
        for row in rows {
            results.push(TokenDistribution {
                holder_address: row.get("holder_address"),
                coins_held: row.get::<i64, _>("coins_held"),
                total_balance: row.get::<i64, _>("total_balance") as u64,
                balance_percentage: row.get("balance_percentage"),
                wealth_rank: row.get::<i64, _>("wealth_rank"),
                first_acquired: row.get::<i64, _>("first_acquired"),
                last_acquired: row.get::<i64, _>("last_acquired"),
                avg_coin_size: row.get::<i64, _>("avg_coin_size") as u64,
                holder_category: row.get("holder_category"),
            });
        }

        Ok(results)
    }

    /// Get trading velocity for a CAT token
    async fn trading_velocity(&self, asset_id: String, days_back: i32) -> GraphQLResult<Option<TokenVelocity>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH current_height AS (
                  SELECT MAX(height) as max_height FROM blocks
                ),
                lookback_block AS (
                  SELECT MAX(height) as start_block 
                  FROM blocks 
                  WHERE timestamp >= NOW() - INTERVAL '%d days'
                ),
                velocity_stats AS (
                  SELECT 
                    c.asset_id,
                    COUNT(DISTINCT s.coin_id) as coins_spent,
                    COUNT(DISTINCT s.spent_block) as active_blocks,
                    SUM(c.amount) as total_amount_moved,
                    COUNT(DISTINCT CASE WHEN s.spent_block >= lb.start_block THEN s.coin_id END) as transfer_coins,
                    COUNT(DISTINCT CASE WHEN s.spent_block >= lb.start_block THEN encode(c.owner_puzzle_hash, 'hex') END) as unique_spenders,
                    (SELECT SUM(amount) FROM cats WHERE asset_id = c.asset_id AND NOT EXISTS(SELECT 1 FROM spends WHERE coin_id = cats.coin_id)) as current_supply,
                    (SELECT COUNT(*) FROM cats WHERE asset_id = c.asset_id) as total_coins
                  FROM cats c
                  JOIN spends s ON c.coin_id = s.coin_id
                  CROSS JOIN lookback_block lb
                  WHERE c.asset_id = $1
                    AND s.spent_block >= lb.start_block
                  GROUP BY c.asset_id, lb.start_block
                ),
                hold_times AS (
                  SELECT AVG(s.spent_block - c.created_block) as avg_hold_time_blocks
                  FROM cats c
                  JOIN spends s ON c.coin_id = s.coin_id
                  WHERE c.asset_id = $1
                )
                SELECT 
                  vs.*,
                  ht.avg_hold_time_blocks,
                  CASE 
                    WHEN vs.current_supply > 0 THEN ROUND((vs.total_amount_moved::NUMERIC / vs.current_supply::NUMERIC) * 100, 2)
                    ELSE 0
                  END as velocity_percentage,
                  CASE 
                    WHEN vs.coins_spent > 0 THEN ROUND((vs.transfer_coins::NUMERIC / vs.coins_spent::NUMERIC) * 100, 2)
                    ELSE 0
                  END as transfer_ratio_percent
                FROM velocity_stats vs, hold_times ht
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH current_height AS (
                  SELECT MAX(height) as max_height FROM blocks
                ),
                lookback_block AS (
                  SELECT MAX(height) as start_block 
                  FROM blocks 
                  WHERE datetime(timestamp) >= datetime('now', '-%d days')
                ),
                velocity_stats AS (
                  SELECT 
                    c.asset_id,
                    COUNT(DISTINCT s.coin_id) as coins_spent,
                    COUNT(DISTINCT s.spent_block) as active_blocks,
                    SUM(c.amount) as total_amount_moved,
                    COUNT(DISTINCT CASE WHEN s.spent_block >= lb.start_block THEN s.coin_id END) as transfer_coins,
                    COUNT(DISTINCT CASE WHEN s.spent_block >= lb.start_block THEN hex(c.owner_puzzle_hash) END) as unique_spenders,
                    (SELECT SUM(amount) FROM cats WHERE asset_id = c.asset_id AND NOT EXISTS(SELECT 1 FROM spends WHERE coin_id = cats.coin_id)) as current_supply,
                    (SELECT COUNT(*) FROM cats WHERE asset_id = c.asset_id) as total_coins
                  FROM cats c
                  JOIN spends s ON c.coin_id = s.coin_id
                  CROSS JOIN lookback_block lb
                  WHERE c.asset_id = ?
                    AND s.spent_block >= lb.start_block
                  GROUP BY c.asset_id, lb.start_block
                ),
                hold_times AS (
                  SELECT AVG(s.spent_block - c.created_block) as avg_hold_time_blocks
                  FROM cats c
                  JOIN spends s ON c.coin_id = s.coin_id
                  WHERE c.asset_id = ?
                )
                SELECT 
                  vs.*,
                  ht.avg_hold_time_blocks,
                  CASE 
                    WHEN vs.current_supply > 0 THEN ROUND((CAST(vs.total_amount_moved AS REAL) / CAST(vs.current_supply AS REAL)) * 100, 2)
                    ELSE 0
                  END as velocity_percentage,
                  CASE 
                    WHEN vs.coins_spent > 0 THEN ROUND((CAST(vs.transfer_coins AS REAL) / CAST(vs.coins_spent AS REAL)) * 100, 2)
                    ELSE 0
                  END as transfer_ratio_percent
                FROM velocity_stats vs, hold_times ht
                "#
            }
        };

        let formatted_query = query.replace("%d", &days_back.to_string());
        
        let row = match self.db_type {
            DatabaseType::Postgres => {
                sqlx::query(&formatted_query)
                    .bind(&asset_id)
                    .fetch_optional(&*self.pool)
                    .await?
            }
            DatabaseType::Sqlite => {
                sqlx::query(&formatted_query)
                    .bind(&asset_id)
                    .bind(&asset_id)
                    .fetch_optional(&*self.pool)
                    .await?
            }
        };

        Ok(row.map(|r| TokenVelocity {
            asset_id: r.get("asset_id"),
            days_analyzed: days_back,
            coins_spent: r.get::<i64, _>("coins_spent"),
            active_blocks: r.get::<i64, _>("active_blocks"),
            total_amount_moved: r.get::<i64, _>("total_amount_moved") as u64,
            transfer_coins: r.get::<i64, _>("transfer_coins"),
            avg_hold_time_blocks: r.get("avg_hold_time_blocks"),
            unique_spenders: r.get::<i64, _>("unique_spenders"),
            current_supply: r.get::<i64, _>("current_supply") as u64,
            total_coins: r.get::<i64, _>("total_coins"),
            velocity_percentage: r.get("velocity_percentage"),
            transfer_ratio_percent: r.get("transfer_ratio_percent"),
        }))
    }

    /// Get holder behavior analysis
    async fn holder_behavior(
        &self,
        asset_id: String,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<HolderBehavior>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH holder_stats AS (
                  SELECT 
                    encode(c.owner_puzzle_hash, 'hex') as holder_address,
                    COUNT(DISTINCT c.coin_id) as total_coins_received,
                    COUNT(DISTINCT s.coin_id) as total_coins_spent,
                    SUM(c.amount) as total_received,
                    SUM(CASE WHEN s.coin_id IS NOT NULL THEN c.amount ELSE 0 END) as total_spent,
                    SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance,
                    MIN(c.created_block) as first_activity,
                    MAX(COALESCE(s.spent_block, c.created_block)) as last_activity,
                    AVG(CASE WHEN s.spent_block IS NOT NULL THEN s.spent_block - c.created_block END) as avg_hold_time,
                    COUNT(DISTINCT c.created_block) as active_blocks_receiving,
                    COUNT(DISTINCT s.spent_block) as active_blocks_spending
                  FROM cats c
                  LEFT JOIN spends s ON c.coin_id = s.coin_id
                  WHERE c.asset_id = $1
                  GROUP BY c.owner_puzzle_hash
                ),
                current_height AS (
                  SELECT MAX(height) as max_height FROM blocks
                )
                SELECT 
                  hs.*,
                  (ch.max_height - hs.last_activity) as blocks_since_last_activity,
                  CASE 
                    WHEN hs.total_coins_spent = 0 THEN 'hodler'
                    WHEN hs.total_coins_spent = hs.total_coins_received THEN 'trader'
                    WHEN hs.current_balance > hs.total_spent THEN 'accumulator'
                    ELSE 'mixed'
                  END as behavior_type,
                  CASE 
                    WHEN hs.total_received > 0 THEN ROUND((hs.total_spent::NUMERIC / hs.total_received::NUMERIC) * 100, 2)
                    ELSE 0
                  END as spend_ratio_percent,
                  CASE 
                    WHEN hs.avg_hold_time < 100 THEN 'short_term'
                    WHEN hs.avg_hold_time < 1000 THEN 'medium_term'
                    ELSE 'long_term'
                  END as hold_strategy
                FROM holder_stats hs, current_height ch
                ORDER BY hs.current_balance DESC
                LIMIT $2 OFFSET $3
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH holder_stats AS (
                  SELECT 
                    hex(c.owner_puzzle_hash) as holder_address,
                    COUNT(DISTINCT c.coin_id) as total_coins_received,
                    COUNT(DISTINCT s.coin_id) as total_coins_spent,
                    SUM(c.amount) as total_received,
                    SUM(CASE WHEN s.coin_id IS NOT NULL THEN c.amount ELSE 0 END) as total_spent,
                    SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance,
                    MIN(c.created_block) as first_activity,
                    MAX(COALESCE(s.spent_block, c.created_block)) as last_activity,
                    AVG(CASE WHEN s.spent_block IS NOT NULL THEN s.spent_block - c.created_block END) as avg_hold_time,
                    COUNT(DISTINCT c.created_block) as active_blocks_receiving,
                    COUNT(DISTINCT s.spent_block) as active_blocks_spending
                  FROM cats c
                  LEFT JOIN spends s ON c.coin_id = s.coin_id
                  WHERE c.asset_id = ?
                  GROUP BY c.owner_puzzle_hash
                ),
                current_height AS (
                  SELECT MAX(height) as max_height FROM blocks
                )
                SELECT 
                  hs.*,
                  (ch.max_height - hs.last_activity) as blocks_since_last_activity,
                  CASE 
                    WHEN hs.total_coins_spent = 0 THEN 'hodler'
                    WHEN hs.total_coins_spent = hs.total_coins_received THEN 'trader'
                    WHEN hs.current_balance > hs.total_spent THEN 'accumulator'
                    ELSE 'mixed'
                  END as behavior_type,
                  CASE 
                    WHEN hs.total_received > 0 THEN ROUND((CAST(hs.total_spent AS REAL) / CAST(hs.total_received AS REAL)) * 100, 2)
                    ELSE 0
                  END as spend_ratio_percent,
                  CASE 
                    WHEN hs.avg_hold_time < 100 THEN 'short_term'
                    WHEN hs.avg_hold_time < 1000 THEN 'medium_term'
                    ELSE 'long_term'
                  END as hold_strategy
                FROM holder_stats hs, current_height ch
                ORDER BY hs.current_balance DESC
                LIMIT ? OFFSET ?
                "#
            }
        };

        let offset = (page - 1) * page_size;
        
        let rows = sqlx::query(query)
            .bind(&asset_id)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(HolderBehavior {
                holder_address: row.get("holder_address"),
                total_coins_received: row.get::<i64, _>("total_coins_received"),
                total_coins_spent: row.get::<i64, _>("total_coins_spent"),
                total_received: row.get::<i64, _>("total_received") as u64,
                total_spent: row.get::<i64, _>("total_spent") as u64,
                current_balance: row.get::<i64, _>("current_balance") as u64,
                first_activity: row.get::<i64, _>("first_activity"),
                last_activity: row.get::<i64, _>("last_activity"),
                blocks_since_last_activity: row.get::<i64, _>("blocks_since_last_activity"),
                avg_hold_time: row.get("avg_hold_time"),
                active_blocks_receiving: row.get::<i64, _>("active_blocks_receiving"),
                active_blocks_spending: row.get::<i64, _>("active_blocks_spending"),
                behavior_type: row.get("behavior_type"),
                spend_ratio_percent: row.get("spend_ratio_percent"),
                hold_strategy: row.get("hold_strategy"),
            });
        }

        Ok(results)
    }

    /// Compare multiple tokens
    async fn token_comparison(
        &self,
        asset_ids: Vec<String>,
    ) -> GraphQLResult<Vec<TokenComparison>> {
        if asset_ids.is_empty() {
            return Ok(vec![]);
        }

        let placeholders = match self.db_type {
            DatabaseType::Postgres => {
                (1..=asset_ids.len())
                    .map(|i| format!("${}", i))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
            DatabaseType::Sqlite => {
                vec!["?"; asset_ids.len()].join(", ")
            }
        };

        let query = match self.db_type {
            DatabaseType::Postgres => {
                format!(r#"
                WITH token_metrics AS (
                  SELECT 
                    c.asset_id,
                    ca.symbol,
                    ca.name as token_name,
                    ca.decimals,
                    COUNT(DISTINCT c.coin_id) as total_coins,
                    COUNT(DISTINCT c.owner_puzzle_hash) as unique_holders,
                    SUM(c.amount) as total_circulating,
                    SUM(CASE WHEN NOT EXISTS(SELECT 1 FROM spends WHERE coin_id = c.coin_id) THEN c.amount ELSE 0 END) as current_supply,
                    MIN(c.created_block) as first_activity,
                    MAX(c.created_block) as last_activity,
                    COUNT(DISTINCT s.coin_id) as coins_spent,
                    COUNT(DISTINCT s.spent_block) as active_trading_blocks
                  FROM cats c
                  LEFT JOIN cat_assets ca ON c.asset_id = ca.asset_id
                  LEFT JOIN spends s ON c.coin_id = s.coin_id
                  WHERE c.asset_id IN ({})
                  GROUP BY c.asset_id, ca.symbol, ca.name, ca.decimals
                )
                SELECT 
                  tm.*,
                  -- Distribution score: unique holders / sqrt(total coins)
                  CASE 
                    WHEN tm.total_coins > 0 THEN ROUND((tm.unique_holders::NUMERIC / SQRT(tm.total_coins)::NUMERIC) * 100, 2)
                    ELSE 0
                  END as distribution_score,
                  -- Activity score: trading blocks / age in blocks
                  CASE 
                    WHEN (tm.last_activity - tm.first_activity) > 0 
                    THEN ROUND((tm.active_trading_blocks::NUMERIC / (tm.last_activity - tm.first_activity)::NUMERIC) * 100, 2)
                    ELSE 0
                  END as activity_score,
                  RANK() OVER (ORDER BY tm.current_supply DESC) as market_cap_rank,
                  RANK() OVER (ORDER BY tm.unique_holders DESC) as holder_rank,
                  RANK() OVER (ORDER BY tm.active_trading_blocks DESC) as activity_rank
                FROM token_metrics tm
                ORDER BY tm.current_supply DESC
                "#, placeholders)
            }
            DatabaseType::Sqlite => {
                format!(r#"
                WITH token_metrics AS (
                  SELECT 
                    c.asset_id,
                    ca.symbol,
                    ca.name as token_name,
                    ca.decimals,
                    COUNT(DISTINCT c.coin_id) as total_coins,
                    COUNT(DISTINCT c.owner_puzzle_hash) as unique_holders,
                    SUM(c.amount) as total_circulating,
                    SUM(CASE WHEN NOT EXISTS(SELECT 1 FROM spends WHERE coin_id = c.coin_id) THEN c.amount ELSE 0 END) as current_supply,
                    MIN(c.created_block) as first_activity,
                    MAX(c.created_block) as last_activity,
                    COUNT(DISTINCT s.coin_id) as coins_spent,
                    COUNT(DISTINCT s.spent_block) as active_trading_blocks
                  FROM cats c
                  LEFT JOIN cat_assets ca ON c.asset_id = ca.asset_id
                  LEFT JOIN spends s ON c.coin_id = s.coin_id
                  WHERE c.asset_id IN ({})
                  GROUP BY c.asset_id, ca.symbol, ca.name, ca.decimals
                )
                SELECT 
                  tm.*,
                  -- Distribution score: unique holders / sqrt(total coins)
                  CASE 
                    WHEN tm.total_coins > 0 THEN ROUND((CAST(tm.unique_holders AS REAL) / SQRT(CAST(tm.total_coins AS REAL))) * 100, 2)
                    ELSE 0
                  END as distribution_score,
                  -- Activity score: trading blocks / age in blocks
                  CASE 
                    WHEN (tm.last_activity - tm.first_activity) > 0 
                    THEN ROUND((CAST(tm.active_trading_blocks AS REAL) / CAST((tm.last_activity - tm.first_activity) AS REAL)) * 100, 2)
                    ELSE 0
                  END as activity_score,
                  RANK() OVER (ORDER BY tm.current_supply DESC) as market_cap_rank,
                  RANK() OVER (ORDER BY tm.unique_holders DESC) as holder_rank,
                  RANK() OVER (ORDER BY tm.active_trading_blocks DESC) as activity_rank
                FROM token_metrics tm
                ORDER BY tm.current_supply DESC
                "#, placeholders)
            }
        };

        let mut query_builder = sqlx::query(&query);
        for asset_id in &asset_ids {
            query_builder = query_builder.bind(asset_id);
        }

        let rows = query_builder.fetch_all(&*self.pool).await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(TokenComparison {
                asset_id: row.get("asset_id"),
                symbol: row.get("symbol"),
                token_name: row.get("token_name"),
                decimals: row.get("decimals"),
                total_coins: row.get::<i64, _>("total_coins"),
                unique_holders: row.get::<i64, _>("unique_holders"),
                total_circulating: row.get::<i64, _>("total_circulating") as u64,
                current_supply: row.get::<i64, _>("current_supply") as u64,
                first_activity: row.get::<i64, _>("first_activity"),
                last_activity: row.get::<i64, _>("last_activity"),
                coins_spent: row.get::<i64, _>("coins_spent"),
                active_trading_blocks: row.get::<i64, _>("active_trading_blocks"),
                distribution_score: row.get("distribution_score"),
                activity_score: row.get("activity_score"),
                market_cap_rank: row.get::<i64, _>("market_cap_rank"),
                holder_rank: row.get::<i64, _>("holder_rank"),
                activity_rank: row.get::<i64, _>("activity_rank"),
            });
        }

        Ok(results)
    }

    /// Get liquidity analysis for a token
    async fn liquidity_analysis(&self, asset_id: String) -> GraphQLResult<Option<LiquidityAnalysis>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH coin_stats AS (
                  SELECT 
                    c.asset_id,
                    COUNT(*) as total_coins,
                    COUNT(CASE WHEN c.amount >= 1000000000 THEN 1 END) as large_coins,
                    COUNT(CASE WHEN c.amount < 100000 THEN 1 END) as dust_coins,
                    COUNT(CASE WHEN NOT EXISTS(SELECT 1 FROM spends WHERE coin_id = c.coin_id) THEN 1 END) as unspent_coins,
                    SUM(c.amount) as total_amount,
                    SUM(CASE WHEN NOT EXISTS(SELECT 1 FROM spends WHERE coin_id = c.coin_id) THEN c.amount ELSE 0 END) as liquid_amount,
                    AVG(c.amount) as avg_coin_size,
                    STDDEV(c.amount) as coin_size_stddev,
                    COUNT(DISTINCT c.owner_puzzle_hash) as unique_addresses,
                    COUNT(DISTINCT CASE WHEN NOT EXISTS(SELECT 1 FROM spends WHERE coin_id = c.coin_id) THEN c.owner_puzzle_hash END) as active_addresses
                  FROM cats c
                  WHERE c.asset_id = $1
                  GROUP BY c.asset_id
                ),
                recent_activity AS (
                  SELECT 
                    COUNT(DISTINCT s.coin_id) as coins_spent_7d,
                    COUNT(DISTINCT s.spent_block) as active_blocks_7d,
                    SUM(c.amount) as amount_moved_7d
                  FROM cats c
                  JOIN spends s ON c.coin_id = s.coin_id
                  JOIN blocks b ON s.spent_block = b.height
                  WHERE c.asset_id = $1
                    AND b.timestamp >= NOW() - INTERVAL '7 days'
                ),
                percentiles AS (
                  SELECT 
                    PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY amount) as median_coin_size
                  FROM cats
                  WHERE asset_id = $1
                )
                SELECT 
                  cs.*,
                  p.median_coin_size,
                  ra.coins_spent_7d,
                  ra.active_blocks_7d,
                  ra.amount_moved_7d,
                  CASE 
                    WHEN cs.total_coins > 0 THEN ROUND((cs.large_coins::NUMERIC / cs.total_coins::NUMERIC) * 100, 2)
                    ELSE 0
                  END as large_coin_ratio,
                  CASE 
                    WHEN cs.total_coins > 0 THEN ROUND((cs.dust_coins::NUMERIC / cs.total_coins::NUMERIC) * 100, 2)
                    ELSE 0
                  END as dust_ratio,
                  CASE 
                    WHEN cs.liquid_amount > 0 THEN ROUND((ra.amount_moved_7d::NUMERIC / cs.liquid_amount::NUMERIC) * 100, 2)
                    ELSE 0
                  END as weekly_velocity_percent
                FROM coin_stats cs, recent_activity ra, percentiles p
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH coin_stats AS (
                  SELECT 
                    c.asset_id,
                    COUNT(*) as total_coins,
                    COUNT(CASE WHEN c.amount >= 1000000000 THEN 1 END) as large_coins,
                    COUNT(CASE WHEN c.amount < 100000 THEN 1 END) as dust_coins,
                    COUNT(CASE WHEN NOT EXISTS(SELECT 1 FROM spends WHERE coin_id = c.coin_id) THEN 1 END) as unspent_coins,
                    SUM(c.amount) as total_amount,
                    SUM(CASE WHEN NOT EXISTS(SELECT 1 FROM spends WHERE coin_id = c.coin_id) THEN c.amount ELSE 0 END) as liquid_amount,
                    AVG(c.amount) as avg_coin_size,
                    COUNT(DISTINCT c.owner_puzzle_hash) as unique_addresses,
                    COUNT(DISTINCT CASE WHEN NOT EXISTS(SELECT 1 FROM spends WHERE coin_id = c.coin_id) THEN c.owner_puzzle_hash END) as active_addresses
                  FROM cats c
                  WHERE c.asset_id = ?
                  GROUP BY c.asset_id
                ),
                recent_activity AS (
                  SELECT 
                    COUNT(DISTINCT s.coin_id) as coins_spent_7d,
                    COUNT(DISTINCT s.spent_block) as active_blocks_7d,
                    SUM(c.amount) as amount_moved_7d
                  FROM cats c
                  JOIN spends s ON c.coin_id = s.coin_id
                  JOIN blocks b ON s.spent_block = b.height
                  WHERE c.asset_id = ?
                    AND datetime(b.timestamp) >= datetime('now', '-7 days')
                )
                SELECT 
                  cs.*,
                  NULL as median_coin_size,
                  NULL as coin_size_stddev,
                  ra.coins_spent_7d,
                  ra.active_blocks_7d,
                  ra.amount_moved_7d,
                  CASE 
                    WHEN cs.total_coins > 0 THEN ROUND((CAST(cs.large_coins AS REAL) / CAST(cs.total_coins AS REAL)) * 100, 2)
                    ELSE 0
                  END as large_coin_ratio,
                  CASE 
                    WHEN cs.total_coins > 0 THEN ROUND((CAST(cs.dust_coins AS REAL) / CAST(cs.total_coins AS REAL)) * 100, 2)
                    ELSE 0
                  END as dust_ratio,
                  CASE 
                    WHEN cs.liquid_amount > 0 THEN ROUND((CAST(ra.amount_moved_7d AS REAL) / CAST(cs.liquid_amount AS REAL)) * 100, 2)
                    ELSE 0
                  END as weekly_velocity_percent
                FROM coin_stats cs, recent_activity ra
                "#
            }
        };

        let row = match self.db_type {
            DatabaseType::Postgres => {
                sqlx::query(query)
                    .bind(&asset_id)
                    .fetch_optional(&*self.pool)
                    .await?
            }
            DatabaseType::Sqlite => {
                sqlx::query(query)
                    .bind(&asset_id)
                    .bind(&asset_id)
                    .fetch_optional(&*self.pool)
                    .await?
            }
        };

        Ok(row.map(|r| LiquidityAnalysis {
            asset_id: r.get("asset_id"),
            total_coins: r.get::<i64, _>("total_coins"),
            large_coins: r.get::<i64, _>("large_coins"),
            dust_coins: r.get::<i64, _>("dust_coins"),
            unspent_coins: r.get::<i64, _>("unspent_coins"),
            total_amount: r.get::<i64, _>("total_amount") as u64,
            liquid_amount: r.get::<i64, _>("liquid_amount") as u64,
            avg_coin_size: r.get::<i64, _>("avg_coin_size") as u64,
            median_coin_size: r.get::<Option<i64>, _>("median_coin_size").map(|v| v as u64),
            coin_size_stddev: r.get("coin_size_stddev"),
            unique_addresses: r.get::<i64, _>("unique_addresses"),
            active_addresses: r.get::<i64, _>("active_addresses"),
            coins_spent_7d: r.get::<i64, _>("coins_spent_7d"),
            active_blocks_7d: r.get::<i64, _>("active_blocks_7d"),
            amount_moved_7d: r.get::<i64, _>("amount_moved_7d") as u64,
            large_coin_ratio: r.get("large_coin_ratio"),
            dust_ratio: r.get("dust_ratio"),
            weekly_velocity_percent: r.get("weekly_velocity_percent"),
        }))
    }

    /// Get top holders of a CAT token
    async fn top_holders(
        &self,
        asset_id: String,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<TopHolder>> {
        let asset_id_bytes = hex::decode(&asset_id)
            .map_err(|_| GraphQLError::InvalidInput("Invalid asset ID hex".to_string()))?;

        let offset = (page - 1) * page_size;

        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH holder_balances AS (
                    SELECT 
                        encode(cb.puzzle_hash, 'hex') as holder_address,
                        SUM(CASE WHEN s.coin_id IS NULL THEN cb.amount ELSE 0 END) as balance
                    FROM cat_balances cb
                    LEFT JOIN spends s ON cb.coin_id = s.coin_id
                    WHERE cb.asset_id = $1
                    GROUP BY cb.puzzle_hash
                    HAVING SUM(CASE WHEN s.coin_id IS NULL THEN cb.amount ELSE 0 END) > 0
                ),
                total_supply AS (
                    SELECT SUM(balance) as total FROM holder_balances
                )
                SELECT 
                    hb.holder_address,
                    hb.balance,
                    (hb.balance * 100.0 / ts.total) as balance_percentage
                FROM holder_balances hb
                CROSS JOIN total_supply ts
                ORDER BY hb.balance DESC
                LIMIT $2 OFFSET $3
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH holder_balances AS (
                    SELECT 
                        hex(cb.puzzle_hash) as holder_address,
                        SUM(CASE WHEN s.coin_id IS NULL THEN cb.amount ELSE 0 END) as balance
                    FROM cat_balances cb
                    LEFT JOIN spends s ON cb.coin_id = s.coin_id
                    WHERE cb.asset_id = ?
                    GROUP BY cb.puzzle_hash
                    HAVING SUM(CASE WHEN s.coin_id IS NULL THEN cb.amount ELSE 0 END) > 0
                ),
                total_supply AS (
                    SELECT SUM(balance) as total FROM holder_balances
                )
                SELECT 
                    hb.holder_address,
                    hb.balance,
                    (hb.balance * 100.0 / ts.total) as balance_percentage
                FROM holder_balances hb
                CROSS JOIN total_supply ts
                ORDER BY hb.balance DESC
                LIMIT ? OFFSET ?
                "#
            }
        };

        let rows = sqlx::query(query)
            .bind(&asset_id_bytes)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?;

        Ok(rows.into_iter().map(|r| TopHolder {
            holder_address: r.get("holder_address"),
            balance: r.get::<i64, _>("balance") as u64,
            balance_percentage: r.get::<f64, _>("balance_percentage"),
        }).collect())
    }
} 