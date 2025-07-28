use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the blockchain indexer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerConfig {
    pub database: DatabaseConfig,
    pub blockchain: BlockchainConfig,
    pub network: NetworkConfig,
    pub sync: SyncConfig,
    pub watchdog: WatchdogConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub connection_string: String,
    pub max_connections: u32,
    pub connection_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub start_height: u32,
    pub network_id: String,
    pub max_peers_historical: usize,
    pub max_peers_realtime: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub introducers: Vec<String>,
    pub default_port: u16,
    pub connection_timeout_secs: u64,
    pub reconnect_delay_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub gap_scan_interval_secs: u64,
    pub status_update_interval_secs: u64,
    pub batch_size: usize,
    pub max_concurrent_downloads: usize,
    pub enable_balance_refresh: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchdogConfig {
    pub timeout_minutes: u64,
    pub check_interval_secs: u64,
    pub max_stuck_time_secs: u64,
    pub max_no_progress_checks: usize,
}

/// Represents a gap in the blockchain that needs to be filled
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockGap {
    pub start_height: u32,
    pub end_height: u32,
}

impl BlockGap {
    pub fn new(start_height: u32, end_height: u32) -> Self {
        Self {
            start_height,
            end_height,
        }
    }

    pub fn size(&self) -> u32 {
        if self.end_height >= self.start_height {
            self.end_height - self.start_height + 1
        } else {
            0
        }
    }
}

/// Sync statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    pub total_blocks: u64,
    pub downloaded_blocks: u64,
    pub start_time: DateTime<Utc>,
    pub last_block_time: DateTime<Utc>,
    pub blocks_per_second: f64,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Represents a block with all its data for processing
#[derive(Debug, Clone)]
pub struct ProcessingBlock {
    pub height: u32,
    pub header_hash: String,
    pub timestamp: u64,
    pub weight: String,
    pub is_transaction_block: bool,
    pub coin_additions: Vec<CoinRecord>,
    pub coin_removals: Vec<CoinRecord>,
    pub coin_spends: Vec<CoinSpendRecord>,
    pub coin_creations: Vec<CoinRecord>,
}

/// Coin record for processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinRecord {
    pub parent_coin_info: String,
    pub puzzle_hash: String,
    pub amount: String,
}

/// Coin spend record for processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinSpendRecord {
    pub coin: CoinRecord,
    pub puzzle_reveal: String,
    pub solution: String,
    pub created_coins: Vec<CoinRecord>,
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub host: String,
    pub port: u16,
    pub is_connected: bool,
    pub last_activity: Option<DateTime<Utc>>,
    pub connection_attempts: u32,
    pub last_error: Option<String>,
}

/// Sync status for monitoring and control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub sync_type: String,
    pub start_height: u32,
    pub last_synced_height: u32,
    pub current_peak_height: u32,
    pub is_running: bool,
    pub is_historical_sync: bool,
    pub last_activity: DateTime<Utc>,
    pub session_start_time: DateTime<Utc>,
    pub blocks_processed_session: u64,
    pub sync_speed_blocks_per_sec: Option<f64>,
    pub progress_percentage: Option<f64>,
    pub eta_minutes: Option<i32>,
    pub total_blocks_synced: u64,
    pub errors_count: u32,
    pub last_error: Option<String>,
    pub queue_size: u32,
    pub active_downloads: u32,
}

/// System preferences for controlling sync behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPreference {
    pub meta_key: String,
    pub meta_value: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

/// Block processing result
#[derive(Debug, Clone)]
pub struct BlockProcessingResult {
    pub height: u32,
    pub coins_added: usize,
    pub coins_spent: usize,
    pub cats_processed: usize,
    pub nfts_processed: usize,
    pub processing_time_ms: u64,
}

/// Asset processing result for CATs and NFTs
#[derive(Debug, Clone)]
pub enum AssetProcessingResult {
    Cat {
        asset_id: String,
        amount: u64,
        inner_puzzle_hash: String,
    },
    Nft {
        launcher_id: String,
        collection_id: Option<String>,
        metadata: Option<serde_json::Value>,
    },
    None,
}

/// Watchdog state for monitoring sync health
#[derive(Debug, Clone)]
pub struct WatchdogState {
    pub last_synced_height: u32,
    pub last_peak_height: u32,
    pub last_activity_time: DateTime<Utc>,
    pub last_peak_update_time: DateTime<Utc>,
    pub last_progress_time: DateTime<Utc>,
    pub gap_fill_start_time: Option<DateTime<Utc>>,
    pub is_gap_filling: bool,
    pub consecutive_no_progress_checks: u32,
}

/// Event types for the sync worker
#[derive(Debug, Clone)]
pub enum SyncEvent {
    BlockReceived {
        height: u32,
        peer_id: String,
    },
    PeerConnected {
        peer_id: String,
        host: String,
        port: u16,
    },
    PeerDisconnected {
        peer_id: String,
        reason: String,
    },
    GapDetected {
        gaps: Vec<BlockGap>,
    },
    GapFilled {
        gap: BlockGap,
        duration_secs: f64,
    },
    SyncPaused,
    SyncResumed,
    ErrorOccurred {
        error: String,
        context: String,
    },
}

/// Metadata for NFTs and CATs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataResult {
    pub url: String,
    pub data: serde_json::Value,
    pub content_type: Option<String>,
    pub retrieved_at: DateTime<Utc>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub blocks_per_second: f64,
    pub coins_per_second: f64,
    pub spends_per_second: f64,
    pub database_write_time_ms: u64,
    pub block_processing_time_ms: u64,
    pub memory_usage_mb: u64,
    pub active_connections: usize,
} 