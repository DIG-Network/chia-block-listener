use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Event emitted when a peer connects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnectedEvent {
    pub peer_id: String,
    pub host: String,
    pub port: u16,
    pub connected_at: DateTime<Utc>,
}

/// Event emitted when a peer disconnects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerDisconnectedEvent {
    pub peer_id: String,
    pub host: String,
    pub port: u16,
    pub reason: String,
    pub disconnected_at: DateTime<Utc>,
}

/// Event emitted when a new peak height is discovered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPeakHeightEvent {
    pub old_peak: Option<u32>,
    pub new_peak: u32,
    pub peer_id: String,
    pub discovered_at: DateTime<Utc>,
}

/// Event emitted when a block is received
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockReceivedEvent {
    pub height: u32,
    pub header_hash: String,
    pub timestamp: u64,
    pub weight: String,
    pub is_transaction_block: bool,
    pub peer_id: String,
    pub received_at: DateTime<Utc>,
    pub coin_additions: Vec<CoinRecord>,
    pub coin_removals: Vec<CoinRecord>,
    pub coin_spends: Vec<CoinSpendRecord>,
    pub coin_creations: Vec<CoinRecord>,
}

/// Simplified coin record for events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinRecord {
    pub parent_coin_info: String,
    pub puzzle_hash: String,
    pub amount: String,
}

/// Simplified coin spend record for events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinSpendRecord {
    pub coin: CoinRecord,
    pub puzzle_reveal: String,
    pub solution: String,
}

/// Peer connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerState {
    Connecting,
    Connected,
    Disconnected,
    Failed,
}

impl fmt::Display for PeerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PeerState::Connecting => write!(f, "connecting"),
            PeerState::Connected => write!(f, "connected"),
            PeerState::Disconnected => write!(f, "disconnected"),
            PeerState::Failed => write!(f, "failed"),
        }
    }
}

/// Peer information and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub host: String,
    pub port: u16,
    pub state: PeerState,
    pub connected_at: Option<DateTime<Utc>>,
    pub last_activity: Option<DateTime<Utc>>,
    pub peak_height: Option<u32>,
    pub connection_attempts: u32,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub average_response_time_ms: Option<f64>,
    pub last_error: Option<String>,
}

impl PeerInfo {
    pub fn new(peer_id: String, host: String, port: u16) -> Self {
        Self {
            peer_id,
            host,
            port,
            state: PeerState::Disconnected,
            connected_at: None,
            last_activity: None,
            peak_height: None,
            connection_attempts: 0,
            successful_requests: 0,
            failed_requests: 0,
            bytes_received: 0,
            bytes_sent: 0,
            average_response_time_ms: None,
            last_error: None,
        }
    }

    /// Calculate success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        let total = self.successful_requests + self.failed_requests;
        if total == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / total as f64) * 100.0
        }
    }

    /// Check if peer is healthy based on recent activity and success rate
    pub fn is_healthy(&self) -> bool {
        match self.state {
            PeerState::Connected => {
                let success_rate = self.success_rate();
                let has_recent_activity = self.last_activity
                    .map(|last| (Utc::now() - last).num_minutes() < 5)
                    .unwrap_or(false);
                
                success_rate >= 80.0 && has_recent_activity
            }
            _ => false,
        }
    }
}

/// Pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerPoolConfig {
    pub network_id: String,
    pub max_peers: usize,
    pub connection_timeout_secs: u64,
    pub request_timeout_secs: u64,
    pub rate_limit_ms: u64,
    pub max_retries: u32,
    pub health_check_interval_secs: u64,
    pub enable_peak_monitoring: bool,
}

impl Default for PeerPoolConfig {
    fn default() -> Self {
        Self {
            network_id: "mainnet".to_string(),
            max_peers: 10,
            connection_timeout_secs: 30,
            request_timeout_secs: 10,
            rate_limit_ms: 500,
            max_retries: 3,
            health_check_interval_secs: 60,
            enable_peak_monitoring: true,
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub total_peers: usize,
    pub connected_peers: usize,
    pub failed_peers: usize,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub current_peak_height: Option<u32>,
    pub uptime_seconds: u64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
}

impl PoolStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }
}

/// Block request with retry information
#[derive(Debug, Clone)]
pub struct BlockRequest {
    pub height: u64,
    pub attempt: u32,
    pub requested_at: DateTime<Utc>,
    pub preferred_peer: Option<String>,
}

impl BlockRequest {
    pub fn new(height: u64) -> Self {
        Self {
            height,
            attempt: 0,
            requested_at: Utc::now(),
            preferred_peer: None,
        }
    }

    pub fn retry(&mut self) {
        self.attempt += 1;
        self.requested_at = Utc::now();
    }
} 