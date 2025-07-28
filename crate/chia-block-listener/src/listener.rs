use crate::error::{BlockListenerError, Result};
use crate::peer_connection::PeerConnection;
use crate::types::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

/// High-performance real-time Chia blockchain event listener
pub struct ChiaBlockListener {
    inner: Arc<RwLock<ListenerInner>>,
    config: BlockListenerConfig,
    message_tx: mpsc::Sender<ListenerMessage>,
    is_running: Arc<RwLock<bool>>,
}

struct ListenerInner {
    peers: HashMap<String, PeerState>,
    stats: ListenerStats,
    current_peak: Option<u32>,
    started_at: Instant,
    event_handlers: Vec<Arc<dyn EventHandler>>,
}

struct PeerState {
    info: PeerInfo,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl ChiaBlockListener {
    /// Create a new block listener with the given configuration
    pub fn new(config: BlockListenerConfig) -> Self {
        let (message_tx, message_rx) = mpsc::channel(1000);
        let (event_tx, event_rx) = mpsc::channel(1000);

        let inner = Arc::new(RwLock::new(ListenerInner {
            peers: HashMap::new(),
            stats: ListenerStats {
                total_peers: 0,
                connected_peers: 0,
                failed_peers: 0,
                total_blocks_received: 0,
                total_peaks_discovered: 0,
                uptime_seconds: 0,
                total_bytes_received: 0,
                total_bytes_sent: 0,
                current_peak_height: None,
                last_block_received: None,
                last_peak_update: None,
            },
            current_peak: None,
            started_at: Instant::now(),
            event_handlers: Vec::new(),
        }));

        let listener = Self {
            inner: Arc::clone(&inner),
            config: config.clone(),
            message_tx,
            is_running: Arc::new(RwLock::new(false)),
        };

        // Start background tasks
        tokio::spawn(Self::message_handler(
            Arc::clone(&inner),
            config.clone(),
            message_rx,
            event_tx.clone(),
        ));

        tokio::spawn(Self::event_handler(
            Arc::clone(&inner),
            event_rx,
        ));

        tokio::spawn(Self::stats_updater(
            Arc::clone(&inner),
        ));

        listener
    }

    /// Start the block listener
    pub async fn start(&self) -> Result<()> {
        {
            let mut running = self.is_running.write().await;
            if *running {
                return Err(BlockListenerError::ListenerAlreadyRunning);
            }
            *running = true;
        }

        info!("🎧 Starting Chia Block Listener");
        info!("📡 Real-time blockchain event monitoring");
        info!("🌐 Network: {}", self.config.network_id);
        info!("👥 Max peers: {}", self.config.max_peers);

        // Initialize stats
        {
            let mut inner = self.inner.write().await;
            inner.started_at = Instant::now();
        }

        info!("✅ Chia Block Listener started successfully");
        Ok(())
    }

    /// Stop the block listener
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping Chia Block Listener...");

        {
            let mut running = self.is_running.write().await;
            *running = false;
        }

        // Send shutdown message
        self.message_tx
            .send(ListenerMessage::Shutdown)
            .await
            .map_err(|_| BlockListenerError::EventChannelClosed)?;

        // Disconnect all peers
        let peer_ids: Vec<String> = {
            let inner = self.inner.read().await;
            inner.peers.keys().cloned().collect()
        };

        for peer_id in peer_ids {
            let _ = self.remove_peer(&peer_id).await;
        }

        info!("✅ Chia Block Listener stopped");
        Ok(())
    }

    /// Add a peer to listen to
    pub async fn add_peer(&self, host: String, port: u16) -> Result<String> {
        let connection_info = ConnectionInfo::new(host, port, self.config.network_id.clone());
        let peer_id = connection_info.peer_id.clone();

        let message = ListenerMessage::AddPeer(connection_info);
        self.message_tx
            .send(message)
            .await
            .map_err(|_| BlockListenerError::EventChannelClosed)?;

        info!("➕ Added peer: {}", peer_id);
        Ok(peer_id)
    }

    /// Remove a peer
    pub async fn remove_peer(&self, peer_id: &str) -> Result<bool> {
        let message = ListenerMessage::RemovePeer(peer_id.to_string());
        self.message_tx
            .send(message)
            .await
            .map_err(|_| BlockListenerError::EventChannelClosed)?;

        info!("➖ Removed peer: {}", peer_id);
        Ok(true)
    }

    /// Get information about all peers
    pub async fn get_peer_info(&self) -> Result<Vec<PeerInfo>> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let message = ListenerMessage::GetPeerInfo(response_tx);
        self.message_tx
            .send(message)
            .await
            .map_err(|_| BlockListenerError::EventChannelClosed)?;

        response_rx
            .await
            .map_err(|_| BlockListenerError::EventChannelClosed)
    }

    /// Get listener statistics
    pub async fn get_stats(&self) -> Result<ListenerStats> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let message = ListenerMessage::GetStats(response_tx);
        self.message_tx
            .send(message)
            .await
            .map_err(|_| BlockListenerError::EventChannelClosed)?;

        response_rx
            .await
            .map_err(|_| BlockListenerError::EventChannelClosed)
    }

    /// Get list of connected peers
    pub async fn get_connected_peers(&self) -> Vec<String> {
        let inner = self.inner.read().await;
        inner
            .peers
            .iter()
            .filter(|(_, state)| state.info.state == PeerState::Connected)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get current peak height
    pub async fn get_peak_height(&self) -> Option<u32> {
        let inner = self.inner.read().await;
        inner.current_peak
    }

    /// Add an event handler
    pub async fn add_event_handler(&self, handler: Arc<dyn EventHandler>) {
        let mut inner = self.inner.write().await;
        inner.event_handlers.push(handler);
    }

    /// Remove all event handlers
    pub async fn clear_event_handlers(&self) {
        let mut inner = self.inner.write().await;
        inner.event_handlers.clear();
    }

    /// Check if the listener is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Message handler task
    async fn message_handler(
        inner: Arc<RwLock<ListenerInner>>,
        config: BlockListenerConfig,
        mut message_rx: mpsc::Receiver<ListenerMessage>,
        event_tx: mpsc::Sender<InternalEvent>,
    ) {
        info!("📬 Block listener message handler started");

        while let Some(message) = message_rx.recv().await {
            match message {
                ListenerMessage::AddPeer(connection_info) => {
                    if let Err(e) = Self::handle_add_peer(
                        &inner,
                        &config,
                        connection_info,
                        &event_tx,
                    ).await {
                        error!("Failed to add peer: {}", e);
                    }
                }
                ListenerMessage::RemovePeer(peer_id) => {
                    Self::handle_remove_peer(&inner, &peer_id).await;
                }
                ListenerMessage::GetPeerInfo(response_tx) => {
                    let info = Self::handle_get_peer_info(&inner).await;
                    let _ = response_tx.send(info);
                }
                ListenerMessage::GetStats(response_tx) => {
                    let stats = Self::handle_get_stats(&inner).await;
                    let _ = response_tx.send(stats);
                }
                ListenerMessage::Shutdown => {
                    info!("📬 Shutdown message received");
                    break;
                }
            }
        }

        info!("📬 Block listener message handler stopped");
    }

    /// Handle adding a new peer
    async fn handle_add_peer(
        inner: &Arc<RwLock<ListenerInner>>,
        config: &BlockListenerConfig,
        connection_info: ConnectionInfo,
        event_tx: &mpsc::Sender<InternalEvent>,
    ) -> Result<()> {
        let peer_id = connection_info.peer_id.clone();

        // Check if peer already exists
        {
            let listener_inner = inner.read().await;
            if listener_inner.peers.contains_key(&peer_id) {
                return Err(BlockListenerError::PeerAlreadyConnected { peer_id });
            }

            if listener_inner.peers.len() >= config.max_peers {
                return Err(BlockListenerError::Other("Maximum peers reached".to_string()));
            }
        }

        // Create peer connection
        let handshake_info = HandshakeInfo {
            network_id: config.network_id.clone(),
            ..HandshakeInfo::default()
        };

        let peer_connection = PeerConnection::new(connection_info, handshake_info)?;
        let peer_info = peer_connection.get_peer_info();

        // Start peer connection task
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let inner_clone = Arc::clone(inner);
        let event_tx_clone = event_tx.clone();
        let peer_id_clone = peer_id.clone();

        tokio::spawn(async move {
            if let Err(e) = peer_connection.connect_and_listen(event_tx_clone, shutdown_rx).await {
                error!("Peer connection failed for {}: {}", peer_id_clone, e);
                
                // Update peer state to failed
                {
                    let mut listener_inner = inner_clone.write().await;
                    if let Some(state) = listener_inner.peers.get_mut(&peer_id_clone) {
                        state.info.state = PeerState::Failed;
                        state.info.last_error = Some(e.to_string());
                        listener_inner.stats.failed_peers += 1;
                    }
                }
            }
        });

        // Add to peer list
        {
            let mut listener_inner = inner.write().await;
            listener_inner.peers.insert(
                peer_id.clone(),
                PeerState {
                    info: peer_info,
                    shutdown_tx: Some(shutdown_tx),
                },
            );
            listener_inner.stats.total_peers = listener_inner.peers.len();
        }

        debug!("Added peer {} to listener", peer_id);
        Ok(())
    }

    /// Handle removing a peer
    async fn handle_remove_peer(inner: &Arc<RwLock<ListenerInner>>, peer_id: &str) {
        let mut listener_inner = inner.write().await;
        
        if let Some(state) = listener_inner.peers.remove(peer_id) {
            // Send shutdown signal to peer connection
            if let Some(shutdown_tx) = state.shutdown_tx {
                let _ = shutdown_tx.send(());
            }
            
            listener_inner.stats.total_peers = listener_inner.peers.len();
            debug!("Removed peer {} from listener", peer_id);
        }
    }

    /// Handle getting peer information
    async fn handle_get_peer_info(inner: &Arc<RwLock<ListenerInner>>) -> Vec<PeerInfo> {
        let listener_inner = inner.read().await;
        listener_inner.peers.values().map(|state| state.info.clone()).collect()
    }

    /// Handle getting statistics
    async fn handle_get_stats(inner: &Arc<RwLock<ListenerInner>>) -> ListenerStats {
        let mut listener_inner = inner.write().await;
        
        // Update uptime
        listener_inner.stats.uptime_seconds = listener_inner.started_at.elapsed().as_secs();
        
        // Update connected peers count
        listener_inner.stats.connected_peers = listener_inner
            .peers
            .values()
            .filter(|state| state.info.state == PeerState::Connected)
            .count();
        
        // Update failed peers count
        listener_inner.stats.failed_peers = listener_inner
            .peers
            .values()
            .filter(|state| state.info.state == PeerState::Failed)
            .count();

        listener_inner.stats.clone()
    }

    /// Event handler task
    async fn event_handler(
        inner: Arc<RwLock<ListenerInner>>,
        mut event_rx: mpsc::Receiver<InternalEvent>,
    ) {
        info!("🎪 Block listener event handler started");

        while let Some(event) = event_rx.recv().await {
            Self::process_event(&inner, event).await;
        }

        info!("🎪 Block listener event handler stopped");
    }

    /// Process a single event
    async fn process_event(inner: &Arc<RwLock<ListenerInner>>, event: InternalEvent) {
        match &event {
            InternalEvent::PeerConnected(peer_event) => {
                {
                    let mut listener_inner = inner.write().await;
                    if let Some(state) = listener_inner.peers.get_mut(&peer_event.peer_id) {
                        state.info.state = PeerState::Connected;
                        state.info.connected_at = Some(peer_event.connected_at);
                        state.info.last_activity = Some(peer_event.connected_at);
                    }
                    listener_inner.stats.connected_peers += 1;
                }

                info!("🔗 Peer connected: {} ({}:{})", 
                      peer_event.peer_id, peer_event.host, peer_event.port);
            }
            InternalEvent::PeerDisconnected(peer_event) => {
                {
                    let mut listener_inner = inner.write().await;
                    if let Some(state) = listener_inner.peers.get_mut(&peer_event.peer_id) {
                        state.info.state = PeerState::Disconnected;
                        if listener_inner.stats.connected_peers > 0 {
                            listener_inner.stats.connected_peers -= 1;
                        }
                    }
                }

                warn!("💔 Peer disconnected: {} - {}", peer_event.peer_id, peer_event.reason);
            }
            InternalEvent::BlockReceived(block_event) => {
                {
                    let mut listener_inner = inner.write().await;
                    listener_inner.stats.total_blocks_received += 1;
                    listener_inner.stats.last_block_received = Some(block_event.received_at);
                    
                    if let Some(state) = listener_inner.peers.get_mut(&block_event.peer_id) {
                        state.info.blocks_received += 1;
                        state.info.last_activity = Some(block_event.received_at);
                    }
                }

                info!("📦 Block {} received from peer {}", 
                      block_event.height, block_event.peer_id);
            }
            InternalEvent::NewPeak(peak_event) => {
                {
                    let mut listener_inner = inner.write().await;
                    listener_inner.current_peak = Some(peak_event.new_peak);
                    listener_inner.stats.current_peak_height = Some(peak_event.new_peak);
                    listener_inner.stats.total_peaks_discovered += 1;
                    listener_inner.stats.last_peak_update = Some(peak_event.discovered_at);
                    
                    if let Some(state) = listener_inner.peers.get_mut(&peak_event.peer_id) {
                        state.info.last_peak_height = Some(peak_event.new_peak);
                        state.info.last_activity = Some(peak_event.discovered_at);
                    }
                }

                info!("🎯 New peak height: {} from peer {}", 
                      peak_event.new_peak, peak_event.peer_id);
            }
            InternalEvent::Error { peer_id, error } => {
                {
                    let mut listener_inner = inner.write().await;
                    if let Some(state) = listener_inner.peers.get_mut(peer_id) {
                        state.info.last_error = Some(error.clone());
                    }
                }

                warn!("⚠️ Error from peer {}: {}", peer_id, error);
            }
        }

        // Call event handlers
        let handlers = {
            let listener_inner = inner.read().await;
            listener_inner.event_handlers.clone()
        };

        for handler in handlers {
            match &event {
                InternalEvent::BlockReceived(e) => handler.on_block_received(e.clone()),
                InternalEvent::PeerConnected(e) => handler.on_peer_connected(e.clone()),
                InternalEvent::PeerDisconnected(e) => handler.on_peer_disconnected(e.clone()),
                InternalEvent::NewPeak(e) => handler.on_new_peak(e.clone()),
                InternalEvent::Error { .. } => {}, // Error events don't have handlers
            }
        }
    }

    /// Statistics updater task
    async fn stats_updater(inner: Arc<RwLock<ListenerInner>>) {
        let mut interval = interval(Duration::from_secs(60)); // Update every minute
        info!("📊 Block listener stats updater started");

        loop {
            interval.tick().await;

            {
                let mut listener_inner = inner.write().await;
                listener_inner.stats.uptime_seconds = listener_inner.started_at.elapsed().as_secs();
            }

            debug!("📊 Updated listener statistics");
        }
    }
}

impl Default for ChiaBlockListener {
    fn default() -> Self {
        Self::new(BlockListenerConfig::default())
    }
}

/// Simple event handler implementation for logging
pub struct LoggingEventHandler;

impl EventHandler for LoggingEventHandler {
    fn on_block_received(&self, event: BlockReceivedEvent) {
        info!("🎉 Block received: {} (height: {}, peer: {})", 
              event.header_hash, event.height, event.peer_id);
    }

    fn on_peer_connected(&self, event: PeerConnectedEvent) {
        info!("🎉 Peer connected: {} ({}:{})", 
              event.peer_id, event.host, event.port);
    }

    fn on_peer_disconnected(&self, event: PeerDisconnectedEvent) {
        warn!("🎉 Peer disconnected: {} - {}", 
              event.peer_id, event.reason);
    }

    fn on_new_peak(&self, event: NewPeakEvent) {
        info!("🎉 New peak: {} from peer {}", 
              event.new_peak, event.peer_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_listener_creation() {
        let config = BlockListenerConfig::default();
        let listener = ChiaBlockListener::new(config);
        
        assert!(!listener.is_running().await);
        let stats = listener.get_stats().await.unwrap();
        assert_eq!(stats.total_peers, 0);
    }

    #[tokio::test]
    async fn test_listener_start_stop() {
        let config = BlockListenerConfig::default();
        let listener = ChiaBlockListener::new(config);
        
        assert!(listener.start().await.is_ok());
        assert!(listener.is_running().await);
        
        assert!(listener.stop().await.is_ok());
        assert!(!listener.is_running().await);
    }

    #[tokio::test]
    async fn test_peer_management() {
        let config = BlockListenerConfig::default();
        let listener = ChiaBlockListener::new(config);
        
        listener.start().await.unwrap();
        
        // Adding a peer should work (though connection will fail in tests)
        let peer_id = listener.add_peer("localhost".to_string(), 8444).await.unwrap();
        assert_eq!(peer_id, "localhost:8444");
        
        // Removing the peer should work
        let removed = listener.remove_peer(&peer_id).await.unwrap();
        assert!(removed);
        
        listener.stop().await.unwrap();
    }

    #[test]
    fn test_logging_event_handler() {
        let handler = LoggingEventHandler;
        
        let event = BlockReceivedEvent {
            height: 1000,
            header_hash: "test_hash".to_string(),
            timestamp: 0,
            weight: "1000".to_string(),
            is_transaction_block: true,
            peer_id: "test_peer".to_string(),
            received_at: chrono::Utc::now(),
            coin_additions: vec![],
            coin_removals: vec![],
            coin_spends: vec![],
            coin_creations: vec![],
        };
        
        // Just test that the handler doesn't panic
        handler.on_block_received(event);
    }
} 