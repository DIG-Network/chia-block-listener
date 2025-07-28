use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Certificate information for TLS connections
#[derive(Debug, Clone)]
pub struct ChiaCertificate {
    pub private_key: Vec<u8>,
    pub certificate: Vec<u8>,
    pub certificate_der: Vec<u8>,
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
            software_version: "chia-peer-tls/1.0.0".to_string(),
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