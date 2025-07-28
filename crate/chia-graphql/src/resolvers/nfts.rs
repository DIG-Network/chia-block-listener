use async_graphql::*;
use sqlx::{AnyPool, Row};
use std::sync::Arc;

use crate::database::DatabaseType;
use crate::error::{GraphQLError, GraphQLResult};
use crate::schema::types::*;

/// NFT (Non-Fungible Token) queries
pub struct NftQueries {
    pool: Arc<AnyPool>,
    db_type: DatabaseType,
}

impl NftQueries {
    pub fn new(pool: Arc<AnyPool>, db_type: DatabaseType) -> Self {
        Self { pool, db_type }
    }
}

#[Object]
impl NftQueries {
    /// Get NFTs by owner puzzle hash
    async fn by_owner(
        &self,
        owner_puzzle_hash: String,
        collection_id: Option<String>,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<Nft>> {
        let owner_puzzle_hash_bytes = hex::decode(owner_puzzle_hash.trim_start_matches("0x"))
            .map_err(|_| GraphQLError::InvalidInput("Invalid hex string".to_string()))?;

        let (query, has_collection_filter) = match self.db_type {
            DatabaseType::Postgres => {
                let query = if collection_id.is_some() {
                    r#"
                    SELECT 
                      n.coin_id,
                      n.launcher_id,
                      n.collection_id,
                      nc.name as collection_name,
                      n.edition_number,
                      n.edition_total,
                      COALESCE(n.data_uris::text, '[]') as data_uris_str,
                      COALESCE(n.metadata_uris::text, '[]') as metadata_uris_str,
                      n.royalty_basis_points,
                      n.created_block,
                      EXISTS(SELECT 1 FROM spends WHERE spends.coin_id = n.coin_id) as is_spent
                    FROM nfts n
                    LEFT JOIN nft_collections nc ON n.collection_id = nc.collection_id
                    WHERE n.owner_puzzle_hash = $1 AND n.collection_id = $2
                    ORDER BY n.created_block DESC, n.edition_number
                    LIMIT $3 OFFSET $4
                    "#
                } else {
                    r#"
                    SELECT 
                      n.coin_id,
                      n.launcher_id,
                      n.collection_id,
                      nc.name as collection_name,
                      n.edition_number,
                      n.edition_total,
                      COALESCE(n.data_uris::text, '[]') as data_uris_str,
                      COALESCE(n.metadata_uris::text, '[]') as metadata_uris_str,
                      n.royalty_basis_points,
                      n.created_block,
                      EXISTS(SELECT 1 FROM spends WHERE spends.coin_id = n.coin_id) as is_spent
                    FROM nfts n
                    LEFT JOIN nft_collections nc ON n.collection_id = nc.collection_id
                    WHERE n.owner_puzzle_hash = $1
                    ORDER BY n.created_block DESC, n.edition_number
                    LIMIT $2 OFFSET $3
                    "#
                };
                (query, collection_id.is_some())
            }
            DatabaseType::Sqlite => {
                let query = if collection_id.is_some() {
                    r#"
                    SELECT 
                      n.coin_id,
                      n.launcher_id,
                      n.collection_id,
                      nc.name as collection_name,
                      n.edition_number,
                      n.edition_total,
                      COALESCE(n.data_uris, '[]') as data_uris_str,
                      COALESCE(n.metadata_uris, '[]') as metadata_uris_str,
                      n.royalty_basis_points,
                      n.created_block,
                      EXISTS(SELECT 1 FROM spends WHERE spends.coin_id = n.coin_id) as is_spent
                    FROM nfts n
                    LEFT JOIN nft_collections nc ON n.collection_id = nc.collection_id
                    WHERE n.owner_puzzle_hash = ? AND n.collection_id = ?
                    ORDER BY n.created_block DESC, n.edition_number
                    LIMIT ? OFFSET ?
                    "#
                } else {
                    r#"
                    SELECT 
                      n.coin_id,
                      n.launcher_id,
                      n.collection_id,
                      nc.name as collection_name,
                      n.edition_number,
                      n.edition_total,
                      COALESCE(n.data_uris, '[]') as data_uris_str,
                      COALESCE(n.metadata_uris, '[]') as metadata_uris_str,
                      n.royalty_basis_points,
                      n.created_block,
                      EXISTS(SELECT 1 FROM spends WHERE spends.coin_id = n.coin_id) as is_spent
                    FROM nfts n
                    LEFT JOIN nft_collections nc ON n.collection_id = nc.collection_id
                    WHERE n.owner_puzzle_hash = ?
                    ORDER BY n.created_block DESC, n.edition_number
                    LIMIT ? OFFSET ?
                    "#
                };
                (query, collection_id.is_some())
            }
        };

        let offset = (page - 1) * page_size;
        
        let rows = if has_collection_filter {
            sqlx::query(query)
                .bind(&owner_puzzle_hash_bytes)
                .bind(collection_id.as_ref().unwrap())
                .bind(page_size)
                .bind(offset)
                .fetch_all(&*self.pool)
                .await?
        } else {
            sqlx::query(query)
                .bind(&owner_puzzle_hash_bytes)
                .bind(page_size)
                .bind(offset)
                .fetch_all(&*self.pool)
                .await?
        };

        let mut results = Vec::new();
        for row in rows {
            let data_uris_str: String = row.get("data_uris_str");
            let metadata_uris_str: String = row.get("metadata_uris_str");
            
            let data_uris: Vec<String> = serde_json::from_str(&data_uris_str).unwrap_or_default();
            let metadata_uris: Vec<String> = serde_json::from_str(&metadata_uris_str).unwrap_or_default();

            results.push(Nft {
                coin_id: row.get("coin_id"),
                launcher_id: row.get("launcher_id"),
                collection_id: row.get("collection_id"),
                collection_name: row.get("collection_name"),
                edition_number: row.get("edition_number"),
                edition_total: row.get("edition_total"),
                data_uris,
                metadata_uris,
                royalty_basis_points: row.get("royalty_basis_points"),
                created_block: row.get("created_block"),
                is_spent: row.get("is_spent"),
            });
        }

        Ok(results)
    }

    /// Get NFTs by collection ID
    async fn by_collection(
        &self,
        collection_id: String,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<Nft>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                  n.coin_id,
                  n.launcher_id,
                  n.collection_id,
                  nc.name as collection_name,
                  n.edition_number,
                  n.edition_total,
                  COALESCE(n.data_uris::text, '[]') as data_uris_str,
                  COALESCE(n.metadata_uris::text, '[]') as metadata_uris_str,
                  n.royalty_basis_points,
                  n.created_block,
                  EXISTS(SELECT 1 FROM spends WHERE spends.coin_id = n.coin_id) as is_spent
                FROM nfts n
                LEFT JOIN nft_collections nc ON n.collection_id = nc.collection_id
                WHERE n.collection_id = $1
                ORDER BY n.created_block DESC, n.edition_number
                LIMIT $2 OFFSET $3
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                  n.coin_id,
                  n.launcher_id,
                  n.collection_id,
                  nc.name as collection_name,
                  n.edition_number,
                  n.edition_total,
                  COALESCE(n.data_uris, '[]') as data_uris_str,
                  COALESCE(n.metadata_uris, '[]') as metadata_uris_str,
                  n.royalty_basis_points,
                  n.created_block,
                  EXISTS(SELECT 1 FROM spends WHERE spends.coin_id = n.coin_id) as is_spent
                FROM nfts n
                LEFT JOIN nft_collections nc ON n.collection_id = nc.collection_id
                WHERE n.collection_id = ?
                ORDER BY n.created_block DESC, n.edition_number
                LIMIT ? OFFSET ?
                "#
            }
        };

        let offset = (page - 1) * page_size;
        
        let rows = sqlx::query(query)
            .bind(&collection_id)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            let data_uris_str: String = row.get("data_uris_str");
            let metadata_uris_str: String = row.get("metadata_uris_str");
            
            let data_uris: Vec<String> = serde_json::from_str(&data_uris_str).unwrap_or_default();
            let metadata_uris: Vec<String> = serde_json::from_str(&metadata_uris_str).unwrap_or_default();

            results.push(Nft {
                coin_id: row.get("coin_id"),
                launcher_id: row.get("launcher_id"),
                collection_id: row.get("collection_id"),
                collection_name: row.get("collection_name"),
                edition_number: row.get("edition_number"),
                edition_total: row.get("edition_total"),
                data_uris,
                metadata_uris,
                royalty_basis_points: row.get("royalty_basis_points"),
                created_block: row.get("created_block"),
                is_spent: row.get("is_spent"),
            });
        }

        Ok(results)
    }

    /// Get collection by ID
    async fn collection_by_id(&self, collection_id: String) -> GraphQLResult<Option<NftCollection>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                "SELECT *, metadata_json::text as metadata_json_str, created_at::text as created_at_str FROM nft_collections WHERE collection_id = $1"
            }
            DatabaseType::Sqlite => {
                "SELECT *, metadata_json as metadata_json_str, created_at as created_at_str FROM nft_collections WHERE collection_id = ?"
            }
        };

        let row = sqlx::query(query)
            .bind(&collection_id)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(row.map(|r| NftCollection {
            collection_id: r.get("collection_id"),
            name: r.get("name"),
            description: r.get("description"),
            total_supply: r.get("total_supply"),
            creator_puzzle_hash: r.get("creator_puzzle_hash"),
            metadata_json: r.get("metadata_json_str"),
            first_seen_block: r.get("first_seen_block"),
            created_at: r.get::<String, _>("created_at_str"),
        }))
    }

    /// Get ownership history for an NFT
    async fn ownership_history(
        &self,
        launcher_id: String,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<NftOwnershipHistory>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                  h.id,
                  h.launcher_id,
                  h.coin_id,
                  h.collection_id,
                  encode(h.old_owner_puzzle_hash, 'hex') as previous_owner_puzzle_hash_hex,
                  encode(h.new_owner_puzzle_hash, 'hex') as new_owner_puzzle_hash_hex,
                  NULL as previous_owner,
                  NULL as new_owner,
                  h.transfer_type,
                  h.block_height,
                  h.timestamp::text as block_timestamp,
                  h.old_nft_coin_id as spend_coin_id,
                  h.timestamp::text as created_at
                FROM nft_ownership_history h
                WHERE h.launcher_id = $1
                ORDER BY h.block_height DESC, h.id
                LIMIT $2 OFFSET $3
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                  h.id,
                  h.launcher_id,
                  h.coin_id,
                  h.collection_id,
                  hex(h.old_owner_puzzle_hash) as previous_owner_puzzle_hash_hex,
                  hex(h.new_owner_puzzle_hash) as new_owner_puzzle_hash_hex,
                  NULL as previous_owner,
                  NULL as new_owner,
                  h.transfer_type,
                  h.block_height,
                  h.timestamp as block_timestamp,
                  h.old_nft_coin_id as spend_coin_id,
                  h.timestamp as created_at
                FROM nft_ownership_history h
                WHERE h.launcher_id = ?
                ORDER BY h.block_height DESC, h.id
                LIMIT ? OFFSET ?
                "#
            }
        };

        let offset = (page - 1) * page_size;
        
        let rows = sqlx::query(query)
            .bind(&launcher_id)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(NftOwnershipHistory {
                id: row.get("id"),
                launcher_id: row.get("launcher_id"),
                coin_id: row.get("coin_id"),
                collection_id: row.get("collection_id"),
                previous_owner_puzzle_hash_hex: row.get("previous_owner_puzzle_hash_hex"),
                new_owner_puzzle_hash_hex: row.get("new_owner_puzzle_hash_hex"),
                previous_owner: row.get("previous_owner"),
                new_owner: row.get("new_owner"),
                transfer_type: row.get("transfer_type"),
                block_height: row.get("block_height"),
                block_timestamp: row.get::<String, _>("block_timestamp"),
                spend_coin_id: row.get("spend_coin_id"),
                created_at: row.get::<String, _>("created_at"),
            });
        }

        Ok(results)
    }

    /// Get comprehensive collection analytics
    async fn collection_analytics(&self, collection_id: String) -> GraphQLResult<Option<CollectionAnalytics>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH collection_stats AS (
                  SELECT 
                    n.collection_id,
                    nc.name as collection_name,
                    COUNT(DISTINCT n.launcher_id) as total_nfts,
                    COUNT(DISTINCT n.owner_puzzle_hash) as unique_owners,
                    COUNT(DISTINCT h.new_owner_puzzle_hash) as total_unique_holders,
                    COUNT(DISTINCT h.id) as total_transfers,
                    MAX(n.edition_total) as max_supply,
                    MIN(n.created_block) as first_mint_block,
                    MAX(n.created_block) as last_mint_block,
                    AVG(n.royalty_basis_points) as avg_royalty_basis_points,
                    COUNT(DISTINCT CASE WHEN EXISTS(SELECT 1 FROM spends WHERE coin_id = n.coin_id) THEN NULL ELSE n.launcher_id END) as current_nfts,
                    COUNT(DISTINCT CASE WHEN h.transfer_type = 'sale' THEN h.id END) as total_sales,
                    MIN(h.block_height) as first_transfer_block,
                    MAX(h.block_height) as last_transfer_block
                  FROM nfts n
                  LEFT JOIN nft_collections nc ON n.collection_id = nc.collection_id
                  LEFT JOIN nft_ownership_history h ON n.launcher_id = h.launcher_id
                  WHERE n.collection_id = $1
                  GROUP BY n.collection_id, nc.name
                ),
                current_height AS (
                  SELECT MAX(height) as max_height FROM blocks
                )
                SELECT 
                  cs.*,
                  (ch.max_height - cs.last_transfer_block) as blocks_since_last_activity,
                  CASE 
                    WHEN cs.total_nfts > 0 THEN ROUND((cs.unique_owners::NUMERIC / cs.total_nfts::NUMERIC) * 100, 2)
                    ELSE 0
                  END as ownership_distribution_percent,
                  CASE 
                    WHEN cs.total_transfers > 0 THEN ROUND((cs.total_sales::NUMERIC / cs.total_transfers::NUMERIC) * 100, 2)
                    ELSE 0
                  END as sale_ratio_percent
                FROM collection_stats cs, current_height ch
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH collection_stats AS (
                  SELECT 
                    n.collection_id,
                    nc.name as collection_name,
                    COUNT(DISTINCT n.launcher_id) as total_nfts,
                    COUNT(DISTINCT n.owner_puzzle_hash) as unique_owners,
                    COUNT(DISTINCT h.new_owner_puzzle_hash) as total_unique_holders,
                    COUNT(DISTINCT h.id) as total_transfers,
                    MAX(n.edition_total) as max_supply,
                    MIN(n.created_block) as first_mint_block,
                    MAX(n.created_block) as last_mint_block,
                    AVG(n.royalty_basis_points) as avg_royalty_basis_points,
                    COUNT(DISTINCT CASE WHEN EXISTS(SELECT 1 FROM spends WHERE coin_id = n.coin_id) THEN NULL ELSE n.launcher_id END) as current_nfts,
                    COUNT(DISTINCT CASE WHEN h.transfer_type = 'sale' THEN h.id END) as total_sales,
                    MIN(h.block_height) as first_transfer_block,
                    MAX(h.block_height) as last_transfer_block
                  FROM nfts n
                  LEFT JOIN nft_collections nc ON n.collection_id = nc.collection_id
                  LEFT JOIN nft_ownership_history h ON n.launcher_id = h.launcher_id
                  WHERE n.collection_id = ?
                  GROUP BY n.collection_id, nc.name
                ),
                current_height AS (
                  SELECT MAX(height) as max_height FROM blocks
                )
                SELECT 
                  cs.*,
                  (ch.max_height - cs.last_transfer_block) as blocks_since_last_activity,
                  CASE 
                    WHEN cs.total_nfts > 0 THEN ROUND((CAST(cs.unique_owners AS REAL) / CAST(cs.total_nfts AS REAL)) * 100, 2)
                    ELSE 0
                  END as ownership_distribution_percent,
                  CASE 
                    WHEN cs.total_transfers > 0 THEN ROUND((CAST(cs.total_sales AS REAL) / CAST(cs.total_transfers AS REAL)) * 100, 2)
                    ELSE 0
                  END as sale_ratio_percent
                FROM collection_stats cs, current_height ch
                "#
            }
        };

        let row = sqlx::query(query)
            .bind(&collection_id)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(row.map(|r| CollectionAnalytics {
            collection_id: r.get("collection_id"),
            collection_name: r.get("collection_name"),
            total_nfts: r.get::<i64, _>("total_nfts"),
            unique_owners: r.get::<i64, _>("unique_owners"),
            total_unique_holders: r.get::<i64, _>("total_unique_holders"),
            total_transfers: r.get::<i64, _>("total_transfers"),
            max_supply: r.get("max_supply"),
            first_mint_block: r.get::<i64, _>("first_mint_block"),
            last_mint_block: r.get::<i64, _>("last_mint_block"),
            avg_royalty_basis_points: r.get("avg_royalty_basis_points"),
            current_nfts: r.get::<i64, _>("current_nfts"),
            total_sales: r.get::<i64, _>("total_sales"),
            first_transfer_block: r.get("first_transfer_block"),
            last_transfer_block: r.get("last_transfer_block"),
            blocks_since_last_activity: r.get("blocks_since_last_activity"),
            ownership_distribution_percent: r.get("ownership_distribution_percent"),
            sale_ratio_percent: r.get("sale_ratio_percent"),
        }))
    }
} 