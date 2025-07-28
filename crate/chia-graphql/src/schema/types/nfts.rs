use async_graphql::*;
use serde::{Deserialize, Serialize};

/// NFT information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Nft {
    /// Coin ID
    pub coin_id: String,
    /// Launcher ID
    pub launcher_id: String,
    /// Collection ID
    pub collection_id: Option<String>,
    /// Collection name
    pub collection_name: Option<String>,
    /// Edition number
    pub edition_number: Option<i32>,
    /// Edition total
    pub edition_total: Option<i32>,
    /// Data URIs
    pub data_uris: Vec<String>,
    /// Metadata URIs
    pub metadata_uris: Vec<String>,
    /// Royalty basis points
    pub royalty_basis_points: Option<i32>,
    /// Created block
    pub created_block: i64,
    /// Is spent
    pub is_spent: bool,
}

/// NFT with owner information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct NftWithOwner {
    /// Coin ID
    pub coin_id: String,
    /// Launcher ID
    pub launcher_id: String,
    /// Owner puzzle hash hex
    pub owner_puzzle_hash_hex: String,
    /// Current owner
    pub current_owner: Option<String>,
    /// Edition number
    pub edition_number: Option<i32>,
    /// Edition total
    pub edition_total: Option<i32>,
    /// Data URIs
    pub data_uris: Vec<String>,
    /// Metadata URIs
    pub metadata_uris: Vec<String>,
    /// Royalty basis points
    pub royalty_basis_points: Option<i32>,
    /// Created block
    pub created_block: i64,
    /// Is spent
    pub is_spent: bool,
}

/// NFT collection information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct NftCollection {
    /// Collection ID
    pub collection_id: String,
    /// Name
    pub name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Total supply
    pub total_supply: Option<i32>,
    /// Creator puzzle hash
    pub creator_puzzle_hash: String,
    /// Metadata JSON
    pub metadata_json: Option<String>,
    /// First seen block
    pub first_seen_block: Option<i64>,
    /// Creation timestamp
    pub created_at: String,
}

/// NFT ownership history entry
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct NftOwnershipHistory {
    /// History ID
    pub id: i32,
    /// Launcher ID
    pub launcher_id: String,
    /// Coin ID
    pub coin_id: String,
    /// Collection ID
    pub collection_id: Option<String>,
    /// Previous owner puzzle hash hex
    pub previous_owner_puzzle_hash_hex: String,
    /// New owner puzzle hash hex
    pub new_owner_puzzle_hash_hex: String,
    /// Previous owner
    pub previous_owner: Option<String>,
    /// New owner
    pub new_owner: Option<String>,
    /// Transfer type
    pub transfer_type: String,
    /// Block height
    pub block_height: i64,
    /// Block timestamp
    pub block_timestamp: String,
    /// Spend coin ID
    pub spend_coin_id: Option<String>,
    /// Creation timestamp
    pub created_at: String,
}

/// NFT ownership by address
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct NftOwnershipByAddress {
    /// Launcher ID
    pub launcher_id: String,
    /// Coin ID
    pub coin_id: String,
    /// Collection ID
    pub collection_id: Option<String>,
    /// Transfer type
    pub transfer_type: String,
    /// Block height
    pub block_height: i64,
    /// Block timestamp
    pub block_timestamp: String,
    /// Was previous owner
    pub was_previous_owner: bool,
    /// Was new owner
    pub was_new_owner: bool,
}

/// NFT owner at block
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct NftOwnerAtBlock {
    /// Owner puzzle hash hex
    pub owner_puzzle_hash_hex: String,
    /// Current owner
    pub current_owner: Option<String>,
    /// Block height
    pub block_height: i64,
    /// Transfer type
    pub transfer_type: String,
}

/// NFT transfer
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct NftTransfer {
    /// Launcher ID
    pub launcher_id: String,
    /// Coin ID
    pub coin_id: String,
    /// Collection ID
    pub collection_id: Option<String>,
    /// Previous owner puzzle hash hex
    pub previous_owner_puzzle_hash_hex: String,
    /// New owner puzzle hash hex
    pub new_owner_puzzle_hash_hex: String,
    /// Transfer type
    pub transfer_type: String,
    /// Block height
    pub block_height: i64,
    /// Block timestamp
    pub block_timestamp: String,
}

/// NFT collection analytics
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct CollectionAnalytics {
    pub collection_id: String,
    pub collection_name: Option<String>,
    pub total_nfts: i64,
    pub unique_owners: i64,
    pub total_unique_holders: i64,
    pub total_transfers: i64,
    pub max_supply: Option<i32>,
    pub first_mint_block: i64,
    pub last_mint_block: i64,
    pub avg_royalty_basis_points: Option<f64>,
    pub current_nfts: i64,
    pub total_sales: i64,
    pub first_transfer_block: Option<i64>,
    pub last_transfer_block: Option<i64>,
    pub blocks_since_last_activity: Option<i64>,
    pub ownership_distribution_percent: f64,
    pub sale_ratio_percent: f64,
}

/// NFT ownership concentration
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct OwnershipConcentration {
    pub owner_address: String,
    pub nfts_owned: i64,
    pub unique_editions: i64,
    pub first_acquired: i64,
    pub last_acquired: i64,
    pub ownership_percentage: f64,
    pub ownership_rank: i64,
    pub owned_nft_ids: String,
}

/// NFT trading velocity
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct TradingVelocity {
    pub collection_id: String,
    pub days_analyzed: i32,
    pub traded_nfts: i64,
    pub total_transfers: i64,
    pub total_sales: i64,
    pub unique_buyers: i64,
    pub unique_sellers: i64,
    pub total_collection_size: i64,
    pub velocity_percentage: f64,
    pub sale_transfer_ratio: f64,
    pub avg_transfers_per_sale: f64,
    pub avg_hold_time_blocks: Option<f64>,
    pub recently_traded: i64,
}

/// NFT rarity analysis
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct RarityAnalysis {
    pub launcher_id: String,
    pub edition_number: Option<i32>,
    pub edition_total: Option<i32>,
    pub current_owner: String,
    pub data_uri_count: Option<i32>,
    pub metadata_uri_count: Option<i32>,
    pub edition_percentile: f64,
    pub transfer_count: i64,
    pub last_transfer_block: Option<i64>,
    pub is_current: bool,
    pub rarity_tier: String,
}

/// NFT collection comparison
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct CollectionComparison {
    pub collection_id: String,
    pub collection_name: Option<String>,
    pub total_nfts: i64,
    pub unique_owners: i64,
    pub total_transfers: i64,
    pub total_sales: i64,
    pub avg_royalty: Option<f64>,
    pub first_mint: i64,
    pub last_activity: Option<i64>,
    pub weekly_transfers: i64,
    pub monthly_transfers: i64,
    pub ownership_distribution: f64,
    pub sale_ratio: f64,
    pub activity_rank: i64,
    pub size_rank: i64,
    pub popularity_rank: i64,
}

/// NFT creator analytics
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct CreatorAnalytics {
    pub creator_address: String,
    pub collections_created: i64,
    pub total_nfts_created: i64,
    pub total_supply_across_collections: Option<i64>,
    pub avg_royalty_rate: Option<f64>,
    pub first_collection_block: Option<i64>,
    pub latest_collection_block: Option<i64>,
    pub total_transfers_all_collections: i64,
    pub total_sales_all_collections: i64,
    pub overall_sale_ratio: f64,
    pub estimated_royalty_earnings: f64,
} 