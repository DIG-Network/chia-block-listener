use async_graphql::*;
use serde::{Deserialize, Serialize};

/// CAT asset information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct CatAsset {
    /// Asset ID
    pub asset_id: String,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token name
    pub name: Option<String>,
    /// Token description
    pub description: Option<String>,
    /// Total supply
    pub total_supply: Option<i64>,
    /// Number of decimals
    pub decimals: Option<i32>,
    /// Metadata JSON
    pub metadata_json: Option<String>,
    /// First seen block
    pub first_seen_block: Option<i64>,
    /// Creation timestamp
    pub created_at: String,
}

/// CAT balance information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct CatBalance {
    /// Asset ID
    pub asset_id: String,
    /// Token symbol
    pub symbol: Option<String>,
    /// Asset name
    pub asset_name: Option<String>,
    /// Owner puzzle hash in hex
    pub owner_puzzle_hash_hex: String,
    /// Number of coins
    pub coin_count: i64,
    /// Total amount
    pub total_amount: u64,
    /// Total balance (adjusted for decimals)
    pub total_balance: f64,
    /// Minimum coin amount
    pub min_coin_amount: u64,
    /// Maximum coin amount
    pub max_coin_amount: u64,
    /// Average coin amount
    pub avg_coin_amount: u64,
    /// Last activity block
    pub last_activity_block: Option<i64>,
    /// Number of decimals
    pub decimals: Option<i32>,
}

/// CAT coin information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct CatCoin {
    /// Coin ID
    pub coin_id: String,
    /// Asset ID
    pub asset_id: String,
    /// Asset name
    pub asset_name: Option<String>,
    /// Asset symbol
    pub asset_symbol: Option<String>,
    /// Amount
    pub amount: u64,
    /// Inner puzzle hash hex
    pub inner_puzzle_hash_hex: String,
    /// Created block
    pub created_block: i64,
    /// Is spent
    pub is_spent: bool,
}

/// CAT holder information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct CatHolder {
    /// Owner puzzle hash hex
    pub owner_puzzle_hash_hex: String,
    /// Number of coins
    pub coin_count: i64,
    /// Total amount
    pub total_amount: u64,
    /// Total balance
    pub total_balance: f64,
    /// Minimum coin amount
    pub min_coin_amount: u64,
    /// Maximum coin amount
    pub max_coin_amount: u64,
    /// Average coin amount
    pub avg_coin_amount: u64,
    /// Last activity block
    pub last_activity_block: Option<i64>,
}

/// CAT asset summary
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct CatAssetSummary {
    /// Asset ID
    pub asset_id: String,
    /// Symbol
    pub symbol: Option<String>,
    /// Asset name
    pub asset_name: Option<String>,
    /// Number of holders
    pub holder_count: i64,
    /// Total circulating supply
    pub total_circulating_supply: u64,
    /// Total circulating balance
    pub total_circulating_balance: f64,
    /// Top holder amount
    pub top_holder_amount: u64,
    /// Average holder balance
    pub avg_holder_balance: f64,
    /// Decimals
    pub decimals: Option<i32>,
}

/// CAT token analytics
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct TokenAnalytics {
    pub asset_id: String,
    pub symbol: Option<String>,
    pub token_name: Option<String>,
    pub decimals: Option<i32>,
    pub total_supply: Option<i64>,
    pub total_coins: i64,
    pub unique_holders: i64,
    pub current_holders: i64,
    pub total_circulating: u64,
    pub current_supply: u64,
    pub first_mint_block: i64,
    pub last_activity_block: i64,
    pub blocks_since_last_activity: i64,
    pub avg_coin_amount: u64,
    pub min_coin_amount: u64,
    pub max_coin_amount: u64,
    pub supply_utilization_percent: f64,
    pub holder_coin_ratio: f64,
}

/// CAT token distribution
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct TokenDistribution {
    pub holder_address: String,
    pub coins_held: i64,
    pub total_balance: u64,
    pub balance_percentage: f64,
    pub wealth_rank: i64,
    pub first_acquired: i64,
    pub last_acquired: i64,
    pub avg_coin_size: u64,
    pub holder_category: String,
}

/// CAT token velocity
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct TokenVelocity {
    pub asset_id: String,
    pub days_analyzed: i32,
    pub coins_spent: i64,
    pub active_blocks: i64,
    pub total_amount_moved: u64,
    pub transfer_coins: i64,
    pub avg_hold_time_blocks: Option<f64>,
    pub unique_spenders: i64,
    pub current_supply: u64,
    pub total_coins: i64,
    pub velocity_percentage: f64,
    pub transfer_ratio_percent: f64,
}

/// CAT holder behavior
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct HolderBehavior {
    pub holder_address: String,
    pub total_coins_received: i64,
    pub total_coins_spent: i64,
    pub total_received: u64,
    pub total_spent: u64,
    pub current_balance: u64,
    pub first_activity: i64,
    pub last_activity: i64,
    pub blocks_since_last_activity: i64,
    pub avg_hold_time: Option<f64>,
    pub active_blocks_receiving: i64,
    pub active_blocks_spending: i64,
    pub behavior_type: String,
    pub spend_ratio_percent: f64,
    pub hold_strategy: String,
}

/// CAT token comparison
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct TokenComparison {
    pub asset_id: String,
    pub symbol: Option<String>,
    pub token_name: Option<String>,
    pub decimals: Option<i32>,
    pub total_coins: i64,
    pub unique_holders: i64,
    pub total_circulating: u64,
    pub current_supply: u64,
    pub first_activity: i64,
    pub last_activity: i64,
    pub coins_spent: i64,
    pub active_trading_blocks: i64,
    pub distribution_score: f64,
    pub activity_score: f64,
    pub market_cap_rank: i64,
    pub holder_rank: i64,
    pub activity_rank: i64,
}

/// CAT liquidity analysis
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct LiquidityAnalysis {
    pub asset_id: String,
    pub total_coins: i64,
    pub large_coins: i64,
    pub dust_coins: i64,
    pub unspent_coins: i64,
    pub total_amount: u64,
    pub liquid_amount: u64,
    pub avg_coin_size: u64,
    pub median_coin_size: Option<u64>,
    pub coin_size_stddev: Option<f64>,
    pub unique_addresses: i64,
    pub active_addresses: i64,
    pub coins_spent_7d: i64,
    pub active_blocks_7d: i64,
    pub amount_moved_7d: u64,
    pub large_coin_ratio: f64,
    pub dust_ratio: f64,
    pub weekly_velocity_percent: f64,
}

/// Weekly velocity analysis showing the percentage of coins that moved
#[derive(Debug, Clone, SimpleObject)]
pub struct WeeklyVelocityAnalysis {
    /// Percentage of supply that moved in the last week
    pub weekly_velocity_percent: f64,
    /// Coins spent in last 7 days
    pub coins_spent_7d: i64,
    /// Active blocks in last 7 days
    pub active_blocks_7d: i64,
    /// Amount moved in last 7 days
    pub amount_moved_7d: u64,
}

/// Top holder of a CAT token
#[derive(Debug, Clone, SimpleObject)]
pub struct TopHolder {
    /// Address of the holder
    pub holder_address: String,
    /// Balance held
    pub balance: u64,
    /// Percentage of total supply
    pub balance_percentage: f64,
} 