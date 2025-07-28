use crate::error::{PeerPoolError, Result};
use crate::peer_connection::PeerConnection;
use crate::types::*;
use chia_protocol::FullBlock;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};

/// High-performance peer pool for batch blockchain data fetching
pub struct ChiaPeerPool {
    inner: Arc<RwLock<PoolInner>>,
    config: PeerPoolConfig,
    request_tx: mpsc::Sender<PoolRequest>,
    event_tx: mpsc::Sender<PoolEvent>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

struct PoolInner {
    peers: HashMap<String, PeerState>,
    peer_order: VecDeque<String>, // For round-robin scheduling
    current_peak: Option<u32>,
    stats: PoolStats,
    started_at: Instant,
}

struct PeerState {
    info: PeerInfo,
    connection: Option<PeerConnection>,
    worker_tx: Option<mpsc::Sender<WorkerRequest>>,
    last_used: Instant,
    is_rate_limited: bool,
}

enum PoolRequest {
    GetBlock {
        height: u64,
        response_tx: oneshot::Sender<Result<BlockReceivedEvent>>,
    },
    GetBlocks {
        heights: Vec<u64>,
        response_tx: oneshot::Sender<Result<Vec<BlockReceivedEvent>>>,
    },
    AddPeer {
        host: String,
        port: u16,
        response_tx: oneshot::Sender<Result<String>>,
    },
    RemovePeer {
        peer_id: String,
        response_tx: oneshot::Sender<Result<bool>>,
    },
    GetPeerInfo {
        response_tx: oneshot::Sender<Vec<PeerInfo>>,
    },
    GetStats {
        response_tx: oneshot::Sender<PoolStats>,
    },
}

enum WorkerRequest {
    GetBlock {
        height: u64,
        response_tx: oneshot::Sender<Result<FullBlock>>,
    },
    Shutdown,
}

enum PoolEvent {
    PeerConnected(PeerConnectedEvent),
    PeerDisconnected(PeerDisconnectedEvent),
    NewPeakHeight(NewPeakHeightEvent),
    BlockReceived(BlockReceivedEvent),
}

impl ChiaPeerPool {
    /// Create a new peer pool with the given configuration
    pub fn new(config: PeerPoolConfig) -> Self {
        let (request_tx, request_rx) = mpsc::channel(1000);
        let (event_tx, event_rx) = mpsc::channel(1000);

        let inner = Arc::new(RwLock::new(PoolInner {
            peers: HashMap::new(),
            peer_order: VecDeque::new(),
            current_peak: None,
            stats: PoolStats {
                total_peers: 0,
                connected_peers: 0,
                failed_peers: 0,
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                average_response_time_ms: 0.0,
                current_peak_height: None,
                uptime_seconds: 0,
                bytes_received: 0,
                bytes_sent: 0,
            },
            started_at: Instant::now(),
        }));

        let pool = Self {
            inner: Arc::clone(&inner),
            config: config.clone(),
            request_tx,
            event_tx: event_tx.clone(),
            shutdown_tx: None,
        };

        // Start background tasks
        tokio::spawn(Self::request_handler(
            Arc::clone(&inner),
            config.clone(),
            request_rx,
            event_tx.clone(),
        ));

        tokio::spawn(Self::event_handler(event_rx));

        if config.enable_peak_monitoring {
            tokio::spawn(Self::health_monitor(
                Arc::clone(&inner),
                config.health_check_interval_secs,
            ));
        }

        pool
    }

    /// Add a peer to the pool
    pub async fn add_peer(&self, host: String, port: u16) -> Result<String> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.request_tx
            .send(PoolRequest::AddPeer {
                host,
                port,
                response_tx,
            })
            .await
            .map_err(|_| PeerPoolError::PoolShutdown)?;

        response_rx
            .await
            .map_err(|_| PeerPoolError::PoolShutdown)?
    }

    /// Remove a peer from the pool
    pub async fn remove_peer(&self, peer_id: String) -> Result<bool> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.request_tx
            .send(PoolRequest::RemovePeer {
                peer_id,
                response_tx,
            })
            .await
            .map_err(|_| PeerPoolError::PoolShutdown)?;

        response_rx
            .await
            .map_err(|_| PeerPoolError::PoolShutdown)?
    }

    /// Get a single block by height
    pub async fn get_block(&self, height: u64) -> Result<BlockReceivedEvent> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.request_tx
            .send(PoolRequest::GetBlock {
                height,
                response_tx,
            })
            .await
            .map_err(|_| PeerPoolError::PoolShutdown)?;

        timeout(Duration::from_secs(self.config.request_timeout_secs), response_rx)
            .await
            .map_err(|_| PeerPoolError::RequestTimeout)?
            .map_err(|_| PeerPoolError::PoolShutdown)?
    }

    /// Get multiple blocks by height (batch operation)
    pub async fn get_blocks(&self, heights: Vec<u64>) -> Result<Vec<BlockReceivedEvent>> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.request_tx
            .send(PoolRequest::GetBlocks {
                heights,
                response_tx,
            })
            .await
            .map_err(|_| PeerPoolError::PoolShutdown)?;

        timeout(
            Duration::from_secs(self.config.request_timeout_secs * 5), // Longer timeout for batch
            response_rx,
        )
        .await
        .map_err(|_| PeerPoolError::RequestTimeout)?
        .map_err(|_| PeerPoolError::PoolShutdown)?
    }

    /// Get information about all peers
    pub async fn get_peer_info(&self) -> Result<Vec<PeerInfo>> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.request_tx
            .send(PoolRequest::GetPeerInfo { response_tx })
            .await
            .map_err(|_| PeerPoolError::PoolShutdown)?;

        response_rx
            .await
            .map_err(|_| PeerPoolError::PoolShutdown)
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> Result<PoolStats> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.request_tx
            .send(PoolRequest::GetStats { response_tx })
            .await
            .map_err(|_| PeerPoolError::PoolShutdown)?;

        response_rx
            .await
            .map_err(|_| PeerPoolError::PoolShutdown)
    }

    /// Get the current peak height across all peers
    pub async fn get_peak_height(&self) -> Option<u32> {
        let inner = self.inner.read().await;
        inner.current_peak
    }

    /// Get list of connected peer IDs
    pub async fn get_connected_peers(&self) -> Vec<String> {
        let inner = self.inner.read().await;
        inner
            .peers
            .iter()
            .filter(|(_, state)| state.info.state == PeerState::Connected)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Shutdown the peer pool
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }

        // Disconnect all peers
        let peer_ids: Vec<String> = {
            let inner = self.inner.read().await;
            inner.peers.keys().cloned().collect()
        };

        for peer_id in peer_ids {
            let _ = self.remove_peer(peer_id).await;
        }

        info!("Peer pool shutdown complete");
        Ok(())
    }

    /// Main request handler task
    async fn request_handler(
        inner: Arc<RwLock<PoolInner>>,
        config: PeerPoolConfig,
        mut request_rx: mpsc::Receiver<PoolRequest>,
        event_tx: mpsc::Sender<PoolEvent>,
    ) {
        info!("Peer pool request handler started");

        while let Some(request) = request_rx.recv().await {
            match request {
                PoolRequest::GetBlock { height, response_tx } => {
                    let result = Self::handle_get_block(
                        &inner,
                        &config,
                        height,
                    ).await;
                    let _ = response_tx.send(result);
                }
                PoolRequest::GetBlocks { heights, response_tx } => {
                    let result = Self::handle_get_blocks(
                        &inner,
                        &config,
                        heights,
                    ).await;
                    let _ = response_tx.send(result);
                }
                PoolRequest::AddPeer { host, port, response_tx } => {
                    let result = Self::handle_add_peer(
                        &inner,
                        &config,
                        host,
                        port,
                        &event_tx,
                    ).await;
                    let _ = response_tx.send(result);
                }
                PoolRequest::RemovePeer { peer_id, response_tx } => {
                    let result = Self::handle_remove_peer(
                        &inner,
                        peer_id,
                        &event_tx,
                    ).await;
                    let _ = response_tx.send(result);
                }
                PoolRequest::GetPeerInfo { response_tx } => {
                    let info = Self::handle_get_peer_info(&inner).await;
                    let _ = response_tx.send(info);
                }
                PoolRequest::GetStats { response_tx } => {
                    let stats = Self::handle_get_stats(&inner).await;
                    let _ = response_tx.send(stats);
                }
            }
        }

        info!("Peer pool request handler stopped");
    }

    /// Handle single block request
    async fn handle_get_block(
        inner: &Arc<RwLock<PoolInner>>,
        config: &PeerPoolConfig,
        height: u64,
    ) -> Result<BlockReceivedEvent> {
        // Find best available peer
        let peer_id = {
            let mut pool_inner = inner.write().await;
            Self::select_best_peer(&mut pool_inner, config)?
        };

        // Make request to selected peer
        Self::request_block_from_peer(inner, &peer_id, height).await
    }

    /// Handle batch block request
    async fn handle_get_blocks(
        inner: &Arc<RwLock<PoolInner>>,
        config: &PeerPoolConfig,
        heights: Vec<u64>,
    ) -> Result<Vec<BlockReceivedEvent>> {
        let mut results = Vec::with_capacity(heights.len());
        let mut failed_requests = Vec::new();

        // Process requests in parallel batches
        let batch_size = 10; // Limit concurrent requests per peer
        for height_batch in heights.chunks(batch_size) {
            let mut batch_tasks = Vec::new();

            for &height in height_batch {
                let inner_clone = Arc::clone(inner);
                let config_clone = config.clone();
                
                let task = tokio::spawn(async move {
                    Self::handle_get_block(&inner_clone, &config_clone, height).await
                });
                batch_tasks.push((height, task));
            }

            // Collect batch results
            for (height, task) in batch_tasks {
                match task.await {
                    Ok(Ok(block)) => results.push(block),
                    Ok(Err(e)) => {
                        warn!("Failed to get block {}: {}", height, e);
                        failed_requests.push(height);
                    }
                    Err(e) => {
                        warn!("Task failed for block {}: {}", height, e);
                        failed_requests.push(height);
                    }
                }
            }
        }

        if failed_requests.is_empty() {
            Ok(results)
        } else {
            Err(PeerPoolError::Other(format!(
                "Failed to fetch {} blocks: {:?}",
                failed_requests.len(),
                failed_requests
            )))
        }
    }

    /// Select the best available peer for a request
    fn select_best_peer(
        pool_inner: &mut PoolInner,
        config: &PeerPoolConfig,
    ) -> Result<String> {
        let now = Instant::now();

        // Filter available peers
        let available_peers: Vec<String> = pool_inner
            .peers
            .iter()
            .filter(|(_, state)| {
                state.info.state == PeerState::Connected
                    && !state.is_rate_limited
                    && now.duration_since(state.last_used).as_millis() >= config.rate_limit_ms as u128
            })
            .map(|(id, _)| id.clone())
            .collect();

        if available_peers.is_empty() {
            return Err(PeerPoolError::NoPeersAvailable);
        }

        // Use round-robin selection
        let peer_id = if let Some(pos) = pool_inner.peer_order.iter().position(|id| available_peers.contains(id)) {
            // Rotate to next peer
            for _ in 0..=pos {
                if let Some(id) = pool_inner.peer_order.pop_front() {
                    pool_inner.peer_order.push_back(id);
                }
            }
            pool_inner.peer_order.front().unwrap().clone()
        } else {
            available_peers[0].clone()
        };

        // Update last used time
        if let Some(state) = pool_inner.peers.get_mut(&peer_id) {
            state.last_used = now;
        }

        Ok(peer_id)
    }

    /// Request a block from a specific peer
    async fn request_block_from_peer(
        inner: &Arc<RwLock<PoolInner>>,
        peer_id: &str,
        height: u64,
    ) -> Result<BlockReceivedEvent> {
        let worker_tx = {
            let pool_inner = inner.read().await;
            pool_inner
                .peers
                .get(peer_id)
                .and_then(|state| state.worker_tx.clone())
                .ok_or_else(|| PeerPoolError::PeerNotFound {
                    peer_id: peer_id.to_string(),
                })?
        };

        let (response_tx, response_rx) = oneshot::channel();
        worker_tx
            .send(WorkerRequest::GetBlock { height, response_tx })
            .await
            .map_err(|_| PeerPoolError::PeerDisconnected {
                peer_id: peer_id.to_string(),
            })?;

        let block = response_rx
            .await
            .map_err(|_| PeerPoolError::RequestTimeout)??;

        // Convert FullBlock to BlockReceivedEvent
        Self::convert_block_to_event(block, peer_id.to_string()).await
    }

    /// Convert FullBlock to BlockReceivedEvent
    async fn convert_block_to_event(
        block: FullBlock,
        peer_id: String,
    ) -> Result<BlockReceivedEvent> {
        // Parse block using the generator parser
        let parser = chia_generator_parser::BlockParser::new();
        let parsed_block = parser.parse_block(block)
            .map_err(|e| PeerPoolError::InvalidBlockData {
                reason: e.to_string(),
            })?;

        // Convert to event format
        Ok(BlockReceivedEvent {
            height: parsed_block.height,
            header_hash: hex::encode(&parsed_block.header_hash),
            timestamp: parsed_block.timestamp.unwrap_or(0) as u64,
            weight: parsed_block.weight.to_string(),
            is_transaction_block: parsed_block.has_transactions_generator,
            peer_id,
            received_at: chrono::Utc::now(),
            coin_additions: parsed_block.coin_additions.into_iter().map(|coin| CoinRecord {
                parent_coin_info: hex::encode(&coin.parent_coin_info),
                puzzle_hash: hex::encode(&coin.puzzle_hash),
                amount: coin.amount.to_string(),
            }).collect(),
            coin_removals: parsed_block.coin_removals.into_iter().map(|coin| CoinRecord {
                parent_coin_info: hex::encode(&coin.parent_coin_info),
                puzzle_hash: hex::encode(&coin.puzzle_hash),
                amount: coin.amount.to_string(),
            }).collect(),
            coin_spends: parsed_block.coin_spends.into_iter().map(|spend| CoinSpendRecord {
                coin: CoinRecord {
                    parent_coin_info: hex::encode(&spend.coin.parent_coin_info),
                    puzzle_hash: hex::encode(&spend.coin.puzzle_hash),
                    amount: spend.coin.amount.to_string(),
                },
                puzzle_reveal: hex::encode(&spend.puzzle_reveal),
                solution: hex::encode(&spend.solution),
            }).collect(),
            coin_creations: parsed_block.coin_creations.into_iter().map(|coin| CoinRecord {
                parent_coin_info: hex::encode(&coin.parent_coin_info),
                puzzle_hash: hex::encode(&coin.puzzle_hash),
                amount: coin.amount.to_string(),
            }).collect(),
        })
    }

    /// Handle adding a new peer
    async fn handle_add_peer(
        inner: &Arc<RwLock<PoolInner>>,
        config: &PeerPoolConfig,
        host: String,
        port: u16,
        event_tx: &mpsc::Sender<PoolEvent>,
    ) -> Result<String> {
        let peer_id = format!("{}:{}", host, port);
        
        // Check if we've reached the maximum number of peers
        {
            let pool_inner = inner.read().await;
            if pool_inner.peers.len() >= config.max_peers {
                return Err(PeerPoolError::Other("Maximum peers reached".to_string()));
            }
            
            if pool_inner.peers.contains_key(&peer_id) {
                return Err(PeerPoolError::Other("Peer already exists".to_string()));
            }
        }

        // Create peer connection
        let connection = PeerConnection::new(
            peer_id.clone(),
            host.clone(),
            port,
            config.network_id.clone(),
        );

        // Start peer worker
        let (worker_tx, worker_rx) = mpsc::channel(100);
        let peer_info = connection.get_info();
        
        tokio::spawn(Self::peer_worker(
            connection,
            worker_rx,
            Arc::clone(inner),
            event_tx.clone(),
        ));

        // Add to pool
        {
            let mut pool_inner = inner.write().await;
            pool_inner.peers.insert(
                peer_id.clone(),
                PeerState {
                    info: peer_info,
                    connection: None,
                    worker_tx: Some(worker_tx),
                    last_used: Instant::now(),
                    is_rate_limited: false,
                },
            );
            pool_inner.peer_order.push_back(peer_id.clone());
            pool_inner.stats.total_peers = pool_inner.peers.len();
        }

        info!("Added peer {} to pool", peer_id);
        Ok(peer_id)
    }

    /// Handle removing a peer
    async fn handle_remove_peer(
        inner: &Arc<RwLock<PoolInner>>,
        peer_id: String,
        event_tx: &mpsc::Sender<PoolEvent>,
    ) -> Result<bool> {
        let was_removed = {
            let mut pool_inner = inner.write().await;
            
            if let Some(state) = pool_inner.peers.remove(&peer_id) {
                // Remove from peer order
                pool_inner.peer_order.retain(|id| id != &peer_id);
                
                // Send shutdown to worker
                if let Some(worker_tx) = &state.worker_tx {
                    let _ = worker_tx.send(WorkerRequest::Shutdown).await;
                }
                
                pool_inner.stats.total_peers = pool_inner.peers.len();
                true
            } else {
                false
            }
        };

        if was_removed {
            info!("Removed peer {} from pool", peer_id);
            
            // Emit disconnection event
            let event = PoolEvent::PeerDisconnected(PeerDisconnectedEvent {
                peer_id: peer_id.clone(),
                host: "unknown".to_string(),
                port: 0,
                reason: "Removed from pool".to_string(),
                disconnected_at: chrono::Utc::now(),
            });
            let _ = event_tx.send(event).await;
        }

        Ok(was_removed)
    }

    /// Handle getting peer information
    async fn handle_get_peer_info(inner: &Arc<RwLock<PoolInner>>) -> Vec<PeerInfo> {
        let pool_inner = inner.read().await;
        pool_inner.peers.values().map(|state| state.info.clone()).collect()
    }

    /// Handle getting pool statistics
    async fn handle_get_stats(inner: &Arc<RwLock<PoolInner>>) -> PoolStats {
        let mut pool_inner = inner.write().await;
        
        // Update uptime
        pool_inner.stats.uptime_seconds = pool_inner.started_at.elapsed().as_secs();
        
        // Update connected peers count
        pool_inner.stats.connected_peers = pool_inner
            .peers
            .values()
            .filter(|state| state.info.state == PeerState::Connected)
            .count();
        
        // Update failed peers count
        pool_inner.stats.failed_peers = pool_inner
            .peers
            .values()
            .filter(|state| state.info.state == PeerState::Failed)
            .count();

        pool_inner.stats.clone()
    }

    /// Event handler task
    async fn event_handler(mut event_rx: mpsc::Receiver<PoolEvent>) {
        info!("Peer pool event handler started");

        while let Some(event) = event_rx.recv().await {
            match event {
                PoolEvent::PeerConnected(event) => {
                    info!("Peer connected: {} ({}:{})", event.peer_id, event.host, event.port);
                }
                PoolEvent::PeerDisconnected(event) => {
                    info!("Peer disconnected: {} - {}", event.peer_id, event.reason);
                }
                PoolEvent::NewPeakHeight(event) => {
                    info!("New peak height: {} from peer {}", event.new_peak, event.peer_id);
                }
                PoolEvent::BlockReceived(event) => {
                    debug!("Block {} received from peer {}", event.height, event.peer_id);
                }
            }
        }

        info!("Peer pool event handler stopped");
    }

    /// Health monitor task
    async fn health_monitor(inner: Arc<RwLock<PoolInner>>, interval_secs: u64) {
        let mut health_interval = interval(Duration::from_secs(interval_secs));
        info!("Peer pool health monitor started ({}s interval)", interval_secs);

        loop {
            health_interval.tick().await;

            let mut pool_inner = inner.write().await;
            let mut disconnected_peers = Vec::new();

            for (peer_id, state) in &mut pool_inner.peers {
                // Check if peer has been inactive too long
                if state.last_used.elapsed() > Duration::from_secs(300) {
                    warn!("Peer {} has been inactive for 5+ minutes", peer_id);
                }

                // Reset rate limiting if enough time has passed
                if state.is_rate_limited && state.last_used.elapsed() > Duration::from_millis(1000) {
                    state.is_rate_limited = false;
                }

                // Mark failed peers for removal
                if state.info.state == PeerState::Failed {
                    disconnected_peers.push(peer_id.clone());
                }
            }

            // Remove failed peers
            for peer_id in disconnected_peers {
                info!("Removing failed peer: {}", peer_id);
                pool_inner.peers.remove(&peer_id);
                pool_inner.peer_order.retain(|id| id != &peer_id);
            }

            drop(pool_inner);
        }
    }

    /// Individual peer worker task
    async fn peer_worker(
        connection: PeerConnection,
        mut worker_rx: mpsc::Receiver<WorkerRequest>,
        inner: Arc<RwLock<PoolInner>>,
        event_tx: mpsc::Sender<PoolEvent>,
    ) {
        let peer_id = connection.peer_id.clone();
        info!("Starting peer worker for {}", peer_id);

        // Try to connect
        let mut ws_stream = match connection.connect(30).await {
            Ok(stream) => {
                // Update peer state to connected
                {
                    let mut pool_inner = inner.write().await;
                    if let Some(state) = pool_inner.peers.get_mut(&peer_id) {
                        state.info.state = PeerState::Connected;
                        state.info.connected_at = Some(chrono::Utc::now());
                        pool_inner.stats.connected_peers += 1;
                    }
                }

                // Emit connected event
                let event = PoolEvent::PeerConnected(PeerConnectedEvent {
                    peer_id: peer_id.clone(),
                    host: connection.host.clone(),
                    port: connection.port,
                    connected_at: chrono::Utc::now(),
                });
                let _ = event_tx.send(event).await;

                stream
            }
            Err(e) => {
                error!("Failed to connect to peer {}: {}", peer_id, e);
                
                // Update peer state to failed
                {
                    let mut pool_inner = inner.write().await;
                    if let Some(state) = pool_inner.peers.get_mut(&peer_id) {
                        state.info.state = PeerState::Failed;
                        state.info.last_error = Some(e.to_string());
                        pool_inner.stats.failed_peers += 1;
                    }
                }

                return;
            }
        };

        // Perform handshake
        if let Err(e) = connection.handshake(&mut ws_stream).await {
            error!("Handshake failed with peer {}: {}", peer_id, e);
            return;
        }

        // Handle requests
        while let Some(request) = worker_rx.recv().await {
            match request {
                WorkerRequest::GetBlock { height, response_tx } => {
                    let start_time = Instant::now();
                    let result = connection.request_block(&mut ws_stream, height).await;
                    let response_time = start_time.elapsed().as_millis() as f64;

                    // Update statistics
                    {
                        let mut pool_inner = inner.write().await;
                        if let Some(state) = pool_inner.peers.get_mut(&peer_id) {
                            match &result {
                                Ok(_) => {
                                    state.info.successful_requests += 1;
                                    pool_inner.stats.successful_requests += 1;
                                }
                                Err(e) => {
                                    state.info.failed_requests += 1;
                                    state.info.last_error = Some(e.to_string());
                                    pool_inner.stats.failed_requests += 1;
                                }
                            }
                            
                            state.info.last_activity = Some(chrono::Utc::now());
                            pool_inner.stats.total_requests += 1;

                            // Update average response time
                            if let Some(avg) = state.info.average_response_time_ms {
                                state.info.average_response_time_ms = Some((avg + response_time) / 2.0);
                            } else {
                                state.info.average_response_time_ms = Some(response_time);
                            }
                        }
                    }

                    let _ = response_tx.send(result);
                }
                WorkerRequest::Shutdown => {
                    info!("Shutting down peer worker for {}", peer_id);
                    break;
                }
            }
        }

        // Clean up connection
        let _ = ws_stream.close(None).await;
        info!("Peer worker stopped for {}", peer_id);
    }
}

impl Default for ChiaPeerPool {
    fn default() -> Self {
        Self::new(PeerPoolConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_peer_pool_creation() {
        let config = PeerPoolConfig::default();
        let pool = ChiaPeerPool::new(config);
        
        let stats = pool.get_stats().await.unwrap();
        assert_eq!(stats.total_peers, 0);
        assert_eq!(stats.connected_peers, 0);
    }

    #[tokio::test]
    async fn test_peer_selection() {
        let mut pool_inner = PoolInner {
            peers: HashMap::new(),
            peer_order: VecDeque::new(),
            current_peak: None,
            stats: PoolStats {
                total_peers: 0,
                connected_peers: 0,
                failed_peers: 0,
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                average_response_time_ms: 0.0,
                current_peak_height: None,
                uptime_seconds: 0,
                bytes_received: 0,
                bytes_sent: 0,
            },
            started_at: Instant::now(),
        };

        let config = PeerPoolConfig::default();

        // Test with no peers
        let result = ChiaPeerPool::select_best_peer(&mut pool_inner, &config);
        assert!(result.is_err());

        // Add a connected peer
        let peer_id = "test_peer".to_string();
        pool_inner.peers.insert(
            peer_id.clone(),
            PeerState {
                info: PeerInfo::new(peer_id.clone(), "localhost".to_string(), 8444),
                connection: None,
                worker_tx: None,
                last_used: Instant::now() - Duration::from_secs(10), // Not rate limited
                is_rate_limited: false,
            },
        );
        pool_inner.peers.get_mut(&peer_id).unwrap().info.state = PeerState::Connected;
        pool_inner.peer_order.push_back(peer_id.clone());

        // Test peer selection
        let selected = ChiaPeerPool::select_best_peer(&mut pool_inner, &config).unwrap();
        assert_eq!(selected, peer_id);
    }
} 