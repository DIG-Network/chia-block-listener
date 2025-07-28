use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockListenerError {
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

    #[error("Certificate error: {0}")]
    Certificate(String),

    #[error("Connection timeout")]
    ConnectionTimeout,

    #[error("No peers connected")]
    NoPeersConnected,

    #[error("Peer not found: {peer_id}")]
    PeerNotFound { peer_id: String },

    #[error("Peer already connected: {peer_id}")]
    PeerAlreadyConnected { peer_id: String },

    #[error("Listener not started")]
    ListenerNotStarted,

    #[error("Listener already running")]
    ListenerAlreadyRunning,

    #[error("Invalid block data: {reason}")]
    InvalidBlockData { reason: String },

    #[error("Event channel closed")]
    EventChannelClosed,

    #[error("Shutdown requested")]
    ShutdownRequested,

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, BlockListenerError>;

impl From<String> for BlockListenerError {
    fn from(msg: String) -> Self {
        BlockListenerError::Other(msg)
    }
}

impl From<&str> for BlockListenerError {
    fn from(msg: &str) -> Self {
        BlockListenerError::Other(msg.to_string())
    }
} 