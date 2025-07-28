use async_graphql::*;
use serde::{Deserialize, Serialize};

/// Combined analytics overview for all blockchain data
#[derive(Debug, Clone, SimpleObject)]
pub struct AnalyticsOverview {
    /// Total number of blocks
    pub total_blocks: i64,
    /// Total number of coins
    pub total_coins: i64,
    /// Total number of unique addresses
    pub total_addresses: i64,
    /// Current blockchain height
    pub current_height: i64,
    /// Network statistics
    pub network_stats: NetworkOverview,
    /// Token statistics
    pub token_stats: TokenOverview,
    /// Address activity summary
    pub address_activity: AddressActivitySummary,
}

/// Network overview statistics
#[derive(Debug, Clone, SimpleObject)]
pub struct NetworkOverview {
    /// Average block time in seconds
    pub avg_block_time: f64,
    /// Current throughput (TPS)
    pub current_tps: f64,
    /// 24-hour transaction volume
    pub daily_transaction_volume: u64,
    /// Network health score (0-100)
    pub health_score: f64,
}

/// Token overview statistics
#[derive(Debug, Clone, SimpleObject)]
pub struct TokenOverview {
    /// Total number of CAT tokens
    pub total_cat_tokens: i64,
    /// Total number of NFT collections
    pub total_nft_collections: i64,
    /// Most active CAT token
    pub most_active_cat: Option<String>,
    /// Most popular NFT collection
    pub most_popular_nft_collection: Option<String>,
}

/// Address activity summary
#[derive(Debug, Clone, SimpleObject)]
pub struct AddressActivitySummary {
    /// Active addresses in last 24 hours
    pub daily_active_addresses: i64,
    /// New addresses created today
    pub new_addresses_today: i64,
    /// Average transactions per active address
    pub avg_transactions_per_address: f64,
    /// Top 1% addresses control percentage
    pub whale_concentration_percent: f64,
}

/// Comprehensive blockchain overview
#[derive(Debug, Clone, SimpleObject)]
pub struct BlockchainOverview {
    /// Total number of blocks
    pub total_blocks: i64,
    /// Total number of coins
    pub total_coins: i64,
    /// Total unique addresses
    pub total_addresses: i64,
    /// Current blockchain height
    pub current_height: i64,
    /// Blocks created in last 24 hours
    pub daily_blocks: i64,
    /// Coins created in last 24 hours
    pub daily_coins: i64,
    /// Average blocks until coin is spent
    pub avg_blocks_to_spend: f64,
    /// Network activity score (0-100)
    pub network_activity_score: f64,
}

/// Blockchain health metrics
#[derive(Debug, Clone, SimpleObject)]
pub struct HealthMetrics {
    /// Overall health score (0-100)
    pub overall_health_score: f64,
    /// Block production health (0-100)
    pub block_time_health: f64,
    /// Network throughput health (0-100)
    pub throughput_health: f64,
    /// Fee market health (0-100)
    pub fee_health: f64,
    /// Coinset health (0-100)
    pub coinset_health: f64,
    /// Transactions per second (24h average)
    pub tps_24h: f64,
    /// Average fee (24h)
    pub avg_fee_24h: f64,
}

/// Performance metrics over time
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct PerformanceMetrics {
    /// Time period
    pub time_period: String,
    /// Average TPS
    pub avg_tps: f64,
    /// Peak TPS
    pub peak_tps: f64,
    /// Average block time
    pub avg_block_time: f64,
    /// Block time variance
    pub block_time_variance: f64,
    /// Network utilization percentage
    pub network_utilization: f64,
}

/// Security metrics
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct SecurityMetrics {
    /// Network hash rate indicator
    pub network_strength_score: f64,
    /// Block finality confidence
    pub finality_confidence: f64,
    /// Consensus health score
    pub consensus_health: f64,
    /// Active validator count estimate
    pub active_validators_estimate: i64,
}

/// Economic metrics
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct EconomicMetrics {
    /// Inflation rate (annual)
    pub inflation_rate: f64,
    /// Total fees paid (24h)
    pub total_fees_24h: u64,
    /// Average transaction value
    pub avg_transaction_value: f64,
    /// Coin velocity (annual)
    pub coin_velocity: f64,
    /// Economic activity score
    pub economic_activity_score: f64,
}

/// Growth trend data point
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject, Default)]
pub struct GrowthTrendDataPoint {
    /// Period identifier
    pub period: String,
    /// Block count in period
    pub block_count: i64,
    /// Coin count in period
    pub coin_count: i64,
    /// Transaction count in period
    pub transaction_count: i64,
    /// Value moved in period
    pub value_moved: u64,
    /// Unique addresses in period
    pub unique_addresses: i64,
    /// Growth rate percentage
    pub growth_rate: f64,
    /// Count change from previous period
    pub count_change: i64,
    /// Value change from previous period
    pub value_change: i64,
} 