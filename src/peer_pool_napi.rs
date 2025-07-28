use chia_peer_pool::{
    ChiaPeerPool as PoolImplementation, 
    PeerPoolConfig, 
    BlockReceivedEvent as PoolBlockEvent,
    PeerConnectedEvent as PoolPeerConnectedEvent, 
    PeerDisconnectedEvent as PoolPeerDisconnectedEvent, 
    NewPeakHeightEvent as PoolNewPeakEvent,
    PeerInfo as PoolPeerInfo,
    PoolStats
};
use napi::bindgen_prelude::*;
use napi::{
    threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
    JsFunction,
};
use napi_derive::napi;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

// NAPI wrapper for the peer pool
#[napi]
pub struct ChiaPeerPool {
    pool: Arc<PoolImplementation>,
    listeners: Arc<RwLock<EventListeners>>,
}

struct EventListeners {
    peer_connected_listeners: Vec<ThreadsafeFunction<PoolPeerConnectedEvent, ErrorStrategy::Fatal>>,
    peer_disconnected_listeners: Vec<ThreadsafeFunction<PoolPeerDisconnectedEvent, ErrorStrategy::Fatal>>,
    new_peak_height_listeners: Vec<ThreadsafeFunction<PoolNewPeakEvent, ErrorStrategy::Fatal>>,
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
pub struct NewPeakHeightEvent {
    pub old_peak: Option<u32>,
    pub new_peak: u32,
    pub peer_id: String,
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
pub struct PeerPoolStats {
    pub total_peers: u32,
    pub connected_peers: u32,
    pub failed_peers: u32,
    pub total_requests: String,      // u64 as string
    pub successful_requests: String, // u64 as string
    pub failed_requests: String,     // u64 as string
    pub average_response_time_ms: f64,
    pub current_peak_height: Option<u32>,
    pub uptime_seconds: String,      // u64 as string
    pub bytes_received: String,      // u64 as string
    pub bytes_sent: String,          // u64 as string
}

#[napi]
impl ChiaPeerPool {
    /// Convert internal BlockReceivedEvent to NAPI-compatible version
    fn convert_block_event(event: PoolBlockEvent) -> BlockReceivedEvent {
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
    fn convert_peer_info(info: PoolPeerInfo) -> PeerInfo {
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

    /// Convert internal PoolStats to NAPI-compatible version
    fn convert_pool_stats(stats: PoolStats) -> PeerPoolStats {
        PeerPoolStats {
            total_peers: stats.total_peers as u32,
            connected_peers: stats.connected_peers as u32,
            failed_peers: stats.failed_peers as u32,
            total_requests: stats.total_requests.to_string(),
            successful_requests: stats.successful_requests.to_string(),
            failed_requests: stats.failed_requests.to_string(),
            average_response_time_ms: stats.average_response_time_ms,
            current_peak_height: stats.current_peak_height,
            uptime_seconds: stats.uptime_seconds.to_string(),
            bytes_received: stats.bytes_received.to_string(),
            bytes_sent: stats.bytes_sent.to_string(),
        }
    }

    #[napi(constructor)]
    pub fn new() -> Self {
        info!("🚀 Creating new ChiaPeerPool with chia-peer-pool crate");
        
        let listeners = Arc::new(RwLock::new(EventListeners {
            peer_connected_listeners: Vec::new(),
            peer_disconnected_listeners: Vec::new(),
            new_peak_height_listeners: Vec::new(),
        }));

        // Create pool with default configuration
        let config = PeerPoolConfig::default();
        let pool = Arc::new(PoolImplementation::new(config));

        info!("✅ ChiaPeerPool created successfully");

        Self { pool, listeners }
    }

    #[napi(factory)]
    pub fn new_with_config(network_id: String, max_peers: u32) -> Self {
        info!("🚀 Creating ChiaPeerPool with custom config: network={}, max_peers={}", network_id, max_peers);
        
        let listeners = Arc::new(RwLock::new(EventListeners {
            peer_connected_listeners: Vec::new(),
            peer_disconnected_listeners: Vec::new(),
            new_peak_height_listeners: Vec::new(),
        }));

        let config = PeerPoolConfig {
            network_id,
            max_peers: max_peers as usize,
            ..PeerPoolConfig::default()
        };

        let pool = Arc::new(PoolImplementation::new(config));

        Self { pool, listeners }
    }

    #[napi(js_name = "addPeer")]
    pub async fn add_peer(&self, host: String, port: u16) -> Result<String> {
        info!("➕ Adding peer: {}:{}", host, port);
        
        self.pool
            .add_peer(host, port)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to add peer: {}", e)))
    }

    #[napi(js_name = "removePeer")]
    pub async fn remove_peer(&self, peer_id: String) -> Result<bool> {
        info!("➖ Removing peer: {}", peer_id);
        
        self.pool
            .remove_peer(peer_id)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to remove peer: {}", e)))
    }

    #[napi(js_name = "getBlock")]
    pub async fn get_block(&self, height: u32) -> Result<BlockReceivedEvent> {
        info!("📦 Requesting block at height: {}", height);
        
        let block_event = self.pool
            .get_block(height as u64)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get block: {}", e)))?;
        
        Ok(Self::convert_block_event(block_event))
    }

    #[napi(js_name = "getBlocks")]
    pub async fn get_blocks(&self, heights: Vec<u32>) -> Result<Vec<BlockReceivedEvent>> {
        info!("📦 Requesting {} blocks", heights.len());
        
        let heights_u64: Vec<u64> = heights.into_iter().map(|h| h as u64).collect();
        
        let block_events = self.pool
            .get_blocks(heights_u64)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get blocks: {}", e)))?;
        
        Ok(block_events.into_iter().map(Self::convert_block_event).collect())
    }

    #[napi(js_name = "getPeerInfo")]
    pub async fn get_peer_info(&self) -> Result<Vec<PeerInfo>> {
        let peer_infos = self.pool
            .get_peer_info()
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get peer info: {}", e)))?;
        
        Ok(peer_infos.into_iter().map(Self::convert_peer_info).collect())
    }

    #[napi(js_name = "getStats")]
    pub async fn get_stats(&self) -> Result<PeerPoolStats> {
        let stats = self.pool
            .get_stats()
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get stats: {}", e)))?;
        
        Ok(Self::convert_pool_stats(stats))
    }

    #[napi(js_name = "getPeakHeight")]
    pub async fn get_peak_height(&self) -> Option<u32> {
        self.pool.get_peak_height().await
    }

    #[napi(js_name = "getConnectedPeers")]
    pub async fn get_connected_peers(&self) -> Vec<String> {
        self.pool.get_connected_peers().await
    }

    #[napi(js_name = "shutdown")]
    pub async fn shutdown(&self) -> Result<()> {
        info!("🛑 Shutting down peer pool");
        
        // Note: The actual shutdown would require &mut self which NAPI doesn't support well
        // In practice, the pool will be cleaned up when this object is garbage collected
        info!("✅ Peer pool shutdown requested (resources will be cleaned up on garbage collection)");
        Ok(())
    }

    #[napi(js_name = "onPeerConnected")]
    pub async fn on_peer_connected(&self, callback: JsFunction) -> Result<()> {
        let tsfn: ThreadsafeFunction<PoolPeerConnectedEvent, ErrorStrategy::Fatal> = callback
            .create_threadsafe_function(0, |ctx| {
                ctx.env.create_object_with_properties(&[
                    ("peerId", ctx.env.create_string(&ctx.value.peer_id)?),
                    ("host", ctx.env.create_string(&ctx.value.host)?),
                    ("port", ctx.env.create_uint32(ctx.value.port as u32)?),
                    ("connectedAt", ctx.env.create_string(&ctx.value.connected_at.to_rfc3339())?),
                ])
            })?;

        self.listeners.write().await.peer_connected_listeners.push(tsfn);
        Ok(())
    }

    #[napi(js_name = "onPeerDisconnected")]
    pub async fn on_peer_disconnected(&self, callback: JsFunction) -> Result<()> {
        let tsfn: ThreadsafeFunction<PoolPeerDisconnectedEvent, ErrorStrategy::Fatal> = callback
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
        Ok(())
    }

    #[napi(js_name = "onNewPeakHeight")]
    pub async fn on_new_peak_height(&self, callback: JsFunction) -> Result<()> {
        let tsfn: ThreadsafeFunction<PoolNewPeakEvent, ErrorStrategy::Fatal> = callback
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

        self.listeners.write().await.new_peak_height_listeners.push(tsfn);
        Ok(())
    }

    #[napi(js_name = "clearEventListeners")]
    pub async fn clear_event_listeners(&self) {
        let mut listeners = self.listeners.write().await;
        listeners.peer_connected_listeners.clear();
        listeners.peer_disconnected_listeners.clear();
        listeners.new_peak_height_listeners.clear();
        info!("🧹 Cleared all event listeners");
    }
}

impl Default for ChiaPeerPool {
    fn default() -> Self {
        Self::new()
    }
}
