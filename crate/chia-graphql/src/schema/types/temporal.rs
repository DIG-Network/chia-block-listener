use async_graphql::*;
use serde::{Deserialize, Serialize};

/// Blockchain growth trend
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct BlockchainGrowthTrend {
    /// Trend date
    pub trend_date: String,
    /// Total blocks
    pub total_blocks: i64,
    /// Total coins
    pub total_coins: i64,
    /// New coins
    pub new_coins: i64,
    /// Total addresses
    pub total_addresses: i64,
    /// New addresses
    pub new_addresses: i64,
    /// Total value
    pub total_value: u64,
    /// Daily growth rate
    pub daily_growth_rate: f64,
}

/// Address balance evolution
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct AddressBalanceEvolution {
    /// Block height
    pub block_height: i64,
    /// Block timestamp
    pub block_timestamp: String,
    /// Balance at block
    pub balance_at_block: u64,
    /// Cumulative balance
    pub cumulative_balance: u64,
    /// Change amount
    pub change_amount: i64,
    /// Change type (receive/spend)
    pub change_type: String,
    /// Coin ID involved
    pub coin_id: String,
}

/// Activity heatmap entry
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ActivityHeatmap {
    /// Hour of day (0-23)
    pub hour_of_day: i32,
    /// Day of week (0-6)
    pub day_of_week: i32,
    /// Total transactions
    pub total_transactions: i64,
    /// Total coins moved
    pub total_coins_moved: i64,
    /// Total value moved
    pub total_value_moved: u64,
    /// Average transaction size
    pub avg_transaction_size: f64,
    /// Activity score (0-100)
    pub activity_score: f64,
}

/// Seasonal pattern
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct SeasonalPattern {
    /// Period type (daily, weekly, monthly)
    pub period_type: String,
    /// Period value
    pub period_value: String,
    /// Average activity
    pub avg_activity: f64,
    /// Peak activity
    pub peak_activity: f64,
    /// Low activity
    pub low_activity: f64,
    /// Variance
    pub variance: f64,
    /// Trend direction
    pub trend_direction: String,
}

/// Time-based coin aggregation
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct TimeBasedCoinAggregation {
    /// Time bucket
    pub time_bucket: String,
    /// Coins created
    pub coins_created: i64,
    /// Coins spent
    pub coins_spent: i64,
    /// Net coin change
    pub net_coin_change: i64,
    /// Value created
    pub value_created: u64,
    /// Value spent
    pub value_spent: u64,
    /// Net value change
    pub net_value_change: i64,
    /// Active addresses
    pub active_addresses: i64,
}

/// Mempool evolution
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct MempoolEvolution {
    /// Timestamp
    pub timestamp: String,
    /// Pending transactions
    pub pending_transactions: i64,
    /// Total fees
    pub total_fees: u64,
    /// Average fee per transaction
    pub avg_fee_per_transaction: f64,
    /// Mempool size bytes
    pub mempool_size_bytes: i64,
    /// Oldest transaction age
    pub oldest_transaction_age_seconds: i64,
} 