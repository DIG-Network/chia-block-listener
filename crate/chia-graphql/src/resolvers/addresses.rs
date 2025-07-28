use async_graphql::*;
use sqlx::{AnyPool, Row};
use std::sync::Arc;

use crate::database::DatabaseType;
use crate::error::{GraphQLError, GraphQLResult};
use crate::schema::types::*;

/// Address-related queries
pub struct AddressQueries {
    pool: Arc<AnyPool>,
    db_type: DatabaseType,
}

impl AddressQueries {
    pub fn new(pool: Arc<AnyPool>, db_type: DatabaseType) -> Self {
        Self { pool, db_type }
    }
}

#[Object]
impl AddressQueries {
    /// Get most active addresses by transaction count
    async fn most_active_addresses(
        &self,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<ActiveAddress>> {
        let offset = (page - 1) * page_size;
        
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH address_activity AS (
                    SELECT 
                        encode(c.puzzle_hash, 'hex') as puzzle_hash_hex,
                        COUNT(DISTINCT c.coin_id) as coins_received,
                        COUNT(DISTINCT s.coin_id) as coins_spent,
                        SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance,
                        SUM(c.amount) as total_received,
                        MIN(c.created_block) as first_activity_block,
                        MAX(COALESCE(s.spent_block, c.created_block)) as last_activity_block,
                        COUNT(DISTINCT c.created_block) as active_blocks_received,
                        COUNT(DISTINCT s.spent_block) as active_blocks_spent
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    GROUP BY c.puzzle_hash
                    HAVING COUNT(DISTINCT c.coin_id) + COUNT(DISTINCT s.coin_id) > 0
                )
                SELECT 
                    puzzle_hash_hex,
                    coins_received,
                    coins_spent,
                    current_balance,
                    total_received,
                    first_activity_block,
                    last_activity_block,
                    active_blocks_received,
                    active_blocks_spent,
                    (coins_received + coins_spent) as total_transactions,
                    (last_activity_block - first_activity_block) as lifespan_blocks
                FROM address_activity
                ORDER BY total_transactions DESC, current_balance DESC
                LIMIT $1 OFFSET $2
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH address_activity AS (
                    SELECT 
                        hex(c.puzzle_hash) as puzzle_hash_hex,
                        COUNT(DISTINCT c.coin_id) as coins_received,
                        COUNT(DISTINCT s.coin_id) as coins_spent,
                        SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance,
                        SUM(c.amount) as total_received,
                        MIN(c.created_block) as first_activity_block,
                        MAX(COALESCE(s.spent_block, c.created_block)) as last_activity_block,
                        COUNT(DISTINCT c.created_block) as active_blocks_received,
                        COUNT(DISTINCT s.spent_block) as active_blocks_spent
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    GROUP BY c.puzzle_hash
                    HAVING COUNT(DISTINCT c.coin_id) + COUNT(DISTINCT s.coin_id) > 0
                )
                SELECT 
                    puzzle_hash_hex,
                    coins_received,
                    coins_spent,
                    current_balance,
                    total_received,
                    first_activity_block,
                    last_activity_block,
                    active_blocks_received,
                    active_blocks_spent,
                    (coins_received + coins_spent) as total_transactions,
                    (last_activity_block - first_activity_block) as lifespan_blocks
                FROM address_activity
                ORDER BY total_transactions DESC, current_balance DESC
                LIMIT ? OFFSET ?
                "#
            }
        };

        let rows = sqlx::query(query)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(ActiveAddress {
                puzzle_hash_hex: row.get("puzzle_hash_hex"),
                coins_received: row.get("coins_received"),
                coins_spent: row.get("coins_spent"),
                current_balance: row.get::<i64, _>("current_balance") as u64,
                total_received: row.get::<i64, _>("total_received") as u64,
                first_activity_block: row.get("first_activity_block"),
                last_activity_block: row.get("last_activity_block"),
                active_blocks_received: row.get("active_blocks_received"),
                active_blocks_spent: row.get("active_blocks_spent"),
                total_transactions: row.get("total_transactions"),
                lifespan_blocks: row.get("lifespan_blocks"),
            });
        }

        Ok(results)
    }

    /// Get address behavior patterns across the network
    async fn address_behavior_patterns(&self) -> GraphQLResult<AddressBehaviorPatterns> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH address_behaviors AS (
                    SELECT 
                        encode(c.puzzle_hash, 'hex') as puzzle_hash_hex,
                        COUNT(DISTINCT c.coin_id) as coins_received,
                        COUNT(DISTINCT s.coin_id) as coins_spent,
                        SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance,
                        MIN(c.created_block) as first_block,
                        MAX(COALESCE(s.spent_block, c.created_block)) as last_block,
                        COUNT(DISTINCT c.created_block) as active_blocks_received,
                        COUNT(DISTINCT s.spent_block) as active_blocks_spent
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    GROUP BY c.puzzle_hash
                )
                SELECT 
                    COUNT(*) as total_addresses,
                    COUNT(CASE WHEN coins_spent = 0 AND current_balance > 0 THEN 1 END) as hodler_addresses,
                    COUNT(CASE WHEN coins_received = 1 AND coins_spent = 1 THEN 1 END) as transient_addresses,
                    COUNT(CASE WHEN coins_received > coins_spent AND current_balance > 0 THEN 1 END) as accumulator_addresses,
                    COUNT(CASE WHEN coins_received = 1 AND coins_spent <= 1 THEN 1 END) as single_use_addresses,
                    COUNT(CASE WHEN current_balance >= 1000000000000000 THEN 1 END) as whale_addresses,
                    AVG(coins_received) as avg_coins_received_per_address,
                    AVG(coins_spent) as avg_coins_spent_per_address,
                    AVG(last_block - first_block) as avg_address_lifespan_blocks
                FROM address_behaviors
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH address_behaviors AS (
                    SELECT 
                        hex(c.puzzle_hash) as puzzle_hash_hex,
                        COUNT(DISTINCT c.coin_id) as coins_received,
                        COUNT(DISTINCT s.coin_id) as coins_spent,
                        SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance,
                        MIN(c.created_block) as first_block,
                        MAX(COALESCE(s.spent_block, c.created_block)) as last_block,
                        COUNT(DISTINCT c.created_block) as active_blocks_received,
                        COUNT(DISTINCT s.spent_block) as active_blocks_spent
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    GROUP BY c.puzzle_hash
                )
                SELECT 
                    COUNT(*) as total_addresses,
                    COUNT(CASE WHEN coins_spent = 0 AND current_balance > 0 THEN 1 END) as hodler_addresses,
                    COUNT(CASE WHEN coins_received = 1 AND coins_spent = 1 THEN 1 END) as transient_addresses,
                    COUNT(CASE WHEN coins_received > coins_spent AND current_balance > 0 THEN 1 END) as accumulator_addresses,
                    COUNT(CASE WHEN coins_received = 1 AND coins_spent <= 1 THEN 1 END) as single_use_addresses,
                    COUNT(CASE WHEN current_balance >= 1000000000000000 THEN 1 END) as whale_addresses,
                    AVG(coins_received) as avg_coins_received_per_address,
                    AVG(coins_spent) as avg_coins_spent_per_address,
                    AVG(last_block - first_block) as avg_address_lifespan_blocks
                FROM address_behaviors
                "#
            }
        };

        let row = sqlx::query(query)
            .fetch_one(&*self.pool)
            .await?;

        Ok(AddressBehaviorPatterns {
            total_addresses: row.get("total_addresses"),
            hodler_addresses: row.get("hodler_addresses"),
            transient_addresses: row.get("transient_addresses"),
            accumulator_addresses: row.get("accumulator_addresses"),
            single_use_addresses: row.get("single_use_addresses"),
            whale_addresses: row.get("whale_addresses"),
            avg_coins_received_per_address: row.get("avg_coins_received_per_address"),
            avg_coins_spent_per_address: row.get("avg_coins_spent_per_address"),
            avg_address_lifespan_blocks: row.get("avg_address_lifespan_blocks"),
        })
    }

    /// Get dormant addresses (no activity in specified blocks)
    async fn dormant_addresses(
        &self,
        #[graphql(default = 10000)] blocks_inactive: i64,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<DormantAddress>> {
        let offset = (page - 1) * page_size;
        
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH current_height AS (
                    SELECT MAX(height) as max_height FROM blocks
                ),
                dormant AS (
                    SELECT 
                        encode(c.puzzle_hash, 'hex') as puzzle_hash_hex,
                        MAX(COALESCE(s.spent_block, c.created_block)) as last_activity_block,
                        SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance,
                        COUNT(DISTINCT c.coin_id) as total_coins_received,
                        COUNT(DISTINCT s.coin_id) as total_coins_spent
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    GROUP BY c.puzzle_hash
                    HAVING SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) > 0
                )
                SELECT 
                    d.puzzle_hash_hex,
                    d.last_activity_block,
                    (ch.max_height - d.last_activity_block) as blocks_since_activity,
                    d.current_balance,
                    d.total_coins_received,
                    d.total_coins_spent
                FROM dormant d, current_height ch
                WHERE (ch.max_height - d.last_activity_block) >= $1
                ORDER BY d.current_balance DESC, blocks_since_activity DESC
                LIMIT $2 OFFSET $3
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH current_height AS (
                    SELECT MAX(height) as max_height FROM blocks
                ),
                dormant AS (
                    SELECT 
                        hex(c.puzzle_hash) as puzzle_hash_hex,
                        MAX(COALESCE(s.spent_block, c.created_block)) as last_activity_block,
                        SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance,
                        COUNT(DISTINCT c.coin_id) as total_coins_received,
                        COUNT(DISTINCT s.coin_id) as total_coins_spent
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    GROUP BY c.puzzle_hash
                    HAVING SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) > 0
                )
                SELECT 
                    d.puzzle_hash_hex,
                    d.last_activity_block,
                    (ch.max_height - d.last_activity_block) as blocks_since_activity,
                    d.current_balance,
                    d.total_coins_received,
                    d.total_coins_spent
                FROM dormant d, current_height ch
                WHERE (ch.max_height - d.last_activity_block) >= ?
                ORDER BY d.current_balance DESC, blocks_since_activity DESC
                LIMIT ? OFFSET ?
                "#
            }
        };

        let rows = sqlx::query(query)
            .bind(blocks_inactive)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(DormantAddress {
                puzzle_hash_hex: row.get("puzzle_hash_hex"),
                last_activity_block: row.get("last_activity_block"),
                blocks_since_activity: row.get("blocks_since_activity"),
                current_balance: row.get::<i64, _>("current_balance") as u64,
                total_coins_received: row.get("total_coins_received"),
                total_coins_spent: row.get("total_coins_spent"),
            });
        }

        Ok(results)
    }

    /// Get new addresses (created in last N blocks)
    async fn new_addresses(
        &self,
        #[graphql(default = 1000)] blocks_back: i64,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<NewAddress>> {
        let offset = (page - 1) * page_size;
        
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH current_height AS (
                    SELECT MAX(height) as max_height FROM blocks
                ),
                new_addrs AS (
                    SELECT 
                        encode(c.puzzle_hash, 'hex') as puzzle_hash_hex,
                        MIN(c.created_block) as first_appearance_block,
                        COUNT(DISTINCT c.coin_id) as coins_received,
                        COUNT(DISTINCT s.coin_id) as coins_spent,
                        SUM(c.amount) as total_received,
                        SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    CROSS JOIN current_height ch
                    GROUP BY c.puzzle_hash, ch.max_height
                    HAVING MIN(c.created_block) >= (ch.max_height - $1)
                )
                SELECT * FROM new_addrs
                ORDER BY first_appearance_block DESC, total_received DESC
                LIMIT $2 OFFSET $3
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH current_height AS (
                    SELECT MAX(height) as max_height FROM blocks
                ),
                new_addrs AS (
                    SELECT 
                        hex(c.puzzle_hash) as puzzle_hash_hex,
                        MIN(c.created_block) as first_appearance_block,
                        COUNT(DISTINCT c.coin_id) as coins_received,
                        COUNT(DISTINCT s.coin_id) as coins_spent,
                        SUM(c.amount) as total_received,
                        SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    CROSS JOIN current_height ch
                    GROUP BY c.puzzle_hash, ch.max_height
                    HAVING MIN(c.created_block) >= (ch.max_height - ?)
                )
                SELECT * FROM new_addrs
                ORDER BY first_appearance_block DESC, total_received DESC
                LIMIT ? OFFSET ?
                "#
            }
        };

        let rows = sqlx::query(query)
            .bind(blocks_back)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(NewAddress {
                puzzle_hash_hex: row.get("puzzle_hash_hex"),
                first_appearance_block: row.get("first_appearance_block"),
                coins_received: row.get("coins_received"),
                coins_spent: row.get("coins_spent"),
                total_received: row.get::<i64, _>("total_received") as u64,
                current_balance: row.get::<i64, _>("current_balance") as u64,
            });
        }

        Ok(results)
    }

    /// Get comprehensive address profile
    async fn address_profile(&self, puzzle_hash: String) -> GraphQLResult<Option<AddressProfile>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH current_height AS (
                    SELECT MAX(height) as max_height FROM blocks
                ),
                address_stats AS (
                    SELECT 
                        encode(c.puzzle_hash, 'hex') as puzzle_hash_hex,
                        COUNT(DISTINCT c.coin_id) as total_coins_received,
                        COUNT(DISTINCT s.coin_id) as total_coins_spent,
                        SUM(c.amount) as total_value_received,
                        SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance,
                        MIN(c.created_block) as first_activity_block,
                        MAX(COALESCE(s.spent_block, c.created_block)) as last_activity_block,
                        COUNT(DISTINCT c.created_block) as active_blocks_received,
                        COUNT(DISTINCT s.spent_block) as active_blocks_spent,
                        MIN(c.amount) as min_coin_received,
                        MAX(c.amount) as max_coin_received,
                        AVG(c.amount) as avg_coin_received
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    WHERE encode(c.puzzle_hash, 'hex') = $1
                    GROUP BY c.puzzle_hash
                )
                SELECT 
                    a.*,
                    ch.max_height - a.last_activity_block as blocks_since_last_activity,
                    a.last_activity_block - a.first_activity_block as lifespan_blocks,
                    a.total_coins_received + a.total_coins_spent as total_transactions,
                    CASE 
                        WHEN a.total_coins_spent = 0 AND a.current_balance > 0 THEN 'hodler'
                        WHEN a.total_coins_received = 1 AND a.total_coins_spent = 1 THEN 'transient'
                        WHEN a.total_coins_received > a.total_coins_spent AND a.current_balance > 0 THEN 'accumulator'
                        WHEN a.current_balance >= 1000000000000000 THEN 'whale'
                        ELSE 'active'
                    END as address_type
                FROM address_stats a, current_height ch
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH current_height AS (
                    SELECT MAX(height) as max_height FROM blocks
                ),
                address_stats AS (
                    SELECT 
                        hex(c.puzzle_hash) as puzzle_hash_hex,
                        COUNT(DISTINCT c.coin_id) as total_coins_received,
                        COUNT(DISTINCT s.coin_id) as total_coins_spent,
                        SUM(c.amount) as total_value_received,
                        SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance,
                        MIN(c.created_block) as first_activity_block,
                        MAX(COALESCE(s.spent_block, c.created_block)) as last_activity_block,
                        COUNT(DISTINCT c.created_block) as active_blocks_received,
                        COUNT(DISTINCT s.spent_block) as active_blocks_spent,
                        MIN(c.amount) as min_coin_received,
                        MAX(c.amount) as max_coin_received,
                        AVG(c.amount) as avg_coin_received
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    WHERE hex(c.puzzle_hash) = ?
                    GROUP BY c.puzzle_hash
                )
                SELECT 
                    a.*,
                    ch.max_height - a.last_activity_block as blocks_since_last_activity,
                    a.last_activity_block - a.first_activity_block as lifespan_blocks,
                    a.total_coins_received + a.total_coins_spent as total_transactions,
                    CASE 
                        WHEN a.total_coins_spent = 0 AND a.current_balance > 0 THEN 'hodler'
                        WHEN a.total_coins_received = 1 AND a.total_coins_spent = 1 THEN 'transient'
                        WHEN a.total_coins_received > a.total_coins_spent AND a.current_balance > 0 THEN 'accumulator'
                        WHEN a.current_balance >= 1000000000000000 THEN 'whale'
                        ELSE 'active'
                    END as address_type
                FROM address_stats a, current_height ch
                "#
            }
        };

        let row = sqlx::query(query)
            .bind(&puzzle_hash)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(row.map(|r| AddressProfile {
            puzzle_hash_hex: r.get("puzzle_hash_hex"),
            total_coins_received: r.get("total_coins_received"),
            total_coins_spent: r.get("total_coins_spent"),
            total_value_received: r.get::<i64, _>("total_value_received") as u64,
            current_balance: r.get::<i64, _>("current_balance") as u64,
            first_activity_block: r.get("first_activity_block"),
            last_activity_block: r.get("last_activity_block"),
            blocks_since_last_activity: r.get("blocks_since_last_activity"),
            lifespan_blocks: r.get("lifespan_blocks"),
            active_blocks_received: r.get("active_blocks_received"),
            active_blocks_spent: r.get("active_blocks_spent"),
            total_transactions: r.get("total_transactions"),
            min_coin_received: r.get::<i64, _>("min_coin_received") as u64,
            max_coin_received: r.get::<i64, _>("max_coin_received") as u64,
            avg_coin_received: r.get::<i64, _>("avg_coin_received") as u64,
            address_type: r.get("address_type"),
        }))
    }

    /// Get transaction history for an address
    async fn address_transaction_history(
        &self,
        puzzle_hash: String,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<AddressTransaction>> {
        let offset = (page - 1) * page_size;
        
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH address_txs AS (
                    -- Receives
                    SELECT 
                        c.coin_id,
                        c.created_block as block_height,
                        b.timestamp as block_timestamp,
                        c.amount,
                        'receive' as transaction_type,
                        s.spent_block,
                        encode(c.parent_coin_info, 'hex') as parent_coin_id,
                        NULL::text as spent_to_address
                    FROM coins c
                    JOIN blocks b ON c.created_block = b.height
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    WHERE encode(c.puzzle_hash, 'hex') = $1
                    
                    UNION ALL
                    
                    -- Spends
                    SELECT 
                        s.coin_id,
                        s.spent_block as block_height,
                        b.timestamp as block_timestamp,
                        c.amount,
                        'spend' as transaction_type,
                        s.spent_block,
                        encode(c.parent_coin_info, 'hex') as parent_coin_id,
                        encode(c2.puzzle_hash, 'hex') as spent_to_address
                    FROM spends s
                    JOIN coins c ON s.coin_id = c.coin_id
                    JOIN blocks b ON s.spent_block = b.height
                    LEFT JOIN coins c2 ON c2.parent_coin_info = decode(s.coin_id, 'hex')
                    WHERE encode(c.puzzle_hash, 'hex') = $1
                )
                SELECT * FROM address_txs
                ORDER BY block_height DESC, transaction_type DESC
                LIMIT $2 OFFSET $3
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH address_txs AS (
                    -- Receives
                    SELECT 
                        c.coin_id,
                        c.created_block as block_height,
                        b.timestamp as block_timestamp,
                        c.amount,
                        'receive' as transaction_type,
                        s.spent_block,
                        hex(c.parent_coin_info) as parent_coin_id,
                        NULL as spent_to_address
                    FROM coins c
                    JOIN blocks b ON c.created_block = b.height
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    WHERE hex(c.puzzle_hash) = ?
                    
                    UNION ALL
                    
                    -- Spends
                    SELECT 
                        s.coin_id,
                        s.spent_block as block_height,
                        b.timestamp as block_timestamp,
                        c.amount,
                        'spend' as transaction_type,
                        s.spent_block,
                        hex(c.parent_coin_info) as parent_coin_id,
                        hex(c2.puzzle_hash) as spent_to_address
                    FROM spends s
                    JOIN coins c ON s.coin_id = c.coin_id
                    JOIN blocks b ON s.spent_block = b.height
                    LEFT JOIN coins c2 ON c2.parent_coin_info = s.coin_id
                    WHERE hex(c.puzzle_hash) = ?
                )
                SELECT * FROM address_txs
                ORDER BY block_height DESC, transaction_type DESC
                LIMIT ? OFFSET ?
                "#
            }
        };

        let rows = match self.db_type {
            DatabaseType::Postgres => {
                sqlx::query(query)
                    .bind(&puzzle_hash)
                    .bind(page_size)
                    .bind(offset)
                    .fetch_all(&*self.pool)
                    .await?
            }
            DatabaseType::Sqlite => {
                sqlx::query(query)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(page_size)
                    .bind(offset)
                    .fetch_all(&*self.pool)
                    .await?
            }
        };

        let mut results = Vec::new();
        for row in rows {
            results.push(AddressTransaction {
                coin_id: row.get("coin_id"),
                block_height: row.get("block_height"),
                block_timestamp: row.get("block_timestamp"),
                amount: row.get::<i64, _>("amount") as u64,
                transaction_type: row.get("transaction_type"),
                spent_block: row.get("spent_block"),
                parent_coin_id: row.get("parent_coin_id"),
                spent_to_address: row.get("spent_to_address"),
            });
        }

        Ok(results)
    }

    /// Get coin flow analysis for an address
    async fn address_coin_flow(&self, puzzle_hash: String) -> GraphQLResult<Option<AddressCoinFlow>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH flow_stats AS (
                    SELECT 
                        COUNT(DISTINCT c.coin_id) as coins_received,
                        SUM(c.amount) as total_received,
                        COUNT(DISTINCT s.coin_id) as coins_spent,
                        SUM(CASE WHEN s.coin_id IS NOT NULL THEN c.amount ELSE 0 END) as total_spent,
                        SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance,
                        COUNT(DISTINCT c.created_block) as blocks_received_in,
                        COUNT(DISTINCT s.spent_block) as blocks_spent_in
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    WHERE encode(c.puzzle_hash, 'hex') = $1
                )
                SELECT 
                    *,
                    (total_received - total_spent) as net_flow,
                    CASE 
                        WHEN total_received > 0 THEN ROUND((total_spent::NUMERIC / total_received::NUMERIC) * 100, 2)
                        ELSE 0
                    END as spend_ratio_percent,
                    CASE 
                        WHEN total_received > 0 THEN ROUND((current_balance::NUMERIC / total_received::NUMERIC) * 100, 2)
                        ELSE 0
                    END as retention_ratio_percent
                FROM flow_stats
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH flow_stats AS (
                    SELECT 
                        COUNT(DISTINCT c.coin_id) as coins_received,
                        SUM(c.amount) as total_received,
                        COUNT(DISTINCT s.coin_id) as coins_spent,
                        SUM(CASE WHEN s.coin_id IS NOT NULL THEN c.amount ELSE 0 END) as total_spent,
                        SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance,
                        COUNT(DISTINCT c.created_block) as blocks_received_in,
                        COUNT(DISTINCT s.spent_block) as blocks_spent_in
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    WHERE hex(c.puzzle_hash) = ?
                )
                SELECT 
                    *,
                    (total_received - total_spent) as net_flow,
                    CASE 
                        WHEN total_received > 0 THEN ROUND((CAST(total_spent AS REAL) / CAST(total_received AS REAL)) * 100, 2)
                        ELSE 0
                    END as spend_ratio_percent,
                    CASE 
                        WHEN total_received > 0 THEN ROUND((CAST(current_balance AS REAL) / CAST(total_received AS REAL)) * 100, 2)
                        ELSE 0
                    END as retention_ratio_percent
                FROM flow_stats
                "#
            }
        };

        let row = sqlx::query(query)
            .bind(&puzzle_hash)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(row.map(|r| AddressCoinFlow {
            coins_received: r.get("coins_received"),
            total_received: r.get::<i64, _>("total_received") as u64,
            coins_spent: r.get("coins_spent"),
            total_spent: r.get::<i64, _>("total_spent") as u64,
            current_balance: r.get::<i64, _>("current_balance") as u64,
            net_flow: r.get("net_flow"),
            blocks_received_in: r.get("blocks_received_in"),
            blocks_spent_in: r.get("blocks_spent_in"),
            spend_ratio_percent: r.get("spend_ratio_percent"),
            retention_ratio_percent: r.get("retention_ratio_percent"),
        }))
    }

    /// Get spending behavior analysis for an address
    async fn address_spending_behavior(&self, puzzle_hash: String) -> GraphQLResult<Option<AddressSpendingBehavior>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH spend_behavior AS (
                    SELECT 
                        COUNT(DISTINCT s.coin_id) as total_spends,
                        AVG(s.spent_block - c.created_block) as avg_blocks_held,
                        MIN(s.spent_block - c.created_block) as min_blocks_held,
                        MAX(s.spent_block - c.created_block) as max_blocks_held,
                        PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY s.spent_block - c.created_block) as median_blocks_held,
                        COUNT(CASE WHEN s.spent_block - c.created_block <= 1 THEN 1 END) as immediate_spends,
                        COUNT(CASE WHEN s.spent_block - c.created_block <= 10 THEN 1 END) as quick_spends,
                        COUNT(CASE WHEN s.spent_block - c.created_block > 1000 THEN 1 END) as long_term_holds,
                        SUM(c.amount) as total_amount_spent,
                        AVG(c.amount) as avg_spend_amount,
                        STDDEV(c.amount) as spend_amount_stddev
                    FROM coins c
                    JOIN spends s ON c.coin_id = s.coin_id
                    WHERE encode(c.puzzle_hash, 'hex') = $1
                )
                SELECT * FROM spend_behavior
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH spend_behavior AS (
                    SELECT 
                        COUNT(DISTINCT s.coin_id) as total_spends,
                        AVG(s.spent_block - c.created_block) as avg_blocks_held,
                        MIN(s.spent_block - c.created_block) as min_blocks_held,
                        MAX(s.spent_block - c.created_block) as max_blocks_held,
                        COUNT(CASE WHEN s.spent_block - c.created_block <= 1 THEN 1 END) as immediate_spends,
                        COUNT(CASE WHEN s.spent_block - c.created_block <= 10 THEN 1 END) as quick_spends,
                        COUNT(CASE WHEN s.spent_block - c.created_block > 1000 THEN 1 END) as long_term_holds,
                        SUM(c.amount) as total_amount_spent,
                        AVG(c.amount) as avg_spend_amount
                    FROM coins c
                    JOIN spends s ON c.coin_id = s.coin_id
                    WHERE hex(c.puzzle_hash) = ?
                )
                SELECT *, NULL as median_blocks_held, NULL as spend_amount_stddev FROM spend_behavior
                "#
            }
        };

        let row = sqlx::query(query)
            .bind(&puzzle_hash)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(row.map(|r| AddressSpendingBehavior {
            total_spends: r.get("total_spends"),
            avg_blocks_held: r.get("avg_blocks_held"),
            min_blocks_held: r.get("min_blocks_held"),
            max_blocks_held: r.get("max_blocks_held"),
            median_blocks_held: r.get("median_blocks_held"),
            immediate_spends: r.get("immediate_spends"),
            quick_spends: r.get("quick_spends"),
            long_term_holds: r.get("long_term_holds"),
            total_amount_spent: r.get::<i64, _>("total_amount_spent") as u64,
            avg_spend_amount: r.get("avg_spend_amount"),
            spend_amount_stddev: r.get("spend_amount_stddev"),
        }))
    }

    /// Get peer addresses that this address has interacted with
    async fn address_peers(
        &self,
        puzzle_hash: String,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<AddressPeer>> {
        let offset = (page - 1) * page_size;
        
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH peer_interactions AS (
                    -- Addresses we sent to
                    SELECT 
                        encode(c2.puzzle_hash, 'hex') as peer_address,
                        COUNT(*) as total_interactions,
                        SUM(c2.amount) as total_value_exchanged,
                        MIN(s.spent_block) as first_interaction,
                        MAX(s.spent_block) as last_interaction,
                        1 as directions_count
                    FROM spends s
                    JOIN coins c ON s.coin_id = c.coin_id
                    JOIN coins c2 ON c2.parent_coin_info = decode(s.coin_id, 'hex')
                    WHERE encode(c.puzzle_hash, 'hex') = $1
                        AND c2.puzzle_hash != c.puzzle_hash
                    GROUP BY encode(c2.puzzle_hash, 'hex')
                    
                    UNION ALL
                    
                    -- Addresses we received from
                    SELECT 
                        encode(c_parent.puzzle_hash, 'hex') as peer_address,
                        COUNT(*) as total_interactions,
                        SUM(c.amount) as total_value_exchanged,
                        MIN(c.created_block) as first_interaction,
                        MAX(c.created_block) as last_interaction,
                        1 as directions_count
                    FROM coins c
                    JOIN coins c_parent ON c.parent_coin_info = decode(c_parent.coin_id, 'hex')
                    WHERE encode(c.puzzle_hash, 'hex') = $1
                        AND c_parent.puzzle_hash != c.puzzle_hash
                    GROUP BY encode(c_parent.puzzle_hash, 'hex')
                )
                SELECT 
                    peer_address,
                    SUM(total_interactions) as total_interactions,
                    SUM(total_value_exchanged) as total_value_exchanged,
                    MIN(first_interaction) as first_interaction,
                    MAX(last_interaction) as last_interaction,
                    COUNT(*) as directions_count
                FROM peer_interactions
                GROUP BY peer_address
                ORDER BY total_interactions DESC, total_value_exchanged DESC
                LIMIT $2 OFFSET $3
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH peer_interactions AS (
                    -- Addresses we sent to
                    SELECT 
                        hex(c2.puzzle_hash) as peer_address,
                        COUNT(*) as total_interactions,
                        SUM(c2.amount) as total_value_exchanged,
                        MIN(s.spent_block) as first_interaction,
                        MAX(s.spent_block) as last_interaction,
                        1 as directions_count
                    FROM spends s
                    JOIN coins c ON s.coin_id = c.coin_id
                    JOIN coins c2 ON c2.parent_coin_info = s.coin_id
                    WHERE hex(c.puzzle_hash) = ?
                        AND c2.puzzle_hash != c.puzzle_hash
                    GROUP BY hex(c2.puzzle_hash)
                    
                    UNION ALL
                    
                    -- Addresses we received from
                    SELECT 
                        hex(c_parent.puzzle_hash) as peer_address,
                        COUNT(*) as total_interactions,
                        SUM(c.amount) as total_value_exchanged,
                        MIN(c.created_block) as first_interaction,
                        MAX(c.created_block) as last_interaction,
                        1 as directions_count
                    FROM coins c
                    JOIN coins c_parent ON c.parent_coin_info = c_parent.coin_id
                    WHERE hex(c.puzzle_hash) = ?
                        AND c_parent.puzzle_hash != c.puzzle_hash
                    GROUP BY hex(c_parent.puzzle_hash)
                )
                SELECT 
                    peer_address,
                    SUM(total_interactions) as total_interactions,
                    SUM(total_value_exchanged) as total_value_exchanged,
                    MIN(first_interaction) as first_interaction,
                    MAX(last_interaction) as last_interaction,
                    COUNT(*) as directions_count
                FROM peer_interactions
                GROUP BY peer_address
                ORDER BY total_interactions DESC, total_value_exchanged DESC
                LIMIT ? OFFSET ?
                "#
            }
        };

        let rows = match self.db_type {
            DatabaseType::Postgres => {
                sqlx::query(query)
                    .bind(&puzzle_hash)
                    .bind(page_size)
                    .bind(offset)
                    .fetch_all(&*self.pool)
                    .await?
            }
            DatabaseType::Sqlite => {
                sqlx::query(query)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(page_size)
                    .bind(offset)
                    .fetch_all(&*self.pool)
                    .await?
            }
        };

        let mut results = Vec::new();
        for row in rows {
            results.push(AddressPeer {
                peer_address: row.get("peer_address"),
                total_interactions: row.get("total_interactions"),
                total_value_exchanged: row.get::<i64, _>("total_value_exchanged") as u64,
                first_interaction: row.get("first_interaction"),
                last_interaction: row.get("last_interaction"),
                directions_count: row.get("directions_count"),
            });
        }

        Ok(results)
    }

    /// Get risk assessment for an address
    async fn address_risk_assessment(&self, puzzle_hash: String) -> GraphQLResult<Option<AddressRiskAssessment>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH risk_data AS (
                    SELECT 
                        COUNT(DISTINCT c.coin_id) as total_coins,
                        COUNT(DISTINCT s.coin_id) as coins_spent,
                        COUNT(DISTINCT c.created_block) + COUNT(DISTINCT s.spent_block) as active_blocks,
                        SUM(c.amount) as total_received,
                        SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance,
                        MIN(c.created_block) as first_block,
                        MAX(COALESCE(s.spent_block, c.created_block)) as last_block,
                        COUNT(CASE WHEN c.amount < 1000000000 THEN 1 END) as dust_coins,
                        COUNT(CASE WHEN c.amount > 1000000000000000 THEN 1 END) as large_coins,
                        AVG(CASE WHEN s.spent_block IS NOT NULL THEN s.spent_block - c.created_block ELSE NULL END) as avg_hold_time
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    WHERE encode(c.puzzle_hash, 'hex') = $1
                )
                SELECT 
                    *,
                    CASE 
                        WHEN total_coins = 1 AND coins_spent = 0 THEN 'low'
                        WHEN dust_coins > total_coins * 0.5 THEN 'high'
                        WHEN avg_hold_time < 10 THEN 'medium'
                        WHEN current_balance >= 1000000000000000 THEN 'low'
                        ELSE 'normal'
                    END as risk_category,
                    CASE 
                        WHEN total_coins = 1 AND coins_spent = 0 THEN 10
                        WHEN dust_coins > total_coins * 0.5 THEN 80
                        WHEN avg_hold_time < 10 THEN 60
                        WHEN current_balance >= 1000000000000000 THEN 20
                        ELSE 40
                    END as risk_score
                FROM risk_data
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH risk_data AS (
                    SELECT 
                        COUNT(DISTINCT c.coin_id) as total_coins,
                        COUNT(DISTINCT s.coin_id) as coins_spent,
                        COUNT(DISTINCT c.created_block) + COUNT(DISTINCT s.spent_block) as active_blocks,
                        SUM(c.amount) as total_received,
                        SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance,
                        MIN(c.created_block) as first_block,
                        MAX(COALESCE(s.spent_block, c.created_block)) as last_block,
                        COUNT(CASE WHEN c.amount < 1000000000 THEN 1 END) as dust_coins,
                        COUNT(CASE WHEN c.amount > 1000000000000000 THEN 1 END) as large_coins,
                        AVG(CASE WHEN s.spent_block IS NOT NULL THEN s.spent_block - c.created_block ELSE NULL END) as avg_hold_time
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    WHERE hex(c.puzzle_hash) = ?
                )
                SELECT 
                    *,
                    CASE 
                        WHEN total_coins = 1 AND coins_spent = 0 THEN 'low'
                        WHEN dust_coins > total_coins * 0.5 THEN 'high'
                        WHEN avg_hold_time < 10 THEN 'medium'
                        WHEN current_balance >= 1000000000000000 THEN 'low'
                        ELSE 'normal'
                    END as risk_category,
                    CASE 
                        WHEN total_coins = 1 AND coins_spent = 0 THEN 10
                        WHEN dust_coins > total_coins * 0.5 THEN 80
                        WHEN avg_hold_time < 10 THEN 60
                        WHEN current_balance >= 1000000000000000 THEN 20
                        ELSE 40
                    END as risk_score
                FROM risk_data
                "#
            }
        };

        let row = sqlx::query(query)
            .bind(&puzzle_hash)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(row.map(|r| AddressRiskAssessment {
            total_coins: r.get("total_coins"),
            coins_spent: r.get("coins_spent"),
            active_blocks: r.get("active_blocks"),
            total_received: r.get::<i64, _>("total_received") as u64,
            current_balance: r.get::<i64, _>("current_balance") as u64,
            first_block: r.get("first_block"),
            last_block: r.get("last_block"),
            dust_coins: r.get("dust_coins"),
            large_coins: r.get("large_coins"),
            avg_hold_time: r.get("avg_hold_time"),
            risk_category: r.get("risk_category"),
            risk_score: r.get("risk_score"),
        }))
    }

    /// Get activity timeline for an address
    async fn address_activity_timeline(
        &self,
        puzzle_hash: String,
        #[graphql(default = 30)] days_back: i32,
    ) -> GraphQLResult<Vec<AddressActivityTimeline>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH daily_activity AS (
                    SELECT 
                        DATE(b.timestamp) as activity_date,
                        COUNT(DISTINCT CASE WHEN c.puzzle_hash = decode($1, 'hex') THEN c.coin_id END) as coins_received,
                        COUNT(DISTINCT CASE WHEN c.puzzle_hash = decode($1, 'hex') AND s.coin_id IS NOT NULL THEN s.coin_id END) as coins_spent,
                        SUM(CASE WHEN c.puzzle_hash = decode($1, 'hex') THEN c.amount ELSE 0 END) as value_received,
                        SUM(CASE WHEN c.puzzle_hash = decode($1, 'hex') AND s.coin_id IS NOT NULL THEN c.amount ELSE 0 END) as value_spent
                    FROM blocks b
                    LEFT JOIN coins c ON c.created_block = b.height
                    LEFT JOIN spends s ON s.spent_block = b.height AND s.coin_id = c.coin_id
                    WHERE b.timestamp >= NOW() - INTERVAL '%d days'
                        AND (c.puzzle_hash = decode($1, 'hex') OR c.puzzle_hash IS NULL)
                    GROUP BY DATE(b.timestamp)
                )
                SELECT 
                    activity_date::text,
                    coins_received,
                    coins_spent,
                    value_received,
                    value_spent,
                    (value_received - value_spent) as net_change,
                    (coins_received + coins_spent) as total_activity
                FROM daily_activity
                WHERE coins_received > 0 OR coins_spent > 0
                ORDER BY activity_date DESC
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH daily_activity AS (
                    SELECT 
                        DATE(b.timestamp) as activity_date,
                        COUNT(DISTINCT CASE WHEN hex(c.puzzle_hash) = ? THEN c.coin_id END) as coins_received,
                        COUNT(DISTINCT CASE WHEN hex(c.puzzle_hash) = ? AND s.coin_id IS NOT NULL THEN s.coin_id END) as coins_spent,
                        SUM(CASE WHEN hex(c.puzzle_hash) = ? THEN c.amount ELSE 0 END) as value_received,
                        SUM(CASE WHEN hex(c.puzzle_hash) = ? AND s.coin_id IS NOT NULL THEN c.amount ELSE 0 END) as value_spent
                    FROM blocks b
                    LEFT JOIN coins c ON c.created_block = b.height
                    LEFT JOIN spends s ON s.spent_block = b.height AND s.coin_id = c.coin_id
                    WHERE datetime(b.timestamp) >= datetime('now', '-%d days')
                        AND (hex(c.puzzle_hash) = ? OR c.puzzle_hash IS NULL)
                    GROUP BY DATE(b.timestamp)
                )
                SELECT 
                    activity_date,
                    coins_received,
                    coins_spent,
                    value_received,
                    value_spent,
                    (value_received - value_spent) as net_change,
                    (coins_received + coins_spent) as total_activity
                FROM daily_activity
                WHERE coins_received > 0 OR coins_spent > 0
                ORDER BY activity_date DESC
                "#
            }
        };

        let formatted_query = query.replace("%d", &days_back.to_string());
        
        let rows = match self.db_type {
            DatabaseType::Postgres => {
                sqlx::query(&formatted_query)
                    .bind(&puzzle_hash)
                    .fetch_all(&*self.pool)
                    .await?
            }
            DatabaseType::Sqlite => {
                sqlx::query(&formatted_query)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .fetch_all(&*self.pool)
                    .await?
            }
        };

        let mut results = Vec::new();
        for row in rows {
            results.push(AddressActivityTimeline {
                activity_date: row.get("activity_date"),
                coins_received: row.get("coins_received"),
                coins_spent: row.get("coins_spent"),
                value_received: row.get::<i64, _>("value_received") as u64,
                value_spent: row.get::<i64, _>("value_spent") as u64,
                net_change: row.get("net_change"),
                total_activity: row.get("total_activity"),
            });
        }

        Ok(results)
    }

    /// Get address concentration analysis
    async fn address_concentration_analysis(&self) -> GraphQLResult<AddressConcentrationAnalysis> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH balance_data AS (
                    SELECT 
                        encode(c.puzzle_hash, 'hex') as puzzle_hash_hex,
                        SUM(c.amount) as balance
                    FROM coins c
                    WHERE NOT EXISTS (SELECT 1 FROM spends s WHERE s.coin_id = c.coin_id)
                    GROUP BY c.puzzle_hash
                    HAVING SUM(c.amount) > 0
                ),
                total_stats AS (
                    SELECT 
                        COUNT(*) as total_addresses_with_balance,
                        SUM(balance) as total_supply
                    FROM balance_data
                ),
                ranked_balances AS (
                    SELECT 
                        balance,
                        ROW_NUMBER() OVER (ORDER BY balance DESC) as rank
                    FROM balance_data
                ),
                concentration AS (
                    SELECT 
                        COUNT(CASE WHEN b.balance >= t.total_supply * 0.01 THEN 1 END) as addresses_with_1_percent_plus,
                        COUNT(CASE WHEN b.balance >= t.total_supply * 0.001 THEN 1 END) as addresses_with_point_1_percent_plus,
                        SUM(CASE WHEN r.rank <= 10 THEN r.balance ELSE 0 END) as top_10_balance,
                        SUM(CASE WHEN r.rank <= 100 THEN r.balance ELSE 0 END) as top_100_balance,
                        SUM(CASE WHEN r.rank <= 1000 THEN r.balance ELSE 0 END) as top_1000_balance
                    FROM balance_data b
                    CROSS JOIN total_stats t
                    LEFT JOIN ranked_balances r ON b.balance = r.balance
                ),
                percentiles AS (
                    SELECT 
                        PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY balance) as median_balance,
                        PERCENTILE_CONT(0.9) WITHIN GROUP (ORDER BY balance) as p90_balance,
                        PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY balance) as p99_balance
                    FROM balance_data
                )
                SELECT 
                    t.total_addresses_with_balance,
                    t.total_supply,
                    c.addresses_with_1_percent_plus,
                    c.addresses_with_point_1_percent_plus,
                    c.top_10_balance,
                    c.top_100_balance,
                    c.top_1000_balance,
                    p.median_balance,
                    p.p90_balance,
                    p.p99_balance
                FROM total_stats t, concentration c, percentiles p
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH balance_data AS (
                    SELECT 
                        hex(c.puzzle_hash) as puzzle_hash_hex,
                        SUM(c.amount) as balance
                    FROM coins c
                    WHERE NOT EXISTS (SELECT 1 FROM spends s WHERE s.coin_id = c.coin_id)
                    GROUP BY c.puzzle_hash
                    HAVING SUM(c.amount) > 0
                ),
                total_stats AS (
                    SELECT 
                        COUNT(*) as total_addresses_with_balance,
                        SUM(balance) as total_supply
                    FROM balance_data
                ),
                ranked_balances AS (
                    SELECT 
                        balance,
                        ROW_NUMBER() OVER (ORDER BY balance DESC) as rank
                    FROM balance_data
                ),
                concentration AS (
                    SELECT 
                        COUNT(CASE WHEN b.balance >= t.total_supply * 0.01 THEN 1 END) as addresses_with_1_percent_plus,
                        COUNT(CASE WHEN b.balance >= t.total_supply * 0.001 THEN 1 END) as addresses_with_point_1_percent_plus,
                        SUM(CASE WHEN r.rank <= 10 THEN r.balance ELSE 0 END) as top_10_balance,
                        SUM(CASE WHEN r.rank <= 100 THEN r.balance ELSE 0 END) as top_100_balance,
                        SUM(CASE WHEN r.rank <= 1000 THEN r.balance ELSE 0 END) as top_1000_balance
                    FROM balance_data b
                    CROSS JOIN total_stats t
                    LEFT JOIN ranked_balances r ON b.balance = r.balance
                )
                SELECT 
                    t.total_addresses_with_balance,
                    t.total_supply,
                    c.addresses_with_1_percent_plus,
                    c.addresses_with_point_1_percent_plus,
                    c.top_10_balance,
                    c.top_100_balance,
                    c.top_1000_balance,
                    NULL as median_balance,
                    NULL as p90_balance,
                    NULL as p99_balance
                FROM total_stats t, concentration c
                "#
            }
        };

        let row = sqlx::query(query)
            .fetch_one(&*self.pool)
            .await?;

        Ok(AddressConcentrationAnalysis {
            total_addresses_with_balance: row.get("total_addresses_with_balance"),
            total_supply: row.get::<i64, _>("total_supply") as u64,
            addresses_with_1_percent_plus: row.get("addresses_with_1_percent_plus"),
            addresses_with_point_1_percent_plus: row.get("addresses_with_point_1_percent_plus"),
            top_10_balance: row.get::<i64, _>("top_10_balance") as u64,
            top_100_balance: row.get::<i64, _>("top_100_balance") as u64,
            top_1000_balance: row.get::<i64, _>("top_1000_balance") as u64,
            median_balance: row.get("median_balance"),
            p90_balance: row.get("p90_balance"),
            p99_balance: row.get("p99_balance"),
        })
    }

    /// Get detailed activity statistics for a specific address
    async fn address_activity(
        &self,
        puzzle_hash_hex: String,
        #[graphql(default = 30)] days_back: i32,
    ) -> GraphQLResult<Option<AddressActivity>> {
        let puzzle_hash = hex::decode(&puzzle_hash_hex)
            .map_err(|_| GraphQLError::InvalidInput("Invalid puzzle hash hex".to_string()))?;

        let blocks_back = days_back * 4608; // Approximately 4608 blocks per day

        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH address_stats AS (
                    SELECT 
                        COUNT(DISTINCT CASE WHEN c.puzzle_hash = $1 THEN c.coin_id END) as coins_received,
                        COUNT(DISTINCT CASE WHEN s.coin_id IS NOT NULL AND c.puzzle_hash = $1 THEN s.coin_id END) as coins_spent,
                        SUM(CASE WHEN c.puzzle_hash = $1 THEN c.amount ELSE 0 END) as value_received,
                        SUM(CASE WHEN s.coin_id IS NOT NULL AND c.puzzle_hash = $1 THEN c.amount ELSE 0 END) as value_spent,
                        COUNT(DISTINCT CASE WHEN c.puzzle_hash != $1 AND s.coin_id IS NOT NULL THEN c.puzzle_hash END) as unique_senders,
                        COUNT(DISTINCT CASE WHEN c.puzzle_hash = $1 AND s.coin_id IS NOT NULL THEN 
                            (SELECT cc.puzzle_hash FROM coins cc WHERE cc.coin_id = s.spent_to_coin_id LIMIT 1) END) as unique_recipients,
                        MAX(c.created_block) as peak_activity_block
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    WHERE (c.puzzle_hash = $1 OR s.coin_id IN (
                        SELECT cc.coin_id FROM coins cc WHERE cc.puzzle_hash = $1
                    ))
                    AND c.created_block >= (SELECT MAX(height) FROM blocks) - $2
                )
                SELECT 
                    coins_received as total_coins_received,
                    coins_spent as total_coins_spent,
                    value_received as total_value_received,
                    value_spent as total_value_spent,
                    unique_senders,
                    unique_recipients,
                    CASE WHEN value_received > 0 THEN value_received / coins_received ELSE 0 END as avg_transaction_value,
                    (SELECT timestamp FROM blocks WHERE height = peak_activity_block) as peak_activity_date
                FROM address_stats
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH address_stats AS (
                    SELECT 
                        COUNT(DISTINCT CASE WHEN c.puzzle_hash = ? THEN c.coin_id END) as coins_received,
                        COUNT(DISTINCT CASE WHEN s.coin_id IS NOT NULL AND c.puzzle_hash = ? THEN s.coin_id END) as coins_spent,
                        SUM(CASE WHEN c.puzzle_hash = ? THEN c.amount ELSE 0 END) as value_received,
                        SUM(CASE WHEN s.coin_id IS NOT NULL AND c.puzzle_hash = ? THEN c.amount ELSE 0 END) as value_spent,
                        COUNT(DISTINCT CASE WHEN c.puzzle_hash != ? AND s.coin_id IS NOT NULL THEN c.puzzle_hash END) as unique_senders,
                        COUNT(DISTINCT CASE WHEN c.puzzle_hash = ? AND s.coin_id IS NOT NULL THEN 
                            (SELECT cc.puzzle_hash FROM coins cc WHERE cc.coin_id = s.spent_to_coin_id LIMIT 1) END) as unique_recipients,
                        MAX(c.created_block) as peak_activity_block
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    WHERE (c.puzzle_hash = ? OR s.coin_id IN (
                        SELECT cc.coin_id FROM coins cc WHERE cc.puzzle_hash = ?
                    ))
                    AND c.created_block >= (SELECT MAX(height) FROM blocks) - ?
                )
                SELECT 
                    coins_received as total_coins_received,
                    coins_spent as total_coins_spent,
                    value_received as total_value_received,
                    value_spent as total_value_spent,
                    unique_senders,
                    unique_recipients,
                    CASE WHEN value_received > 0 THEN value_received / coins_received ELSE 0 END as avg_transaction_value,
                    (SELECT timestamp FROM blocks WHERE height = peak_activity_block) as peak_activity_date
                FROM address_stats
                "#
            }
        };

        let row = match self.db_type {
            DatabaseType::Postgres => {
                sqlx::query(query)
                    .bind(&puzzle_hash)
                    .bind(blocks_back)
                    .fetch_optional(&*self.pool)
                    .await?
            }
            DatabaseType::Sqlite => {
                sqlx::query(query)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(blocks_back)
                    .fetch_optional(&*self.pool)
                    .await?
            }
        };

        Ok(row.map(|r| AddressActivity {
            total_coins_received: r.get::<i64, _>("total_coins_received") as u64,
            total_coins_spent: r.get::<i64, _>("total_coins_spent") as u64,
            total_value_received: r.get::<i64, _>("total_value_received") as u64,
            total_value_spent: r.get::<i64, _>("total_value_spent") as u64,
            unique_senders: r.get::<i64, _>("unique_senders") as u32,
            unique_recipients: r.get::<i64, _>("unique_recipients") as u32,
            avg_transaction_value: r.get::<f64, _>("avg_transaction_value"),
            peak_activity_date: r.get("peak_activity_date"),
        }))
    }

    /// Get addresses sorted by their current balance
    async fn addresses_by_balance(
        &self,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<AddressByBalance>> {
        let offset = (page - 1) * page_size;

        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH address_balances AS (
                    SELECT 
                        encode(c.puzzle_hash, 'hex') as puzzle_hash_hex,
                        SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance,
                        COUNT(CASE WHEN s.coin_id IS NULL THEN c.coin_id END) as coin_count,
                        MIN(c.created_block) as first_seen,
                        MAX(COALESCE(s.spent_block, c.created_block)) as last_seen
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    GROUP BY c.puzzle_hash
                    HAVING SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) > 0
                )
                SELECT * FROM address_balances
                ORDER BY current_balance DESC
                LIMIT $1 OFFSET $2
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH address_balances AS (
                    SELECT 
                        hex(c.puzzle_hash) as puzzle_hash_hex,
                        SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) as current_balance,
                        COUNT(CASE WHEN s.coin_id IS NULL THEN c.coin_id END) as coin_count,
                        MIN(c.created_block) as first_seen,
                        MAX(COALESCE(s.spent_block, c.created_block)) as last_seen
                    FROM coins c
                    LEFT JOIN spends s ON c.coin_id = s.coin_id
                    GROUP BY c.puzzle_hash
                    HAVING SUM(CASE WHEN s.coin_id IS NULL THEN c.amount ELSE 0 END) > 0
                )
                SELECT * FROM address_balances
                ORDER BY current_balance DESC
                LIMIT ? OFFSET ?
                "#
            }
        };

        let rows = sqlx::query(query)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?;

        Ok(rows.into_iter().map(|r| AddressByBalance {
            puzzle_hash_hex: r.get("puzzle_hash_hex"),
            current_balance: r.get::<i64, _>("current_balance") as u64,
            coin_count: r.get::<i64, _>("coin_count") as u32,
            first_seen: r.get("first_seen"),
            last_seen: r.get("last_seen"),
        }).collect())
    }

    /// Get the interaction network for an address
    async fn address_interaction_network(
        &self,
        puzzle_hash_hex: String,
        #[graphql(default = 1)] depth: i32,
        #[graphql(default = 20)] limit: i32,
    ) -> GraphQLResult<Vec<AddressInteraction>> {
        let puzzle_hash = hex::decode(&puzzle_hash_hex)
            .map_err(|_| GraphQLError::InvalidInput("Invalid puzzle hash hex".to_string()))?;

        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH RECURSIVE interaction_network AS (
                    -- Direct interactions (depth 1)
                    SELECT 
                        encode($1, 'hex') as from_address,
                        encode(target.puzzle_hash, 'hex') as to_address,
                        COUNT(*) as interaction_count,
                        SUM(source.amount) as total_value,
                        MIN(source.created_block) as first_interaction,
                        MAX(source.created_block) as last_interaction,
                        1 as depth_level
                    FROM coins source
                    INNER JOIN spends s ON source.coin_id = s.coin_id
                    INNER JOIN coins target ON s.spent_to_coin_id = target.coin_id
                    WHERE source.puzzle_hash = $1 AND target.puzzle_hash != $1
                    GROUP BY target.puzzle_hash
                    
                    UNION ALL
                    
                    -- Reverse interactions  
                    SELECT 
                        encode(source.puzzle_hash, 'hex') as from_address,
                        encode($1, 'hex') as to_address,
                        COUNT(*) as interaction_count,
                        SUM(source.amount) as total_value,
                        MIN(source.created_block) as first_interaction,
                        MAX(source.created_block) as last_interaction,
                        1 as depth_level
                    FROM coins source
                    INNER JOIN spends s ON source.coin_id = s.coin_id
                    INNER JOIN coins target ON s.spent_to_coin_id = target.coin_id
                    WHERE target.puzzle_hash = $1 AND source.puzzle_hash != $1
                    GROUP BY source.puzzle_hash
                )
                SELECT * FROM interaction_network
                WHERE depth_level <= $2
                ORDER BY total_value DESC
                LIMIT $3
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                    hex(?) as from_address,
                    hex(target.puzzle_hash) as to_address,
                    COUNT(*) as interaction_count,
                    SUM(source.amount) as total_value,
                    MIN(source.created_block) as first_interaction,
                    MAX(source.created_block) as last_interaction
                FROM coins source
                INNER JOIN spends s ON source.coin_id = s.coin_id
                INNER JOIN coins target ON s.spent_to_coin_id = target.coin_id
                WHERE source.puzzle_hash = ? AND target.puzzle_hash != ?
                GROUP BY target.puzzle_hash
                
                UNION ALL
                
                SELECT 
                    hex(source.puzzle_hash) as from_address,
                    hex(?) as to_address,
                    COUNT(*) as interaction_count,
                    SUM(source.amount) as total_value,
                    MIN(source.created_block) as first_interaction,
                    MAX(source.created_block) as last_interaction
                FROM coins source
                INNER JOIN spends s ON source.coin_id = s.coin_id
                INNER JOIN coins target ON s.spent_to_coin_id = target.coin_id
                WHERE target.puzzle_hash = ? AND source.puzzle_hash != ?
                GROUP BY source.puzzle_hash
                
                ORDER BY total_value DESC
                LIMIT ?
                "#
            }
        };

        let rows = match self.db_type {
            DatabaseType::Postgres => {
                sqlx::query(query)
                    .bind(&puzzle_hash)
                    .bind(depth)
                    .bind(limit)
                    .fetch_all(&*self.pool)
                    .await?
            }
            DatabaseType::Sqlite => {
                sqlx::query(query)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(&puzzle_hash)
                    .bind(limit)
                    .fetch_all(&*self.pool)
                    .await?
            }
        };

        Ok(rows.into_iter().map(|r| AddressInteraction {
            from_address: r.get("from_address"),
            to_address: r.get("to_address"),
            interaction_count: r.get::<i64, _>("interaction_count") as u32,
            total_value: r.get::<i64, _>("total_value") as u64,
            first_interaction: r.get("first_interaction"),
            last_interaction: r.get("last_interaction"),
        }).collect())
    }

    /// Analyze address activity patterns over time
    async fn address_time_analysis(&self, puzzle_hash_hex: String) -> GraphQLResult<Option<AddressTimeAnalysis>> {
        let puzzle_hash = hex::decode(&puzzle_hash_hex)
            .map_err(|_| GraphQLError::InvalidInput("Invalid puzzle hash hex".to_string()))?;

        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH time_analysis AS (
                    SELECT 
                        EXTRACT(hour FROM TO_TIMESTAMP(b.timestamp)) as hour_of_day,
                        EXTRACT(dow FROM TO_TIMESTAMP(b.timestamp)) as day_of_week,
                        COUNT(*) as transaction_count,
                        AVG(LEAD(b.timestamp) OVER (ORDER BY b.timestamp) - b.timestamp) as avg_time_between
                    FROM coins c
                    INNER JOIN blocks b ON c.created_block = b.height
                    WHERE c.puzzle_hash = $1
                    GROUP BY EXTRACT(hour FROM TO_TIMESTAMP(b.timestamp)), EXTRACT(dow FROM TO_TIMESTAMP(b.timestamp))
                ),
                summary AS (
                    SELECT 
                        mode() WITHIN GROUP (ORDER BY hour_of_day) as most_active_hour,
                        mode() WITHIN GROUP (ORDER BY day_of_week) as most_active_day,
                        AVG(avg_time_between) as avg_time_between_transactions,
                        MAX(avg_time_between) as longest_inactive_period
                    FROM time_analysis
                )
                SELECT * FROM summary
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH time_analysis AS (
                    SELECT 
                        CAST(strftime('%H', datetime(b.timestamp, 'unixepoch')) AS INTEGER) as hour_of_day,
                        CAST(strftime('%w', datetime(b.timestamp, 'unixepoch')) AS INTEGER) as day_of_week,
                        COUNT(*) as transaction_count
                    FROM coins c
                    INNER JOIN blocks b ON c.created_block = b.height
                    WHERE c.puzzle_hash = ?
                    GROUP BY strftime('%H', datetime(b.timestamp, 'unixepoch')), strftime('%w', datetime(b.timestamp, 'unixepoch'))
                ),
                summary AS (
                    SELECT 
                        (SELECT hour_of_day FROM time_analysis ORDER BY transaction_count DESC LIMIT 1) as most_active_hour,
                        (SELECT day_of_week FROM time_analysis ORDER BY transaction_count DESC LIMIT 1) as most_active_day,
                        3600.0 as avg_time_between_transactions,
                        86400.0 as longest_inactive_period
                    FROM time_analysis
                    LIMIT 1
                )
                SELECT * FROM summary
                "#
            }
        };

        let row = sqlx::query(query)
            .bind(&puzzle_hash)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(row.map(|r| AddressTimeAnalysis {
            most_active_hour: r.get::<i32, _>("most_active_hour") as u32,
            most_active_day_of_week: r.get::<i32, _>("most_active_day") as u32,
            avg_time_between_transactions: r.get::<f64, _>("avg_time_between_transactions"),
            longest_inactive_period: r.get::<f64, _>("longest_inactive_period"),
            weekly_activity_pattern: vec![], // Simplified for now
            hourly_activity_pattern: vec![], // Simplified for now
        }))
    }
} 