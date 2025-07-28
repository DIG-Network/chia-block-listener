use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Event types that the block listener can emit
pub const EVENT_BLOCK_RECEIVED: &str = "blockReceived";
pub const EVENT_PEER_CONNECTED: &str = "peerConnected";
pub const EVENT_PEER_DISCONNECTED: &str = "peerDisconnected";
pub const EVENT_NEW_PEAK: &str = "newPeak";

/// Configuration for the block listener
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockListenerConfig {
    pub network_id: String,
    pub max_peers: usize,
    pub connection_timeout_secs: u64,
    pub reconnect_delay_secs: u64,
    pub enable_auto_reconnect: bool,
    pub peak_monitoring_enabled: bool,
    pub block_parsing_enabled: bool,
}

impl Default for BlockListenerConfig {
    fn default() -> Self {
        Self {
            network_id: "mainnet".to_string(),
            max_peers: 5,
            connection_timeout_secs: 30,
            reconnect_delay_secs: 5,
            enable_auto_reconnect: true,
            peak_monitoring_enabled: true,
            block_parsing_enabled: true,
        }
    }
}

/// Event emitted when a peer connects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnectedEvent {
    pub peer_id: String,
    pub host: String,
    pub port: u16,
    pub connected_at: DateTime<Utc>,
    pub network_id: String,
}

/// Event emitted when a peer disconnects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerDisconnectedEvent {
    pub peer_id: String,
    pub host: String,
    pub port: u16,
    pub reason: String,
    pub disconnected_at: DateTime<Utc>,
    pub was_expected: bool,
}

/// Event emitted when a new peak height is detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPeakEvent {
    pub peer_id: String,
    pub old_peak: Option<u32>,
    pub new_peak: u32,
    pub discovered_at: DateTime<Utc>,
    pub header_hash: String,
    pub weight: String,
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
    Disconnected,
    Connecting,
    Connected,
    Failed,
    Reconnecting,
}

impl std::fmt::Display for PeerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerState::Disconnected => write!(f, "disconnected"),
            PeerState::Connecting => write!(f, "connecting"),
            PeerState::Connected => write!(f, "connected"),
            PeerState::Failed => write!(f, "failed"),
            PeerState::Reconnecting => write!(f, "reconnecting"),
        }
    }
}

/// Information about a connected peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub host: String,
    pub port: u16,
    pub state: PeerState,
    pub connected_at: Option<DateTime<Utc>>,
    pub last_activity: Option<DateTime<Utc>>,
    pub last_peak_height: Option<u32>,
    pub connection_attempts: u32,
    pub last_error: Option<String>,
    pub blocks_received: u64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
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
            last_peak_height: None,
            connection_attempts: 0,
            last_error: None,
            blocks_received: 0,
            bytes_received: 0,
            bytes_sent: 0,
        }
    }

    /// Check if this peer is healthy (connected and active)
    pub fn is_healthy(&self) -> bool {
        match self.state {
            PeerState::Connected => {
                self.last_activity
                    .map(|last| (Utc::now() - last).num_minutes() < 10)
                    .unwrap_or(false)
            }
            _ => false,
        }
    }

    /// Get connection duration in seconds
    pub fn connection_duration_secs(&self) -> Option<i64> {
        self.connected_at.map(|connected| (Utc::now() - connected).num_seconds())
    }
}

/// Statistics for the block listener
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerStats {
    pub total_peers: usize,
    pub connected_peers: usize,
    pub failed_peers: usize,
    pub total_blocks_received: u64,
    pub total_peaks_discovered: u64,
    pub uptime_seconds: u64,
    pub total_bytes_received: u64,
    pub total_bytes_sent: u64,
    pub current_peak_height: Option<u32>,
    pub last_block_received: Option<DateTime<Utc>>,
    pub last_peak_update: Option<DateTime<Utc>>,
}

/// Event handler trait for receiving block listener events
pub trait EventHandler: Send + Sync {
    fn on_block_received(&self, event: BlockReceivedEvent);
    fn on_peer_connected(&self, event: PeerConnectedEvent);
    fn on_peer_disconnected(&self, event: PeerDisconnectedEvent);
    fn on_new_peak(&self, event: NewPeakEvent);
}

/// Event emitted internally for processing
#[derive(Debug, Clone)]
pub enum InternalEvent {
    PeerConnected(PeerConnectedEvent),
    PeerDisconnected(PeerDisconnectedEvent),
    BlockReceived(BlockReceivedEvent),
    NewPeak(NewPeakEvent),
    Error { peer_id: String, error: String },
}

/// Certificate information for TLS connections
#[derive(Debug, Clone)]
pub struct ChiaCertificate {
    pub private_key: Vec<u8>,
    pub certificate: Vec<u8>,
    pub certificate_der: Vec<u8>,
}

/// Connection info for a peer
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub peer_id: String,
    pub host: String,
    pub port: u16,
    pub network_id: String,
    pub use_tls: bool,
}

impl ConnectionInfo {
    pub fn new(host: String, port: u16, network_id: String) -> Self {
        let peer_id = format!("{}:{}", host, port);
        Self {
            peer_id,
            host,
            port,
            network_id,
            use_tls: true,
        }
    }

    pub fn websocket_url(&self) -> String {
        let protocol = if self.use_tls { "wss" } else { "ws" };
        format!("{}://{}:{}/ws", protocol, self.host, self.port)
    }
}

/// Message types for internal communication
#[derive(Debug)]
pub enum ListenerMessage {
    AddPeer(ConnectionInfo),
    RemovePeer(String),
    GetPeerInfo(tokio::sync::oneshot::Sender<Vec<PeerInfo>>),
    GetStats(tokio::sync::oneshot::Sender<ListenerStats>),
    Shutdown,
}

/// Raw message received from a peer
#[derive(Debug, Clone)]
pub struct RawMessage {
    pub peer_id: String,
    pub message_type: u8,
    pub payload: Vec<u8>,
    pub received_at: DateTime<Utc>,
}

/// Handshake information for Chia protocol
#[derive(Debug, Clone)]
pub struct HandshakeInfo {
    pub network_id: String,
    pub protocol_version: String,
    pub software_version: String,
    pub server_port: u16,
    pub node_type: u8,
    pub capabilities: Vec<(u16, String)>,
}

impl Default for HandshakeInfo {
    fn default() -> Self {
        Self {
            network_id: "mainnet".to_string(),
            protocol_version: "0.0.36".to_string(),
            software_version: "chia-block-listener/1.0.0".to_string(),
            server_port: 8444,
            node_type: 1, // FullNode
            capabilities: vec![
                (1, "base".to_string()),
                (2, "block_headers".to_string()),
                (3, "rate_limits_v2".to_string()),
            ],
        }
    }
} 