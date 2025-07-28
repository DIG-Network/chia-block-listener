use async_graphql::*;
use serde::{Deserialize, Serialize};

/// Spend velocity analysis
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct SpendVelocityAnalysis {
    /// Average blocks to spend
    pub avg_blocks_to_spend: f64,
    /// Median blocks to spend
    pub median_blocks_to_spend: Option<f64>,
    /// Immediate spends (spent in same block)
    pub immediate_spends: i64,
    /// Quick spends (within 10 blocks)
    pub quick_spends: i64,
    /// Medium spends (10-100 blocks)
    pub medium_spends: i64,
    /// Slow spends (100+ blocks)
    pub slow_spends: i64,
    /// Total coins analyzed
    pub total_coins_analyzed: i64,
    /// Total value spent
    pub total_value_spent: u64,
}

/// Complex transaction pattern
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ComplexTransactionPattern {
    /// Spent block
    pub spent_block: i64,
    /// Block timestamp
    pub block_timestamp: String,
    /// Number of coins spent
    pub coins_spent: i64,
    /// Number of coins created
    pub coins_created: i64,
    /// Total value in
    pub total_value_in: u64,
    /// Total value out
    pub total_value_out: u64,
    /// Fee amount
    pub fee_amount: u64,
    /// Unique puzzles used
    pub unique_puzzles_used: i64,
    /// Unique solutions used
    pub unique_solutions_used: i64,
    /// Transaction complexity score
    pub complexity_score: f64,
}

/// Spending frequency by address
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct SpendingFrequencyByAddress {
    /// Puzzle hash hex
    pub puzzle_hash_hex: String,
    /// Total coins spent
    pub total_coins_spent: i64,
    /// Total value spent
    pub total_value_spent: u64,
    /// First spend block
    pub first_spend_block: i64,
    /// Last spend block
    pub last_spend_block: i64,
    /// Active spending blocks
    pub active_spending_blocks: i64,
    /// Average spends per block
    pub avg_spends_per_block: f64,
    /// Spending frequency score
    pub spending_frequency_score: f64,
}

/// Transaction fee analysis
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct TransactionFeeAnalysis {
    /// Time period
    pub time_period: String,
    /// Total fees collected
    pub total_fees: u64,
    /// Average fee per transaction
    pub avg_fee_per_transaction: f64,
    /// Median fee
    pub median_fee: Option<f64>,
    /// Min fee
    pub min_fee: u64,
    /// Max fee
    pub max_fee: u64,
    /// Fee standard deviation
    pub fee_stddev: Option<f64>,
    /// Zero fee transactions
    pub zero_fee_transactions: i64,
    /// High fee transactions (>1 XCH)
    pub high_fee_transactions: i64,
}

/// Transaction pattern by type
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct TransactionPatternByType {
    /// Transaction type
    pub transaction_type: String,
    /// Count
    pub count: i64,
    /// Average value in
    pub avg_value_in: f64,
    /// Average value out
    pub avg_value_out: f64,
    /// Average fee
    pub avg_fee: f64,
}

/// Transaction throughput
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct TransactionThroughput {
    /// Time bucket
    pub time_bucket: String,
    /// Transactions count
    pub transactions_count: i64,
    /// Coins moved
    pub coins_moved: i64,
    /// Value moved
    pub value_moved: u64,
    /// Unique addresses
    pub unique_addresses: i64,
    /// TPS (transactions per second)
    pub tps: f64,
    /// Peak TPS in period
    pub peak_tps: f64,
} 