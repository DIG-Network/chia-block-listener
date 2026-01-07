use crate::error::ChiaError;
use crate::types::{
    BlockReceivedEvent, CoinRecord, CoinSpend, NewPeakHeightEvent, PeerConnectedEvent,
    PeerDisconnectedEvent,
};
use crate::peer::PeerConnection;
use chia_generator_parser::{BlockParser, ParsedBlock};
use chia_protocol::FullBlock;

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::time::timeout;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

const RATE_LIMIT_MS: u64 = 500; // 500ms cooldown between peer usage
const REQUEST_TIMEOUT_MS: u64 = 5000; // 5 second timeout for block requests (reduced from 10s)
const CONNECTION_TIMEOUT_MS: u64 = 3000; // 3 second timeout for connections (reduced from 5s)

pub type PeerConnectedCallback = Box<dyn Fn(PeerConnectedEvent) + Send + Sync + 'static>;
pub type PeerDisconnectedCallback = Box<dyn Fn(PeerDisconnectedEvent) + Send + Sync + 'static>;
pub type NewPeakHeightCallback = Box<dyn Fn(NewPeakHeightEvent) + Send + Sync + 'static>;
pub type BlockReceivedCallback = Box<dyn Fn(BlockReceivedEvent) + Send + Sync + 'static>;

// Rust-native events and types are defined in crate::types and used throughout this module.

struct PeerWorkerParams {
    peer_connection: PeerConnection,
    peer_id: String,
    host: String,
    port: u16,
    disconnected_callback: Arc<RwLock<Option<PeerDisconnectedCallback>>>,
    inner: Arc<RwLock<ChiaPeerPoolInner>>,
    new_peak_callback: Arc<RwLock<Option<NewPeakHeightCallback>>>,
}

pub struct ChiaPeerPool {
    inner: Arc<RwLock<ChiaPeerPoolInner>>,
    request_sender: mpsc::Sender<PoolRequest>,
    connected_callback: Arc<RwLock<Option<PeerConnectedCallback>>>,
    disconnected_callback: Arc<RwLock<Option<PeerDisconnectedCallback>>>,
    new_peak_callback: Arc<RwLock<Option<NewPeakHeightCallback>>>,
    block_received_callback: Arc<RwLock<Option<BlockReceivedCallback>>>,
}

struct ChiaPeerPoolInner {
    peers: HashMap<String, PeerInfo>,
    peer_ids: Vec<String>,    // For round-robin
    round_robin_index: usize, // Track current position in round-robin
    highest_peak: Option<u32>,
}

struct PeerInfo {
    last_used: Instant,
    is_connected: bool,
    worker_tx: Option<mpsc::Sender<WorkerRequest>>,
    peak_height: Option<u32>,
}

enum PoolRequest {
    GetBlockByHeight {
        height: u64,
        response_tx: oneshot::Sender<Result<BlockReceivedEvent, ChiaError>>,
    },
}

enum WorkerRequest {
    GetBlock {
        height: u64,
        response_tx: oneshot::Sender<Result<FullBlock, ChiaError>>,
    },
    Shutdown,
}

// Connection state for each worker
struct WorkerConnection {
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    is_healthy: bool,
}

impl ChiaPeerPool {
    pub fn new() -> Self {
        let (request_sender, request_receiver) = mpsc::channel(100);
        let inner = Arc::new(RwLock::new(ChiaPeerPoolInner {
            peers: HashMap::new(),
            peer_ids: Vec::new(),
            round_robin_index: 0,
            highest_peak: None,
        }));

        let pool = Self {
            inner: inner.clone(),
            request_sender,
            connected_callback: Arc::new(RwLock::new(None)),
            disconnected_callback: Arc::new(RwLock::new(None)),
            new_peak_callback: Arc::new(RwLock::new(None)),
            block_received_callback: Arc::new(RwLock::new(None)),
        };

        // Start the request processor
        pool.start_request_processor(request_receiver);

        pool
    }

    pub fn set_event_callbacks(
        &self,
        connected_callback: PeerConnectedCallback,
        disconnected_callback: PeerDisconnectedCallback,
        new_peak_callback: NewPeakHeightCallback,
    ) {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            *self.connected_callback.write().await = Some(connected_callback);
            *self.disconnected_callback.write().await = Some(disconnected_callback);
            *self.new_peak_callback.write().await = Some(new_peak_callback);
        });
    }

    pub fn set_block_received_callback(&self, callback: BlockReceivedCallback) {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            *self.block_received_callback.write().await = Some(callback);
        });
    }

    pub async fn add_peer(
        &self,
        host: String,
        port: u16,
        network_id: String,
    ) -> Result<String, ChiaError> {
        info!("Adding peer {}:{} to pool", host, port);

        let peer_connection = PeerConnection::new(host.clone(), port, network_id);
        let peer_id = format!("{host}:{port}");

        // Establish connection upfront
        info!("Establishing initial connection for peer {}", peer_id);
        let ws_stream = timeout(
            Duration::from_millis(CONNECTION_TIMEOUT_MS),
            peer_connection.connect(),
        )
        .await
        .map_err(|_| ChiaError::Connection("Initial connection timeout".to_string()))?
        .map_err(|e| {
            error!("Initial connection failed for peer {}: {}", peer_id, e);
            e
        })?;

        // Perform handshake
        let mut ws_stream = ws_stream;
        timeout(
            Duration::from_millis(CONNECTION_TIMEOUT_MS),
            peer_connection.handshake(&mut ws_stream),
        )
        .await
        .map_err(|_| ChiaError::Connection("Handshake timeout".to_string()))?
        .map_err(|e| {
            error!("Handshake failed for peer {}: {}", peer_id, e);
            e
        })?;

        info!(
            "Successfully established initial connection for peer {}",
            peer_id
        );

        // Create worker for this peer with the established connection
        let (worker_tx, worker_rx) = mpsc::channel(10);
        let peer_conn_clone = peer_connection.clone();
        let peer_id_clone = peer_id.clone();
        let host_clone = host.clone();
        let disconnected_callback = self.disconnected_callback.clone();
        let new_peak_callback = self.new_peak_callback.clone();

        let inner_clone = self.inner.clone();

        // Pass the established connection to the worker
        let initial_connection = WorkerConnection {
            ws_stream,
            is_healthy: true,
        };

        tokio::spawn(async move {
            Self::peer_worker_with_connection(
                worker_rx,
                PeerWorkerParams {
                    peer_connection: peer_conn_clone,
                    peer_id: peer_id_clone,
                    host: host_clone,
                    port,
                    disconnected_callback,
                    inner: inner_clone,
                    new_peak_callback,
                },
                Some(initial_connection),
            )
            .await;
        });

        let mut guard = self.inner.write().await;
        guard.peers.insert(
            peer_id.clone(),
            PeerInfo {
                last_used: Instant::now()
                    .checked_sub(Duration::from_millis(RATE_LIMIT_MS))
                    .unwrap_or(Instant::now()),
                is_connected: true,
                worker_tx: Some(worker_tx),
                peak_height: None,
            },
        );
        guard.peer_ids.push(peer_id.clone());

        // Emit connected event
        if let Some(callback) = &*self.connected_callback.read().await {
            callback(PeerConnectedEvent {
                peer_id: peer_id.clone(),
                host: host.clone(),
                port: port as u32,
            });
        }

        drop(guard);
        Ok(peer_id)
    }

    pub async fn get_block_by_height(&self, height: u64) -> Result<BlockReceivedEvent, ChiaError> {
        self.get_block_by_height_with_failover(height, 3).await
    }

    async fn get_block_by_height_with_failover(
        &self,
        height: u64,
        max_retries: usize,
    ) -> Result<BlockReceivedEvent, ChiaError> {
        let mut attempted_peers = Vec::new();
        let mut last_error = ChiaError::Connection("No peers available".to_string());

        for attempt in 0..max_retries {
            // Find an available peer that hasn't been attempted yet
            let peer_id = {
                let guard = self.inner.read().await;
                let mut best_peer = None;
                let now = Instant::now();

                for peer_id in &guard.peer_ids {
                    if !attempted_peers.contains(peer_id) {
                        if let Some(peer_info) = guard.peers.get(peer_id) {
                            if peer_info.is_connected {
                                let time_since_last_use = now.duration_since(peer_info.last_used);

                                // Prefer peers that are immediately available
                                if time_since_last_use >= Duration::from_millis(RATE_LIMIT_MS) {
                                    best_peer = Some(peer_id.clone());
                                    break;
                                }

                                // Or take the first available peer if none are immediately ready
                                if best_peer.is_none() {
                                    best_peer = Some(peer_id.clone());
                                }
                            }
                        }
                    }
                }
                best_peer
            };

            if let Some(peer_id) = peer_id {
                attempted_peers.push(peer_id.clone());

                // Try to get the block from this peer
                let (response_tx, response_rx) = oneshot::channel();

                if let Err(e) = self
                    .request_sender
                    .send(PoolRequest::GetBlockByHeight {
                        height,
                        response_tx,
                    })
                    .await
                {
                    warn!("Failed to send request to peer {}: {}", peer_id, e);
                    last_error = ChiaError::Connection("Request channel closed".to_string());
                    continue;
                }

                match response_rx.await {
                    Ok(Ok(block_event)) => {
                        debug!(
                            "Successfully got block {} from peer {} on attempt {}",
                            height,
                            peer_id,
                            attempt + 1
                        );
                        return Ok(block_event);
                    }
                    Ok(Err(e)) => {
                        warn!(
                            "Peer {} failed to get block {}: {}. Trying next peer...",
                            peer_id, height, e
                        );
                        last_error = e;

                        // Check if this peer should be disconnected
                        match &last_error {
                            ChiaError::WebSocket(_) => {
                                info!(
                                    "Removing failed peer {} from pool due to WebSocket error",
                                    peer_id
                                );
                                let _ = self.remove_peer(peer_id).await;
                            }
                            ChiaError::Connection(msg)
                                if msg.contains("timeout")
                                    || msg.contains("closed")
                                    || msg.contains("failed") =>
                            {
                                info!(
                                    "Removing failed peer {} from pool due to connection error",
                                    peer_id
                                );
                                let _ = self.remove_peer(peer_id).await;
                            }
                            ChiaError::Protocol(msg) => {
                                // Protocol errors indicate the peer is misbehaving or incompatible
                                if msg.contains("Block request rejected")
                                    || msg.contains("Failed to parse")
                                    || msg.contains("Unexpected message")
                                    || msg.contains("Handshake failed")
                                {
                                    info!("Removing failed peer {} from pool due to protocol error: {}", peer_id, msg);
                                    let _ = self.remove_peer(peer_id).await;
                                } else {
                                    warn!("Minor protocol error for peer {}: {}. Not removing from pool", peer_id, msg);
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(_) => {
                        last_error = ChiaError::Connection("Response channel closed".to_string());
                    }
                }
            } else {
                // No more peers to try
                break;
            }
        }

        error!(
            "Failed to get block {} after trying {} peers",
            height,
            attempted_peers.len()
        );
        Err(last_error)
    }

    pub async fn remove_peer(&self, peer_id: String) -> Result<bool, ChiaError> {
        let mut guard = self.inner.write().await;

        if let Some(mut peer_info) = guard.peers.remove(&peer_id) {
            if let Some(worker_tx) = peer_info.worker_tx.take() {
                let _ = worker_tx.send(WorkerRequest::Shutdown).await;
            }

            guard.peer_ids.retain(|id| id != &peer_id);
            // Adjust round_robin_index if needed
            if guard.round_robin_index >= guard.peer_ids.len() && !guard.peer_ids.is_empty() {
                guard.round_robin_index = 0;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn shutdown(&self) -> Result<(), ChiaError> {
        let mut guard = self.inner.write().await;

        for (_, mut peer_info) in guard.peers.drain() {
            if let Some(worker_tx) = peer_info.worker_tx.take() {
                let _ = worker_tx.send(WorkerRequest::Shutdown).await;
            }
        }

        guard.peer_ids.clear();
        guard.round_robin_index = 0;
        Ok(())
    }

    pub async fn get_connected_peers(&self) -> Result<Vec<String>, ChiaError> {
        let guard = self.inner.read().await;
        Ok(guard.peer_ids.clone())
    }

    pub async fn get_highest_peak(&self) -> Option<u32> {
        self.inner.read().await.highest_peak
    }

    fn start_request_processor(&self, mut receiver: mpsc::Receiver<PoolRequest>) {
        let inner = self.inner.clone();
        let block_cb_arc = self.block_received_callback.clone();

        tokio::spawn(async move {
            let mut request_queue: VecDeque<PoolRequest> = VecDeque::new();

            loop {
                // Process incoming requests and queued requests more aggressively
                tokio::select! {
                    // Prioritize incoming requests
                    incoming_request = receiver.recv() => {
                        match incoming_request {
                            Some(request) => {
                                request_queue.push_back(request);
                            }
                            None => {
                                debug!("Request processor channel closed");
                                break;
                            }
                        }
                    }
                    // Process queued requests aggressively
                    _ = tokio::time::sleep(Duration::from_millis(10)) => {
                        // Try to process as many queued requests as possible
                        let mut processed_count = 0;
                        const MAX_BATCH_SIZE: usize = 10; // Process up to 10 requests per batch

                        while !request_queue.is_empty() && processed_count < MAX_BATCH_SIZE {
                            let mut guard = inner.write().await;

                            if let Some(request) = request_queue.front() {
                                match request {
                                    PoolRequest::GetBlockByHeight { .. } => {
                                        // Round-robin peer selection with 500ms cooldown
                                        let now = Instant::now();
                                        let mut selected_peer = None;
                                        let mut attempts = 0;
                                        let total_peers = guard.peer_ids.len();

                                        // Try to find an available peer using round-robin
                                        while attempts < total_peers {
                                            if !guard.peer_ids.is_empty() {
                                                let peer_id = &guard.peer_ids[guard.round_robin_index];

                                                if let Some(peer_info) = guard.peers.get(peer_id) {
                                                    if peer_info.is_connected {
                                                        let time_since_last_use = now.duration_since(peer_info.last_used);

                                                        // Check if this peer is available (past cooldown)
                                                        if time_since_last_use >= Duration::from_millis(RATE_LIMIT_MS) {
                                                            selected_peer = Some(peer_id.clone());
                                                            // Move to next peer for next request
                                                            guard.round_robin_index = (guard.round_robin_index + 1) % total_peers;
                                                            break;
                                                        }
                                                    }
                                                }

                                                // Move to next peer and try again
                                                guard.round_robin_index = (guard.round_robin_index + 1) % total_peers;
                                            }
                                            attempts += 1;
                                        }

                                        if let Some(peer_id) = selected_peer {
                                            if let Some(peer_info) = guard.peers.get(&peer_id) {
                                                let time_since_last_use = now.duration_since(peer_info.last_used);

                                                // Only proceed if peer is available (we already checked this in round-robin)
                                                if time_since_last_use >= Duration::from_millis(RATE_LIMIT_MS) {

                                                    if let Some(request) = request_queue.pop_front() {
                                                        if let Some(peer_info) = guard.peers.get_mut(&peer_id) {
                                                            peer_info.last_used = now;

                                                            match request {
                                                                PoolRequest::GetBlockByHeight {
                                                                    height,
                                                                    response_tx,
                                                                } => {
                                                                    if let Some(worker_tx) = &peer_info.worker_tx {
                                                                        let (worker_response_tx, worker_response_rx) =
                                                                            oneshot::channel();

                                                                        if worker_tx
                                                                            .send(WorkerRequest::GetBlock {
                                                                                height,
                                                                                response_tx: worker_response_tx,
                                                                            })
                                                                            .await
                                                                            .is_err()
                                                                        {
                                                                            error!("Failed to send request to worker for peer {}", peer_id);
                                                                            let _ = response_tx.send(Err(ChiaError::Connection(
                                                                                "Worker channel closed".to_string(),
                                                                            )));
                                                                            continue;
                                                                        }

                                                                        // Process response asynchronously for maximum throughput
                                                                        let peer_id_clone = peer_id.clone();
                                                                        let block_cb_arc2 = block_cb_arc.clone();
                                                                        tokio::spawn(async move {
                                                                            match worker_response_rx.await {
                                                                                Ok(Ok(full_block)) => {
                                                                                    // Parse the block
                                                                                    let parser = BlockParser::new();
                                                                                    match parser.parse_full_block(&full_block) {
                                                                                        Ok(parsed_block) => {
                                                                                            let block_event = Self::convert_parsed_block_to_external(
                                                                                                &parsed_block,
                                                                                                peer_id_clone,
                                                                                            );
                                                                                            // Fire block-received callback to subscribers if present
                                                                                            if let Some(cb) = &*block_cb_arc2.read().await {
                                                                                                cb(block_event.clone());
                                                                                            }
                                                                                            let _ = response_tx.send(Ok(block_event));
                                                                                        }
                                                                                        Err(e) => {
                                                                                            let _ = response_tx.send(Err(
                                                                                                ChiaError::Protocol(format!("Failed to parse block: {e}")),
                                                                                            ));
                                                                                        }
                                                                                    }
                                                                                }
                                                                                Ok(Err(e)) => {
                                                                                    let _ = response_tx.send(Err(e));
                                                                                }
                                                                                Err(_) => {
                                                                                    let _ = response_tx.send(Err(
                                                                                        ChiaError::Connection(
                                                                                            "Worker response channel closed".to_string(),
                                                                                        ),
                                                                                    ));
                                                                                }
                                                                            }
                                                                        });

                                                                        processed_count += 1;
                                                                    } else {
                                                                        error!("No worker available for peer {}", peer_id);
                                                                        let _ = response_tx.send(Err(ChiaError::Connection(
                                                                            "No worker available".to_string(),
                                                                        )));
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    // No peers available immediately, break and wait
                                                    break;
                                                }
                                            }
                                        } else {
                                            // No peers available at all
                                            break;
                                        }
                                    }
                                }
                            }

                            drop(guard); // Release lock between iterations
                        }
                    }
                }
            }
        });
    }

    async fn peer_worker_with_connection(
        mut receiver: mpsc::Receiver<WorkerRequest>,
        params: PeerWorkerParams,
        initial_connection: Option<WorkerConnection>,
    ) {
        info!(
            "Starting optimized worker for peer {} with{} initial connection",
            params.peer_id,
            if initial_connection.is_some() {
                ""
            } else {
                "out"
            }
        );

        let mut connection: Option<WorkerConnection> = initial_connection;
        let mut connection_failures = 0;
        let mut last_connection_attempt = Instant::now() - Duration::from_secs(60);
        const MAX_CONNECTION_FAILURES: u32 = 5;
        const CONNECTION_RETRY_DELAY: Duration = Duration::from_secs(10);

        while let Some(request) = receiver.recv().await {
            match request {
                WorkerRequest::GetBlock {
                    height,
                    response_tx,
                } => {
                    debug!(
                        "Worker {} processing block request for height {}",
                        params.peer_id, height
                    );

                    // Check if we have a healthy connection
                    let has_healthy_connection =
                        connection.is_some() && connection.as_ref().unwrap().is_healthy;

                    debug!(
                        "Connection state for {}: exists={}, healthy={}",
                        params.peer_id,
                        connection.is_some(),
                        connection.as_ref().map(|c| c.is_healthy).unwrap_or(false)
                    );

                    if !has_healthy_connection {
                        // Only try to reconnect if we haven't hit the failure limit and enough time has passed
                        let should_attempt_reconnection = connection_failures
                            < MAX_CONNECTION_FAILURES
                            && last_connection_attempt.elapsed() >= CONNECTION_RETRY_DELAY;

                        if should_attempt_reconnection || connection.is_none() {
                            info!("Re-establishing connection for peer {} (attempt #{}, last failure: {}s ago)", 
                                  params.peer_id, connection_failures + 1, last_connection_attempt.elapsed().as_secs());

                            last_connection_attempt = Instant::now();

                            match Self::establish_connection(&params).await {
                                Ok(new_connection) => {
                                    connection = Some(new_connection);
                                    connection_failures = 0;
                                    info!(
                                        "Successfully re-established connection for peer {}",
                                        params.peer_id
                                    );
                                }
                                Err(e) => {
                                    connection_failures += 1;
                                    error!("Failed to re-establish connection for {} (failure #{}): {}", 
                                           params.peer_id, connection_failures, e);

                                    // Check if this is a severe error that should trigger disconnection
                                    let should_disconnect = match &e {
                                        ChiaError::WebSocket(_) => true,
                                        ChiaError::Connection(msg)
                                            if msg.contains("timeout")
                                                || msg.contains("failed") =>
                                        {
                                            true
                                        }
                                        _ => false,
                                    };

                                    if should_disconnect
                                        && connection_failures >= MAX_CONNECTION_FAILURES
                                    {
                                        error!("Too many connection failures for peer {}, disconnecting", params.peer_id);
                                        Self::disconnect_peer_internal(&params.inner, &params)
                                            .await;
                                    }

                                    let _ = response_tx.send(Err(e));
                                    continue;
                                }
                            }
                        } else {
                            // Too many failures or too recent attempt, reject the request
                            let retry_in = CONNECTION_RETRY_DELAY
                                .saturating_sub(last_connection_attempt.elapsed());
                            warn!(
                                "Peer {} unavailable (too many failures), retry in {}s",
                                params.peer_id,
                                retry_in.as_secs()
                            );
                            let _ = response_tx.send(Err(ChiaError::Connection(format!(
                                "Peer temporarily unavailable, retry in {}s",
                                retry_in.as_secs()
                            ))));
                            continue;
                        }
                    }

                    // Use the connection for block request
                    if let Some(ref mut conn) = connection {
                        match Self::request_block_with_connection(height, conn, &params).await {
                            Ok(block) => {
                                debug!(
                                    "Successfully processed block {} via persistent connection",
                                    height
                                );
                                let _ = response_tx.send(Ok(block));
                            }
                            Err(e) => {
                                warn!(
                                    "Block request failed for {} via persistent connection: {}",
                                    params.peer_id, e
                                );

                                // Check if this is an error that indicates the peer should be disconnected
                                let should_disconnect = match &e {
                                    ChiaError::WebSocket(_) => {
                                        error!("WebSocket error detected for peer {}, marking for disconnection", params.peer_id);
                                        true
                                    }
                                    ChiaError::Connection(msg)
                                        if msg.contains("timeout")
                                            || msg.contains("closed")
                                            || msg.contains("failed") =>
                                    {
                                        error!("Connection error detected for peer {}, marking for disconnection", params.peer_id);
                                        true
                                    }
                                    ChiaError::Protocol(msg) => {
                                        // Protocol errors indicate the peer is misbehaving or incompatible
                                        if msg.contains("Block request rejected")
                                            || msg.contains("Failed to parse")
                                            || msg.contains("Unexpected message")
                                            || msg.contains("Handshake failed")
                                        {
                                            error!("Protocol error detected for peer {}: {}. Marking for disconnection", params.peer_id, msg);
                                            true
                                        } else {
                                            warn!("Minor protocol error for peer {}: {}. Not disconnecting", params.peer_id, msg);
                                            false
                                        }
                                    }
                                    _ => false,
                                };

                                if should_disconnect {
                                    // Mark peer as disconnected in the pool
                                    Self::disconnect_peer_internal(&params.inner, &params).await;

                                    // Mark connection as completely unusable
                                    conn.is_healthy = false;
                                    connection = None; // Clear the connection entirely

                                    // Increment connection failures to prevent immediate retry
                                    connection_failures = MAX_CONNECTION_FAILURES;
                                } else {
                                    // Just mark connection as unhealthy for reconnection attempt
                                    conn.is_healthy = false;
                                }

                                let _ = response_tx.send(Err(e));
                            }
                        }
                    } else {
                        let _ = response_tx.send(Err(ChiaError::Connection(
                            "No connection available".to_string(),
                        )));
                    }
                }
                WorkerRequest::Shutdown => {
                    info!("Shutting down optimized worker for peer {}", params.peer_id);
                    break;
                }
            }
        }

        // Emit disconnected event when worker shuts down
        if let Some(callback) = &*params.disconnected_callback.read().await {
            callback(PeerDisconnectedEvent {
                peer_id: params.peer_id,
                host: params.host,
                port: params.port as u32,
                message: Some("Worker shutdown".to_string()),
            });
        }
    }

    async fn establish_connection(
        params: &PeerWorkerParams,
    ) -> Result<WorkerConnection, ChiaError> {
        info!("Establishing connection for peer {}", params.peer_id);

        // Add timeout to connection establishment
        let connection_future = async {
            // Create connection
            let ws_stream = params.peer_connection.connect().await?;

            // Perform handshake
            let mut ws_stream = ws_stream;
            params.peer_connection.handshake(&mut ws_stream).await?;

            Ok::<WebSocketStream<MaybeTlsStream<TcpStream>>, ChiaError>(ws_stream)
        };

        let ws_stream = timeout(
            Duration::from_millis(CONNECTION_TIMEOUT_MS),
            connection_future,
        )
        .await
        .map_err(|_| ChiaError::Connection("Connection timeout".to_string()))?
        .map_err(|e| {
            error!("Connection failed for peer {}: {}", params.peer_id, e);
            e
        })?;

        info!(
            "Connection established and handshake completed for peer {}",
            params.peer_id
        );

        Ok(WorkerConnection {
            ws_stream,
            is_healthy: true,
        })
    }

    async fn request_block_with_connection(
        height: u64,
        connection: &mut WorkerConnection,
        params: &PeerWorkerParams,
    ) -> Result<FullBlock, ChiaError> {
        debug!(
            "Requesting block {} from peer {} (persistent connection)",
            height, params.peer_id
        );

        // Add timeout to block request
        let request_future = params
            .peer_connection
            .request_block_by_height(height, &mut connection.ws_stream);

        match timeout(Duration::from_millis(REQUEST_TIMEOUT_MS), request_future).await {
            Ok(Ok(block)) => {
                debug!(
                    "Successfully received block {} from peer {} via persistent connection",
                    height, params.peer_id
                );

                // Update peak height tracking
                Self::update_peak_height(height as u32, params).await;

                Ok(block)
            }
            Ok(Err(e)) => {
                warn!(
                    "Block request failed for height {} from peer {}: {}",
                    height, params.peer_id, e
                );
                connection.is_healthy = false; // Mark for reconnection
                Err(e)
            }
            Err(_) => {
                warn!(
                    "Block request timeout for height {} from peer {}",
                    height, params.peer_id
                );
                connection.is_healthy = false; // Mark for reconnection
                Err(ChiaError::Connection("Request timeout".to_string()))
            }
        }
    }

    async fn update_peak_height(block_height: u32, params: &PeerWorkerParams) {
        let mut guard = params.inner.write().await;
        if let Some(peer_info) = guard.peers.get_mut(&params.peer_id) {
            match peer_info.peak_height {
                Some(current_peak) => {
                    if block_height > current_peak {
                        peer_info.peak_height = Some(block_height);
                    }
                }
                None => {
                    peer_info.peak_height = Some(block_height);
                }
            }
        }

        // Update global highest peak
        let old_peak = guard.highest_peak;
        match guard.highest_peak {
            Some(current_highest) => {
                if block_height > current_highest {
                    guard.highest_peak = Some(block_height);
                    info!("New highest peak from block fetch: {}", block_height);
                    drop(guard);

                    // Emit new peak event
                    if let Some(callback) = &*params.new_peak_callback.read().await {
                        callback(NewPeakHeightEvent {
                            old_peak,
                            new_peak: block_height,
                            peer_id: params.peer_id.clone(),
                        });
                    }
                }
            }
            None => {
                guard.highest_peak = Some(block_height);
                info!("First peak height set: {}", block_height);
                drop(guard);

                // Emit new peak event
                if let Some(callback) = &*params.new_peak_callback.read().await {
                    callback(NewPeakHeightEvent {
                        old_peak,
                        new_peak: block_height,
                        peer_id: params.peer_id.clone(),
                    });
                }
            }
        }
    }

    // Helper function to convert internal types to external types
    fn convert_parsed_block_to_external(
        parsed_block: &ParsedBlock,
        peer_id: String,
    ) -> BlockReceivedEvent {
        BlockReceivedEvent {
            peer_id,
            height: parsed_block.height,
            weight: parsed_block.weight.clone(),
            header_hash: parsed_block.header_hash.clone(),
            timestamp: parsed_block.timestamp.unwrap_or(0),
            coin_additions: parsed_block
                .coin_additions
                .iter()
                .map(|coin| CoinRecord {
                    parent_coin_info: coin.parent_coin_info.clone(),
                    puzzle_hash: coin.puzzle_hash.clone(),
                    amount: coin.amount.to_string(),
                })
                .collect(),
            coin_removals: parsed_block
                .coin_removals
                .iter()
                .map(|coin| CoinRecord {
                    parent_coin_info: coin.parent_coin_info.clone(),
                    puzzle_hash: coin.puzzle_hash.clone(),
                    amount: coin.amount.to_string(),
                })
                .collect(),
            coin_spends: parsed_block
                .coin_spends
                .iter()
                .map(|spend| CoinSpend {
                    coin: CoinRecord {
                        parent_coin_info: spend.coin.parent_coin_info.clone(),
                        puzzle_hash: spend.coin.puzzle_hash.clone(),
                        amount: spend.coin.amount.to_string(),
                    },
                    puzzle_reveal: spend.puzzle_reveal.clone(),
                    solution: spend.solution.clone(),
                    offset: spend.offset,
                })
                .collect(),
            coin_creations: parsed_block
                .coin_creations
                .iter()
                .map(|coin| CoinRecord {
                    parent_coin_info: coin.parent_coin_info.clone(),
                    puzzle_hash: coin.puzzle_hash.clone(),
                    amount: coin.amount.to_string(),
                })
                .collect(),
            has_transactions_generator: parsed_block.has_transactions_generator,
            generator_size: parsed_block.generator_size.unwrap_or(0),
        }
    }

    async fn disconnect_peer_internal(
        inner: &Arc<RwLock<ChiaPeerPoolInner>>,
        params: &PeerWorkerParams,
    ) {
        let mut guard = inner.write().await;
        if let Some(mut peer_info) = guard.peers.remove(&params.peer_id) {
            if let Some(worker_tx) = peer_info.worker_tx.take() {
                let _ = worker_tx.send(WorkerRequest::Shutdown).await;
            }
            guard.peer_ids.retain(|id| id != &params.peer_id);
            // Adjust round_robin_index if needed
            if guard.round_robin_index >= guard.peer_ids.len() && !guard.peer_ids.is_empty() {
                guard.round_robin_index = 0;
            }
        }

        // Emit disconnected event
        if let Some(callback) = &*params.disconnected_callback.read().await {
            callback(PeerDisconnectedEvent {
                peer_id: params.peer_id.clone(),
                host: params.host.clone(),
                port: params.port as u32,
                message: Some("Peer disconnected due to WebSocket failure".to_string()),
            });
        }
    }
}
