use async_graphql::*;
use serde::{Deserialize, Serialize};

/// Basic coin information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct CoinInfo {
    /// Coin ID hex
    pub coin_id: String,
    /// Parent coin ID hex
    pub parent_coin_id: String,
    /// Puzzle hash hex
    pub puzzle_hash: String,
    /// Coin amount in mojos
    pub amount: u64,
    /// Block height where created
    pub created_block: i64,
    /// Block height where spent (if any)
    pub spent_block: Option<i64>,
}

/// Basic block information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct BlockInfo {
    /// Block height
    pub height: i64,
    /// Block hash hex
    pub block_hash: String,
    /// Previous block hash hex
    pub prev_hash: String,
    /// Block timestamp
    pub timestamp: String,
    /// Number of coins in block
    pub coin_count: i64,
    /// Number of spends in block
    pub spend_count: i64,
}

/// Basic balance information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct BalanceInfo {
    /// Puzzle hash hex
    pub puzzle_hash: String,
    /// Confirmed balance in mojos
    pub confirmed_balance: u64,
    /// Unconfirmed balance in mojos
    pub unconfirmed_balance: u64,
    /// Number of unspent coins
    pub unspent_coin_count: i64,
    /// First activity block
    pub first_activity_block: Option<i64>,
    /// Last activity block
    pub last_activity_block: Option<i64>,
}

/// Parent coin information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ParentCoin {
    /// Parent coin ID
    pub parent_coin_id: String,
    /// Parent puzzle hash in hex
    pub parent_puzzle_hash_hex: String,
    /// Parent amount
    pub parent_amount: u64,
    /// Block where parent was created
    pub parent_created_block: i64,
    /// Parent's parent coin info in hex
    pub parent_parent_coin_info_hex: String,
}

/// Child coin information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ChildCoin {
    /// Child coin ID
    pub child_coin_id: String,
    /// Child puzzle hash in hex
    pub child_puzzle_hash_hex: String,
    /// Child amount
    pub child_amount: u64,
    /// Block where child was created
    pub child_created_block: i64,
}

/// Pagination input for queries
#[derive(Debug, Clone, InputObject)]
pub struct PaginationInput {
    /// Page number (1-based)
    #[graphql(default = 1)]
    pub page: i32,
    /// Number of items per page
    #[graphql(default = 20)]
    pub page_size: i32,
} 