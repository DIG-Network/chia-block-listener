use async_graphql::*;
use serde::{Deserialize, Serialize};

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct NetworkStats {
    /// Total number of blocks
    pub total_blocks: i64,
    /// Total number of coins
    pub total_coins: i64,
    /// Total number of addresses
    pub total_addresses: i64,
    /// Total value in circulation
    pub total_value: u64,
    /// Average block time in seconds
    pub avg_block_time_seconds: f64,
    /// Average coins per block
    pub avg_coins_per_block: f64,
    /// Average value per block
    pub avg_value_per_block: f64,
    /// Number of unique puzzles
    pub unique_puzzles: i64,
    /// Number of unique solutions
    pub unique_solutions: i64,
    /// Current blockchain height
    pub current_height: i64,
}

/// Coin type distribution
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct CoinTypeDistribution {
    /// Coin type
    pub coin_type: String,
    /// Number of coins
    pub coin_count: i64,
    /// Total value
    pub total_value: u64,
    /// Average coin size
    pub avg_coin_size: u64,
    /// Coin count percentage
    pub coin_count_percentage: f64,
    /// Value percentage
    pub value_percentage: f64,
}

/// Block production statistics
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct BlockProductionStats {
    /// Average block time in seconds
    pub avg_block_time_seconds: f64,
    /// Minimum block time
    pub min_block_time_seconds: i64,
    /// Maximum block time
    pub max_block_time_seconds: i64,
    /// Median block time
    pub median_block_time: Option<f64>,
    /// Block time standard deviation
    pub block_time_stddev: Option<f64>,
    /// Blocks produced today
    pub blocks_today: i64,
    /// Blocks produced this week
    pub blocks_this_week: i64,
    /// Blocks produced this month
    pub blocks_this_month: i64,
}

/// Network growth metrics
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct NetworkGrowthMetrics {
    /// Date of the metric
    pub metric_date: String,
    /// Total blocks on this date
    pub total_blocks: i64,
    /// New blocks created
    pub blocks_created: i64,
    /// Total unique addresses
    pub unique_addresses: i64,
    /// New addresses created
    pub new_addresses: i64,
    /// Total coins
    pub total_coins: i64,
    /// New coins created
    pub new_coins: i64,
    /// Total value
    pub total_value: u64,
    /// Value created
    pub value_created: u64,
}

/// Active addresses over time
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ActiveAddressesOverTime {
    /// Time period
    pub time_period: String,
    /// Active addresses
    pub active_addresses: i64,
    /// New addresses
    pub new_addresses: i64,
    /// Addresses with spends
    pub addresses_with_spends: i64,
    /// Addresses with receives
    pub addresses_with_receives: i64,
}

/// Blockchain activity metrics
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct BlockchainActivityMetrics {
    /// Hour of day (0-23)
    pub hour_of_day: i32,
    /// Day of week (0-6)
    pub day_of_week: i32,
    /// Average blocks per hour
    pub avg_blocks_per_hour: f64,
    /// Average transactions per hour
    pub avg_transactions_per_hour: f64,
    /// Average value moved per hour
    pub avg_value_moved_per_hour: f64,
    /// Peak activity score
    pub peak_activity_score: f64,
}

/// Unspent coin analysis (Chia coinset model)
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct UnspentCoinAnalysis {
    /// Total unspent coins
    pub total_unspent_coins: i64,
    /// Total unspent coin value
    pub total_unspent_value: u64,
    /// Average coin value
    pub avg_coin_value: f64,
    /// Median coin value
    pub median_coin_value: Option<f64>,
    /// Dust coins (< 1 mojo)
    pub dust_coins: i64,
    /// Small coins (< 0.01 XCH)
    pub small_coins: i64,
    /// Medium coins (0.01 - 1 XCH)
    pub medium_coins: i64,
    /// Large coins (> 1 XCH)
    pub large_coins: i64,
    /// Average coin age in blocks
    pub avg_coin_age_blocks: f64,
}

/// Throughput analysis
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ThroughputAnalysis {
    /// Time period
    pub time_period: String,
    /// Transactions per second
    pub tps: f64,
    /// Coins created per second
    pub coins_per_second: f64,
    /// Value moved per second
    pub value_per_second: f64,
    /// Blocks per hour
    pub blocks_per_hour: f64,
    /// Peak TPS
    pub peak_tps: f64,
    /// Average TPS
    pub avg_tps: f64,
} 