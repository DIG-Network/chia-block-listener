use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("Database error: {0}")]
    Database(#[from] chia_block_database::error::DatabaseError),

    #[error("DNS discovery error: {0}")]
    DnsDiscovery(#[from] dns_discovery::DnsDiscoveryError),

    #[error("Peer pool error: {0}")]
    PeerPool(String),

    #[error("Block listener error: {0}")]
    BlockListener(String),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("SQL error: {0}")]
    Sql(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Sync timeout")]
    SyncTimeout,

    #[error("Peer connection failed: {host}:{port}")]
    PeerConnectionFailed { host: String, port: u16 },

    #[error("No peers available")]
    NoPeersAvailable,

    #[error("Sync paused")]
    SyncPaused,

    #[error("Exit requested: {reason}")]
    ExitRequested { reason: String },

    #[error("Gap sync failed: {start_height} to {end_height}")]
    GapSyncFailed {
        start_height: u32,
        end_height: u32,
    },

    #[error("Block processing failed at height {height}: {source}")]
    BlockProcessingFailed {
        height: u32,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Metadata fetch failed for URL {url}: {reason}")]
    MetadataFetchFailed { url: String, reason: String },

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, IndexerError>;

impl From<String> for IndexerError {
    fn from(msg: String) -> Self {
        IndexerError::Other(msg)
    }
}

impl From<&str> for IndexerError {
    fn from(msg: &str) -> Self {
        IndexerError::Other(msg.to_string())
    }
} 