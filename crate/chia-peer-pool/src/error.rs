use thiserror::Error;

#[derive(Error, Debug)]
pub enum PeerPoolError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("TLS error: {0}")]
    Tls(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Handshake error: {0}")]
    Handshake(String),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Block request timeout")]
    RequestTimeout,

    #[error("Connection timeout")]
    ConnectionTimeout,

    #[error("No peers available")]
    NoPeersAvailable,

    #[error("Peer not found: {peer_id}")]
    PeerNotFound { peer_id: String },

    #[error("Peer disconnected: {peer_id}")]
    PeerDisconnected { peer_id: String },

    #[error("Block not found at height {height}")]
    BlockNotFound { height: u64 },

    #[error("Pool shutdown")]
    PoolShutdown,

    #[error("Invalid block data: {reason}")]
    InvalidBlockData { reason: String },

    #[error("Rate limit exceeded for peer {peer_id}")]
    RateLimitExceeded { peer_id: String },

    #[error("Maximum retries exceeded")]
    MaxRetriesExceeded,

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, PeerPoolError>;

impl From<String> for PeerPoolError {
    fn from(msg: String) -> Self {
        PeerPoolError::Other(msg)
    }
}

impl From<&str> for PeerPoolError {
    fn from(msg: &str) -> Self {
        PeerPoolError::Other(msg.to_string())
    }
} 