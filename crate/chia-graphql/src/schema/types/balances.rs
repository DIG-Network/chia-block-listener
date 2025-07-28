use async_graphql::*;
use serde::{Deserialize, Serialize};

/// Balance distribution statistics
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct BalanceDistributionStats {
    /// Total number of addresses
    pub total_addresses: i64,
    /// Total supply in mojos
    pub total_supply: u64,
    /// Average balance
    pub avg_balance: f64,
    /// Minimum balance
    pub min_balance: u64,
    /// Maximum balance
    pub max_balance: u64,
    /// Addresses with 1 XCH or more
    pub addresses_with_1_xch_plus: i64,
    /// Addresses with 1000 XCH or more
    pub addresses_with_1000_xch_plus: i64,
    /// Addresses with 1 million XCH or more
    pub addresses_with_1m_xch_plus: i64,
}

/// Richest address information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct RichestAddress {
    /// Puzzle hash in hex format
    pub puzzle_hash_hex: String,
    /// Total balance in mojos
    pub total_mojos: u64,
    /// Total balance in XCH
    pub total_xch: f64,
    /// Number of coins
    pub coin_count: i64,
    /// Minimum coin amount
    pub min_coin_amount: u64,
    /// Maximum coin amount
    pub max_coin_amount: u64,
    /// Average coin amount
    pub avg_coin_amount: u64,
    /// Rank position
    pub rank_position: i64,
}

/// Balance change information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct BalanceChange {
    /// Block height
    pub block_height: i64,
    /// Net change in balance
    pub net_change: i64,
    /// Number of coins added
    pub coins_added: i64,
    /// Number of coins spent
    pub coins_spent: i64,
}

/// Address within a balance range
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct AddressInRange {
    /// Puzzle hash in hex format
    pub puzzle_hash_hex: String,
    /// Total balance in mojos
    pub total_mojos: u64,
    /// Total balance in XCH
    pub total_xch: f64,
    /// Number of coins
    pub coin_count: i64,
}

/// Coin size distribution
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct CoinSizeDistribution {
    /// Size category description
    pub size_category: String,
    /// Number of coins in this category
    pub coin_count: i64,
    /// Total amount in this category
    pub total_amount: u64,
    /// Minimum amount in category
    pub min_amount: u64,
    /// Maximum amount in category
    pub max_amount: u64,
    /// Average amount in category
    pub avg_amount: u64,
}

/// Balance percentiles
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct BalancePercentiles {
    /// Median balance (50th percentile)
    pub median: u64,
    /// 75th percentile
    pub percentile_75: u64,
    /// 90th percentile
    pub percentile_90: u64,
    /// 95th percentile
    pub percentile_95: u64,
    /// 99th percentile
    pub percentile_99: u64,
} 