use async_graphql::*;
use serde::{Deserialize, Serialize};

/// Active address with transaction statistics
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ActiveAddress {
    /// Puzzle hash in hex format
    pub puzzle_hash_hex: String,
    /// Number of coins received
    pub coins_received: i64,
    /// Number of coins spent
    pub coins_spent: i64,
    /// Current balance in mojos
    pub current_balance: u64,
    /// Total amount received
    pub total_received: u64,
    /// First activity block
    pub first_activity_block: i64,
    /// Last activity block
    pub last_activity_block: i64,
    /// Number of blocks with receive activity
    pub active_blocks_received: i64,
    /// Number of blocks with spend activity
    pub active_blocks_spent: i64,
    /// Total number of transactions
    pub total_transactions: i64,
    /// Lifespan in blocks
    pub lifespan_blocks: i64,
}

/// Address behavior patterns across the network
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct AddressBehaviorPatterns {
    /// Total number of addresses
    pub total_addresses: i64,
    /// Number of hodler addresses
    pub hodler_addresses: i64,
    /// Number of transient addresses
    pub transient_addresses: i64,
    /// Number of accumulator addresses
    pub accumulator_addresses: i64,
    /// Number of single-use addresses
    pub single_use_addresses: i64,
    /// Number of whale addresses
    pub whale_addresses: i64,
    /// Average coins received per address
    pub avg_coins_received_per_address: f64,
    /// Average coins spent per address
    pub avg_coins_spent_per_address: f64,
    /// Average address lifespan in blocks
    pub avg_address_lifespan_blocks: f64,
}

/// Dormant address information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct DormantAddress {
    /// Puzzle hash in hex format
    pub puzzle_hash_hex: String,
    /// Last activity block
    pub last_activity_block: i64,
    /// Blocks since last activity
    pub blocks_since_activity: i64,
    /// Current balance in mojos
    pub current_balance: u64,
    /// Total coins received
    pub total_coins_received: i64,
    /// Total coins spent
    pub total_coins_spent: i64,
}

/// New address information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct NewAddress {
    /// Puzzle hash in hex format
    pub puzzle_hash_hex: String,
    /// First appearance block
    pub first_appearance_block: i64,
    /// Number of coins received
    pub coins_received: i64,
    /// Number of coins spent
    pub coins_spent: i64,
    /// Total amount received
    pub total_received: u64,
    /// Current balance
    pub current_balance: u64,
}

/// Address concentration analysis
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct AddressConcentrationAnalysis {
    /// Total addresses with balance
    pub total_addresses_with_balance: i64,
    /// Total supply
    pub total_supply: u64,
    /// Addresses with 1% or more of supply
    pub addresses_with_1_percent_plus: i64,
    /// Addresses with 0.1% or more of supply
    pub addresses_with_point_1_percent_plus: i64,
    /// Balance held by top 10 addresses
    pub top_10_balance: u64,
    /// Balance held by top 100 addresses
    pub top_100_balance: u64,
    /// Balance held by top 1000 addresses
    pub top_1000_balance: u64,
    /// Median balance
    pub median_balance: Option<f64>,
    /// 90th percentile balance
    pub p90_balance: Option<f64>,
    /// 99th percentile balance
    pub p99_balance: Option<f64>,
}

/// Comprehensive address profile
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct AddressProfile {
    /// Puzzle hash in hex format
    pub puzzle_hash_hex: String,
    /// Total coins received
    pub total_coins_received: i64,
    /// Total coins spent
    pub total_coins_spent: i64,
    /// Total value received in mojos
    pub total_value_received: u64,
    /// Current balance in mojos
    pub current_balance: u64,
    /// First activity block
    pub first_activity_block: i64,
    /// Last activity block
    pub last_activity_block: i64,
    /// Blocks since last activity
    pub blocks_since_last_activity: i64,
    /// Lifespan in blocks
    pub lifespan_blocks: i64,
    /// Active blocks with receives
    pub active_blocks_received: i64,
    /// Active blocks with spends
    pub active_blocks_spent: i64,
    /// Total transaction count
    pub total_transactions: i64,
    /// Minimum coin received
    pub min_coin_received: u64,
    /// Maximum coin received
    pub max_coin_received: u64,
    /// Average coin received
    pub avg_coin_received: u64,
    /// Address type classification
    pub address_type: String,
}

/// Address transaction history entry
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct AddressTransaction {
    /// Coin ID
    pub coin_id: String,
    /// Block height
    pub block_height: i64,
    /// Block timestamp
    pub block_timestamp: String,
    /// Amount in mojos
    pub amount: u64,
    /// Transaction type (receive/spend)
    pub transaction_type: String,
    /// Block when spent (if applicable)
    pub spent_block: Option<i64>,
    /// Parent coin ID
    pub parent_coin_id: String,
    /// Address spent to (if applicable)
    pub spent_to_address: Option<String>,
}

/// Address coin flow analysis
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct AddressCoinFlow {
    /// Coins received count
    pub coins_received: i64,
    /// Total received amount
    pub total_received: u64,
    /// Coins spent count
    pub coins_spent: i64,
    /// Total spent amount
    pub total_spent: u64,
    /// Current balance
    pub current_balance: u64,
    /// Net flow (positive = inflow, negative = outflow)
    pub net_flow: i64,
    /// Blocks with receive activity
    pub blocks_received_in: i64,
    /// Blocks with spend activity
    pub blocks_spent_in: i64,
    /// Spend ratio percentage
    pub spend_ratio_percent: f64,
    /// Retention ratio percentage
    pub retention_ratio_percent: f64,
}

/// Address spending behavior analysis
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct AddressSpendingBehavior {
    /// Total number of spends
    pub total_spends: i64,
    /// Average blocks held before spending
    pub avg_blocks_held: f64,
    /// Minimum blocks held
    pub min_blocks_held: i64,
    /// Maximum blocks held
    pub max_blocks_held: i64,
    /// Median blocks held
    pub median_blocks_held: Option<f64>,
    /// Immediate spends (within 1 block)
    pub immediate_spends: i64,
    /// Quick spends (within 10 blocks)
    pub quick_spends: i64,
    /// Long-term holds (over 1000 blocks)
    pub long_term_holds: i64,
    /// Total amount spent
    pub total_amount_spent: u64,
    /// Average spend amount
    pub avg_spend_amount: f64,
    /// Spend amount standard deviation
    pub spend_amount_stddev: Option<f64>,
}

/// Address peer interaction
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct AddressPeer {
    /// Peer address (puzzle hash hex)
    pub peer_address: String,
    /// Total number of interactions
    pub total_interactions: i64,
    /// Total value exchanged
    pub total_value_exchanged: u64,
    /// First interaction block
    pub first_interaction: i64,
    /// Last interaction block
    pub last_interaction: i64,
    /// Number of different directions
    pub directions_count: i64,
}

/// Address risk assessment
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct AddressRiskAssessment {
    /// Total coins
    pub total_coins: i64,
    /// Coins spent
    pub coins_spent: i64,
    /// Active blocks
    pub active_blocks: i64,
    /// Total received
    pub total_received: u64,
    /// Current balance
    pub current_balance: u64,
    /// First block
    pub first_block: i64,
    /// Last block
    pub last_block: i64,
    /// Number of dust coins
    pub dust_coins: i64,
    /// Number of large coins
    pub large_coins: i64,
    /// Average hold time
    pub avg_hold_time: f64,
    /// Risk category
    pub risk_category: String,
    /// Risk score (0-100)
    pub risk_score: i64,
}

/// Address activity timeline entry
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct AddressActivityTimeline {
    /// Activity date
    pub activity_date: String,
    /// Coins received count
    pub coins_received: i64,
    /// Coins spent count
    pub coins_spent: i64,
    /// Value received
    pub value_received: u64,
    /// Value spent
    pub value_spent: u64,
    /// Net change in value
    pub net_change: i64,
    /// Total activity count
    pub total_activity: i64,
}

/// Detailed activity statistics for an address
#[derive(Debug, Clone, SimpleObject)]
pub struct AddressActivity {
    /// Total coins received
    pub total_coins_received: u64,
    /// Total coins spent
    pub total_coins_spent: u64,
    /// Total value received
    pub total_value_received: u64,
    /// Total value spent
    pub total_value_spent: u64,
    /// Number of unique senders
    pub unique_senders: u32,
    /// Number of unique recipients
    pub unique_recipients: u32,
    /// Average transaction value
    pub avg_transaction_value: f64,
    /// Date of peak activity
    pub peak_activity_date: Option<String>,
}

/// Address sorted by balance
#[derive(Debug, Clone, SimpleObject)]
pub struct AddressByBalance {
    /// Puzzle hash as hex
    pub puzzle_hash_hex: String,
    /// Current balance in mojos
    pub current_balance: u64,
    /// Number of coins
    pub coin_count: u32,
    /// First seen block
    pub first_seen: Option<i64>,
    /// Last seen block
    pub last_seen: Option<i64>,
}

/// Address interaction record
#[derive(Debug, Clone, SimpleObject)]
pub struct AddressInteraction {
    /// From address
    pub from_address: String,
    /// To address
    pub to_address: String,
    /// Number of interactions
    pub interaction_count: u32,
    /// Total value transferred
    pub total_value: u64,
    /// First interaction block
    pub first_interaction: Option<i64>,
    /// Last interaction block
    pub last_interaction: Option<i64>,
}

/// Time-based activity analysis for an address
#[derive(Debug, Clone, SimpleObject)]
pub struct AddressTimeAnalysis {
    /// Most active hour (0-23)
    pub most_active_hour: u32,
    /// Most active day of week (0-6)
    pub most_active_day_of_week: u32,
    /// Average time between transactions (seconds)
    pub avg_time_between_transactions: f64,
    /// Longest inactive period (seconds)
    pub longest_inactive_period: f64,
    /// Weekly activity pattern
    pub weekly_activity_pattern: Vec<ActivityPattern>,
    /// Hourly activity pattern
    pub hourly_activity_pattern: Vec<ActivityPattern>,
}

/// Activity pattern for time analysis
#[derive(Debug, Clone, SimpleObject)]
pub struct ActivityPattern {
    /// Time period (hour or day)
    pub period: u32,
    /// Transaction count in this period
    pub transaction_count: u32,
} 