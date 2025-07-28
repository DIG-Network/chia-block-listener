use chia_block_listener::{
    ChiaBlockListener as ListenerImplementation,
    BlockListenerConfig,
    BlockReceivedEvent as ListenerBlockEvent,
    PeerConnectedEvent as ListenerPeerConnectedEvent,
    PeerDisconnectedEvent as ListenerPeerDisconnectedEvent,
    NewPeakEvent as ListenerNewPeakEvent,
    PeerInfo as ListenerPeerInfo,
    ListenerStats,
    EventHandler,
    LoggingEventHandler,
};
use napi::bindgen_prelude::*;
use napi::{
    threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
    JsFunction,
};
use napi_derive::napi;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

// Event type constants for JavaScript
pub const EVENT_BLOCK_RECEIVED: &str = "blockReceived";
pub const EVENT_PEER_CONNECTED: &str = "peerConnected";
pub const EVENT_PEER_DISCONNECTED: &str = "peerDisconnected";
pub const EVENT_NEW_PEAK: &str = "newPeak";

// Export event types for TypeScript
#[napi(object)]
pub struct EventTypes {
    pub block_received: String,
    pub peer_connected: String,
    pub peer_disconnected: String,
    pub new_peak: String,
}

#[napi]
pub fn get_event_types() -> EventTypes {
    EventTypes {
        block_received: EVENT_BLOCK_RECEIVED.to_string(),
        peer_connected: EVENT_PEER_CONNECTED.to_string(),
        peer_disconnected: EVENT_PEER_DISCONNECTED.to_string(),
        new_peak: EVENT_NEW_PEAK.to_string(),
    }
}

// NAPI wrapper for the block listener
#[napi]
pub struct ChiaBlockListener {
    listener: Arc<ListenerImplementation>,
    listeners: Arc<RwLock<EventListeners>>,
}

struct EventListeners {
    block_listeners: Vec<ThreadsafeFunction<ListenerBlockEvent, ErrorStrategy::Fatal>>,
    peer_connected_listeners: Vec<ThreadsafeFunction<ListenerPeerConnectedEvent, ErrorStrategy::Fatal>>,
    peer_disconnected_listeners: Vec<ThreadsafeFunction<ListenerPeerDisconnectedEvent, ErrorStrategy::Fatal>>,
    new_peak_listeners: Vec<ThreadsafeFunction<ListenerNewPeakEvent, ErrorStrategy::Fatal>>,
}

// NAPI-compatible event types for TypeScript
#[napi(object)]
#[derive(Clone)]
pub struct BlockReceivedEvent {
    pub height: u32,
    pub header_hash: String,
    pub timestamp: String,
    pub weight: String,
    pub is_transaction_block: bool,
    pub peer_id: String,
    pub received_at: String,
    pub coin_additions: Vec<CoinRecord>,
    pub coin_removals: Vec<CoinRecord>,
    pub coin_spends: Vec<CoinSpendRecord>,
    pub coin_creations: Vec<CoinRecord>,
}

#[napi(object)]
#[derive(Clone)]
pub struct CoinRecord {
    pub parent_coin_info: String,
    pub puzzle_hash: String,
    pub amount: String,
}

#[napi(object)]
#[derive(Clone)]
pub struct CoinSpendRecord {
    pub coin: CoinRecord,
    pub puzzle_reveal: String,
    pub solution: String,
}

#[napi(object)]
#[derive(Clone)]
pub struct PeerConnectedEvent {
    pub peer_id: String,
    pub host: String,
    pub port: u32,
    pub connected_at: String,
}

#[napi(object)]
#[derive(Clone)]
pub struct PeerDisconnectedEvent {
    pub peer_id: String,
    pub host: String,
    pub port: u32,
    pub reason: String,
    pub disconnected_at: String,
}

#[napi(object)]
#[derive(Clone)]
pub struct NewPeakEvent {
    pub peer_id: String,
    pub old_peak: Option<u32>,
    pub new_peak: u32,
    pub discovered_at: String,
}

#[napi(object)]
#[derive(Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub host: String,
    pub port: u32,
    pub state: String,
    pub connected_at: Option<String>,
    pub last_activity: Option<String>,
    pub last_peak_height: Option<u32>,
    pub connection_attempts: u32,
    pub last_error: Option<String>,
    pub blocks_received: String, // u64 as string
    pub bytes_received: String,  // u64 as string
    pub bytes_sent: String,      // u64 as string
}

#[napi(object)]
#[derive(Clone)]
pub struct ListenerStatistics {
    pub total_peers: u32,
    pub connected_peers: u32,
    pub failed_peers: u32,
    pub total_blocks_received: String,  // u64 as string
    pub total_peaks_discovered: String, // u64 as string
    pub uptime_seconds: String,         // u64 as string
    pub total_bytes_received: String,   // u64 as string
    pub total_bytes_sent: String,       // u64 as string
    pub current_peak_height: Option<u32>,
    pub last_block_received: Option<String>,
    pub last_peak_update: Option<String>,
}

#[napi]
impl ChiaBlockListener {
    /// Convert internal BlockReceivedEvent to NAPI-compatible version
    fn convert_block_event(event: ListenerBlockEvent) -> BlockReceivedEvent {
        BlockReceivedEvent {
            height: event.height,
            header_hash: event.header_hash,
            timestamp: event.timestamp.to_string(),
            weight: event.weight,
            is_transaction_block: event.is_transaction_block,
            peer_id: event.peer_id,
            received_at: event.received_at.to_rfc3339(),
            coin_additions: event.coin_additions.into_iter().map(|coin| CoinRecord {
                parent_coin_info: coin.parent_coin_info,
                puzzle_hash: coin.puzzle_hash,
                amount: coin.amount,
            }).collect(),
            coin_removals: event.coin_removals.into_iter().map(|coin| CoinRecord {
                parent_coin_info: coin.parent_coin_info,
                puzzle_hash: coin.puzzle_hash,
                amount: coin.amount,
            }).collect(),
            coin_spends: event.coin_spends.into_iter().map(|spend| CoinSpendRecord {
                coin: CoinRecord {
                    parent_coin_info: spend.coin.parent_coin_info,
                    puzzle_hash: spend.coin.puzzle_hash,
                    amount: spend.coin.amount,
                },
                puzzle_reveal: spend.puzzle_reveal,
                solution: spend.solution,
            }).collect(),
            coin_creations: event.coin_creations.into_iter().map(|coin| CoinRecord {
                parent_coin_info: coin.parent_coin_info,
                puzzle_hash: coin.puzzle_hash,
                amount: coin.amount,
            }).collect(),
        }
    }

    /// Convert internal PeerInfo to NAPI-compatible version
    fn convert_peer_info(info: ListenerPeerInfo) -> PeerInfo {
        PeerInfo {
            peer_id: info.peer_id,
            host: info.host,
            port: info.port as u32,
            state: info.state.to_string(),
            connected_at: info.connected_at.map(|dt| dt.to_rfc3339()),
            last_activity: info.last_activity.map(|dt| dt.to_rfc3339()),
            last_peak_height: info.last_peak_height,
            connection_attempts: info.connection_attempts,
            last_error: info.last_error,
            blocks_received: info.blocks_received.to_string(),
            bytes_received: info.bytes_received.to_string(),
            bytes_sent: info.bytes_sent.to_string(),
        }
    }

    /// Convert internal ListenerStats to NAPI-compatible version
    fn convert_listener_stats(stats: ListenerStats) -> ListenerStatistics {
        ListenerStatistics {
            total_peers: stats.total_peers as u32,
            connected_peers: stats.connected_peers as u32,
            failed_peers: stats.failed_peers as u32,
            total_blocks_received: stats.total_blocks_received.to_string(),
            total_peaks_discovered: stats.total_peaks_discovered.to_string(),
            uptime_seconds: stats.uptime_seconds.to_string(),
            total_bytes_received: stats.total_bytes_received.to_string(),
            total_bytes_sent: stats.total_bytes_sent.to_string(),
            current_peak_height: stats.current_peak_height,
            last_block_received: stats.last_block_received.map(|dt| dt.to_rfc3339()),
            last_peak_update: stats.last_peak_update.map(|dt| dt.to_rfc3339()),
        }
    }

    #[napi(constructor)]
    pub fn new() -> Self {
        info!("🎧 Creating new ChiaBlockListener with chia-block-listener crate");
        
        let listeners = Arc::new(RwLock::new(EventListeners {
            block_listeners: Vec::new(),
            peer_connected_listeners: Vec::new(),
            peer_disconnected_listeners: Vec::new(),
            new_peak_listeners: Vec::new(),
        }));

        // Create listener with default configuration
        let config = BlockListenerConfig::default();
        let listener = Arc::new(ListenerImplementation::new(config));

        info!("✅ ChiaBlockListener created successfully");

        Self { listener, listeners }
    }

    #[napi(factory)]
    pub fn new_with_config(
        network_id: String, 
        max_peers: u32,
        enable_auto_reconnect: bool
    ) -> Self {
        info!("🎧 Creating ChiaBlockListener with custom config: network={}, max_peers={}, auto_reconnect={}", 
              network_id, max_peers, enable_auto_reconnect);
        
        let listeners = Arc::new(RwLock::new(EventListeners {
            block_listeners: Vec::new(),
            peer_connected_listeners: Vec::new(),
            peer_disconnected_listeners: Vec::new(),
            new_peak_listeners: Vec::new(),
        }));

        let config = BlockListenerConfig {
            network_id,
            max_peers: max_peers as usize,
            enable_auto_reconnect,
            ..BlockListenerConfig::default()
        };

        let listener = Arc::new(ListenerImplementation::new(config));

        Self { listener, listeners }
    }

    #[napi(js_name = "start")]
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting block listener");
        
        self.listener
            .start()
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to start listener: {}", e)))
    }

    #[napi(js_name = "stop")]
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping block listener");
        
        self.listener
            .stop()
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to stop listener: {}", e)))
    }

    #[napi(js_name = "addPeer")]
    pub async fn add_peer(&self, host: String, port: u16) -> Result<String> {
        info!("➕ Adding peer: {}:{}", host, port);
        
        self.listener
            .add_peer(host, port)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to add peer: {}", e)))
    }

    #[napi(js_name = "removePeer")]
    pub async fn remove_peer(&self, peer_id: String) -> Result<bool> {
        info!("➖ Removing peer: {}", peer_id);
        
        self.listener
            .remove_peer(&peer_id)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to remove peer: {}", e)))
    }

    #[napi(js_name = "getPeerInfo")]
    pub async fn get_peer_info(&self) -> Result<Vec<PeerInfo>> {
        let peer_infos = self.listener
            .get_peer_info()
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get peer info: {}", e)))?;
        
        Ok(peer_infos.into_iter().map(Self::convert_peer_info).collect())
    }

    #[napi(js_name = "getStats")]
    pub async fn get_stats(&self) -> Result<ListenerStatistics> {
        let stats = self.listener
            .get_stats()
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get stats: {}", e)))?;
        
        Ok(Self::convert_listener_stats(stats))
    }

    #[napi(js_name = "getConnectedPeers")]
    pub async fn get_connected_peers(&self) -> Vec<String> {
        self.listener.get_connected_peers().await
    }

    #[napi(js_name = "getPeakHeight")]
    pub async fn get_peak_height(&self) -> Option<u32> {
        self.listener.get_peak_height().await
    }

    #[napi(js_name = "isRunning")]
    pub async fn is_running(&self) -> bool {
        self.listener.is_running().await
    }

    #[napi(js_name = "onBlockReceived")]
    pub async fn on_block_received(&self, callback: JsFunction) -> Result<()> {
        let tsfn: ThreadsafeFunction<ListenerBlockEvent, ErrorStrategy::Fatal> = callback
            .create_threadsafe_function(0, |ctx| {
                let event = Self::convert_block_event(ctx.value.clone());
                ctx.env.to_js_value(&event)
            })?;

        self.listeners.write().await.block_listeners.push(tsfn);
        info!("📦 Block received event listener added");
        Ok(())
    }

    #[napi(js_name = "onPeerConnected")]
    pub async fn on_peer_connected(&self, callback: JsFunction) -> Result<()> {
        let tsfn: ThreadsafeFunction<ListenerPeerConnectedEvent, ErrorStrategy::Fatal> = callback
            .create_threadsafe_function(0, |ctx| {
                ctx.env.create_object_with_properties(&[
                    ("peerId", ctx.env.create_string(&ctx.value.peer_id)?),
                    ("host", ctx.env.create_string(&ctx.value.host)?),
                    ("port", ctx.env.create_uint32(ctx.value.port as u32)?),
                    ("connectedAt", ctx.env.create_string(&ctx.value.connected_at.to_rfc3339())?),
                ])
            })?;

        self.listeners.write().await.peer_connected_listeners.push(tsfn);
        info!("🔗 Peer connected event listener added");
        Ok(())
    }

    #[napi(js_name = "onPeerDisconnected")]
    pub async fn on_peer_disconnected(&self, callback: JsFunction) -> Result<()> {
        let tsfn: ThreadsafeFunction<ListenerPeerDisconnectedEvent, ErrorStrategy::Fatal> = callback
            .create_threadsafe_function(0, |ctx| {
                ctx.env.create_object_with_properties(&[
                    ("peerId", ctx.env.create_string(&ctx.value.peer_id)?),
                    ("host", ctx.env.create_string(&ctx.value.host)?),
                    ("port", ctx.env.create_uint32(ctx.value.port as u32)?),
                    ("reason", ctx.env.create_string(&ctx.value.reason)?),
                    ("disconnectedAt", ctx.env.create_string(&ctx.value.disconnected_at.to_rfc3339())?),
                ])
            })?;

        self.listeners.write().await.peer_disconnected_listeners.push(tsfn);
        info!("💔 Peer disconnected event listener added");
        Ok(())
    }

    #[napi(js_name = "onNewPeak")]
    pub async fn on_new_peak(&self, callback: JsFunction) -> Result<()> {
        let tsfn: ThreadsafeFunction<ListenerNewPeakEvent, ErrorStrategy::Fatal> = callback
            .create_threadsafe_function(0, |ctx| {
                let mut properties = vec![
                    ("newPeak", ctx.env.create_uint32(ctx.value.new_peak)?),
                    ("peerId", ctx.env.create_string(&ctx.value.peer_id)?),
                    ("discoveredAt", ctx.env.create_string(&ctx.value.discovered_at.to_rfc3339())?),
                ];

                if let Some(old_peak) = ctx.value.old_peak {
                    properties.push(("oldPeak", ctx.env.create_uint32(old_peak)?));
                }

                ctx.env.create_object_with_properties(&properties)
            })?;

        self.listeners.write().await.new_peak_listeners.push(tsfn);
        info!("🎯 New peak event listener added");
        Ok(())
    }

    #[napi(js_name = "clearEventListeners")]
    pub async fn clear_event_listeners(&self) {
        let mut listeners = self.listeners.write().await;
        listeners.block_listeners.clear();
        listeners.peer_connected_listeners.clear();
        listeners.peer_disconnected_listeners.clear();
        listeners.new_peak_listeners.clear();
        info!("🧹 Cleared all event listeners");
    }

    #[napi(js_name = "enableLogging")]
    pub async fn enable_logging(&self) {
        let logging_handler = Arc::new(LoggingEventHandler);
        self.listener.add_event_handler(logging_handler).await;
        info!("📝 Logging event handler enabled");
    }
}
