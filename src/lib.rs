#![deny(clippy::all)]

use napi_derive::napi;

mod block_parser_napi;
mod database_napi;
mod dns_discovery_napi;
mod error;
mod event_emitter;
mod peer_pool_napi;

// Re-export all the NAPI bindings that use the new crates
pub use block_parser_napi::{ChiaBlockParser, CoinInfoWrapper, CoinSpendInfoWrapper, ParsedBlockWrapper, GeneratorBlockInfoWrapper, BlockHeightInfoWrapper};
pub use database_napi::{ChiaBlockDatabase, DatabaseType, Block, Spend, BlockchainNamespace, AnalyticsNamespace, AssetsNamespace, SystemNamespace};
pub use dns_discovery_napi::{DnsDiscoveryClient, PeerAddressWrapper, DiscoveryResultWrapper, AddressResult, DnsDiscoveryErrorInfo};
pub use event_emitter::{ChiaBlockListener, EventTypes, get_event_types, BlockReceivedEvent, PeerConnectedEvent, PeerDisconnectedEvent, NewPeakEvent, CoinRecord, CoinSpendRecord, PeerInfo, ListenerStatistics};
pub use peer_pool_napi::{ChiaPeerPool, BlockReceivedEvent as PoolBlockEvent, PeerConnectedEvent as PoolPeerConnectedEvent, PeerDisconnectedEvent as PoolPeerDisconnectedEvent, NewPeakHeightEvent, PeerInfo as PoolPeerInfo, PeerPoolStats};

#[napi]
pub fn init_tracing() {
    // Initialize logging with a filter for our crate
    use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        .add_directive("chia_block_listener=debug".parse().unwrap())
        .add_directive("chia_peer_pool=debug".parse().unwrap())
        .add_directive("chia_block_database=debug".parse().unwrap())
        .add_directive("chia_full_indexer=debug".parse().unwrap())
        .add_directive("dns_discovery=debug".parse().unwrap());

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();
}

/// Get version information for all components
#[napi]
pub fn get_version_info() -> VersionInfo {
    VersionInfo {
        main_version: env!("CARGO_PKG_VERSION").to_string(),
        peer_pool_version: "0.1.0".to_string(), // chia-peer-pool version
        block_listener_version: "0.1.0".to_string(), // chia-block-listener version
        block_database_version: "0.1.0".to_string(), // chia-block-database version
        full_indexer_version: "0.1.0".to_string(), // chia-full-indexer version
        dns_discovery_version: "0.1.0".to_string(), // dns-discovery version
        build_timestamp: env!("BUILD_TIMESTAMP").unwrap_or("unknown").to_string(),
        commit_hash: env!("GIT_HASH").unwrap_or("unknown").to_string(),
    }
}

#[napi(object)]
pub struct VersionInfo {
    pub main_version: String,
    pub peer_pool_version: String,
    pub block_listener_version: String,
    pub block_database_version: String,
    pub full_indexer_version: String,
    pub dns_discovery_version: String,
    pub build_timestamp: String,
    pub commit_hash: String,
}

/// Health check function for monitoring
#[napi]
pub fn health_check() -> HealthStatus {
    HealthStatus {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        components: vec![
            ComponentStatus {
                name: "peer-pool".to_string(),
                status: "available".to_string(),
                version: "0.1.0".to_string(),
            },
            ComponentStatus {
                name: "block-listener".to_string(), 
                status: "available".to_string(),
                version: "0.1.0".to_string(),
            },
            ComponentStatus {
                name: "block-database".to_string(),
                status: "available".to_string(),
                version: "0.1.0".to_string(),
            },
            ComponentStatus {
                name: "dns-discovery".to_string(),
                status: "available".to_string(),
                version: "0.1.0".to_string(),
            },
        ],
    }
}

#[napi(object)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: String,
    pub components: Vec<ComponentStatus>,
}

#[napi(object)]
pub struct ComponentStatus {
    pub name: String,
    pub status: String,
    pub version: String,
}

/// Utility function to create a full indexer configuration
#[napi]
pub fn create_indexer_config(
    database_url: String,
    network_id: String,
    max_peers: u32,
    enable_auto_sync: bool
) -> IndexerConfig {
    IndexerConfig {
        database_url,
        network_id,
        max_peers,
        enable_auto_sync,
        peer_pool_config: PeerPoolConfigInfo {
            connection_timeout_secs: 30,
            request_timeout_secs: 10,
            rate_limit_ms: 500,
            max_retries: 3,
        },
        block_listener_config: BlockListenerConfigInfo {
            connection_timeout_secs: 30,
            reconnect_delay_secs: 5,
            enable_auto_reconnect: true,
            peak_monitoring_enabled: true,
        },
    }
}

#[napi(object)]
pub struct IndexerConfig {
    pub database_url: String,
    pub network_id: String,
    pub max_peers: u32,
    pub enable_auto_sync: bool,
    pub peer_pool_config: PeerPoolConfigInfo,
    pub block_listener_config: BlockListenerConfigInfo,
}

#[napi(object)]
pub struct PeerPoolConfigInfo {
    pub connection_timeout_secs: u32,
    pub request_timeout_secs: u32,
    pub rate_limit_ms: u32,
    pub max_retries: u32,
}

#[napi(object)]
pub struct BlockListenerConfigInfo {
    pub connection_timeout_secs: u32,
    pub reconnect_delay_secs: u32,
    pub enable_auto_reconnect: bool,
    pub peak_monitoring_enabled: bool,
}
