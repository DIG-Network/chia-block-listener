use chia_block_listener::error::ChiaError;
use chia_block_listener::peer::PeerConnection;
use chia_generator_parser::{types::ParsedBlock, BlockParser};

use napi::{
    bindgen_prelude::*,
    threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
    JsFunction,
};
use napi_derive::napi;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, RwLock};
use tracing::{error, info};

#[allow(dead_code)]
pub const EVENT_BLOCK_RECEIVED: &str = "blockReceived";
#[allow(dead_code)]
pub const EVENT_PEER_CONNECTED: &str = "peerConnected";
#[allow(dead_code)]
pub const EVENT_PEER_DISCONNECTED: &str = "peerDisconnected";

// Export event types for TypeScript
#[napi(object)]
pub struct EventTypes {
    pub block_received: String,
    pub peer_connected: String,
    pub peer_disconnected: String,
}

#[napi]
#[allow(dead_code)]
pub fn get_event_types() -> EventTypes {
    EventTypes {
        block_received: EVENT_BLOCK_RECEIVED.to_string(),
        peer_connected: EVENT_PEER_CONNECTED.to_string(),
        peer_disconnected: EVENT_PEER_DISCONNECTED.to_string(),
    }
}

#[napi]
pub struct ChiaBlockListener {
    inner: Arc<RwLock<ChiaBlockListenerInner>>,
}

struct ChiaBlockListenerInner {
    peers: HashMap<String, PeerConnectionInfo>,
    block_listeners: Vec<ThreadsafeFunction<BlockReceivedEvent, ErrorStrategy::Fatal>>,
    peer_connected_listeners: Vec<ThreadsafeFunction<PeerConnectedEvent, ErrorStrategy::Fatal>>,
    peer_disconnected_listeners:
        Vec<ThreadsafeFunction<PeerDisconnectedEvent, ErrorStrategy::Fatal>>,
    block_sender: mpsc::Sender<ParsedBlockEvent>,
    event_sender: mpsc::Sender<PeerEvent>,
}

struct PeerConnectionInfo {
    connection: PeerConnection,
    disconnect_tx: Option<oneshot::Sender<()>>,
    is_connected: bool,
}

#[derive(Clone)]
struct ParsedBlockEvent {
    peer_id: String,
    block: ParsedBlock,
}

#[derive(Clone)]
struct PeerEvent {
    event_type: PeerEventType,
    peer_id: String,
    host: String,
    port: u16,
    message: Option<String>,
}

#[derive(Clone)]
enum PeerEventType {
    Connected,
    Disconnected,
    Error,
}

// Export for TypeScript
#[napi(object)]
#[derive(Clone)]
pub struct PeerConnectedEvent {
    #[napi(js_name = "peerId")]
    pub peer_id: String,
    pub host: String,
    pub port: u32,
}

// Export for TypeScript
#[napi(object)]
#[derive(Clone)]
pub struct PeerDisconnectedEvent {
    #[napi(js_name = "peerId")]
    pub peer_id: String,
    pub host: String,
    pub port: u32,
    pub message: Option<String>,
}

// Event struct for block received callbacks
#[napi(object)]
#[derive(Clone)]
pub struct BlockReceivedEvent {
    #[napi(js_name = "peerId")]
    pub peer_id: String,
    pub height: u32,
    pub weight: String,
    #[napi(js_name = "headerHash")]
    pub header_hash: String,
    pub timestamp: u32,
    #[napi(js_name = "coinAdditions")]
    pub coin_additions: Vec<CoinRecord>,
    #[napi(js_name = "coinRemovals")]
    pub coin_removals: Vec<CoinRecord>,
    #[napi(js_name = "coinSpends")]
    pub coin_spends: Vec<CoinSpend>,
    #[napi(js_name = "coinCreations")]
    pub coin_creations: Vec<CoinRecord>,
    #[napi(js_name = "hasTransactionsGenerator")]
    pub has_transactions_generator: bool,
    #[napi(js_name = "generatorSize")]
    pub generator_size: u32,
}

#[napi(object)]
#[derive(Clone)]
pub struct CoinRecord {
    #[napi(js_name = "parentCoinInfo")]
    pub parent_coin_info: String,
    #[napi(js_name = "puzzleHash")]
    pub puzzle_hash: String,
    pub amount: String,
}

#[napi(object)]
#[derive(Clone)]
pub struct CoinSpend {
    pub coin: CoinRecord,
    #[napi(js_name = "puzzleReveal")]
    pub puzzle_reveal: String,
    pub solution: String,
    pub offset: u32,
}

#[napi]
impl ChiaBlockListener {
    #[napi(constructor)]
    pub fn new() -> Self {
        let (block_sender, block_receiver) = mpsc::channel(100);
        let (event_sender, event_receiver) = mpsc::channel(100);

        let inner = Arc::new(RwLock::new(ChiaBlockListenerInner {
            peers: HashMap::new(),
            block_listeners: Vec::new(),
            peer_connected_listeners: Vec::new(),
            peer_disconnected_listeners: Vec::new(),
            block_sender,
            event_sender,
        }));

        let inner_clone = inner.clone();
        tokio::spawn(async move {
            Self::event_loop(inner_clone, block_receiver, event_receiver).await;
        });

        Self { inner }
    }

    async fn event_loop(
        inner: Arc<RwLock<ChiaBlockListenerInner>>,
        mut block_receiver: mpsc::Receiver<ParsedBlockEvent>,
        mut event_receiver: mpsc::Receiver<PeerEvent>,
    ) {
        loop {
            tokio::select! {
                Some(block_event) = block_receiver.recv() => {
                    // Convert ParsedBlock to external Block format
                    let block_received_event = ChiaBlockListener::convert_parsed_block_to_external(&block_event.block, block_event.peer_id);

                    let listeners = {
                        let guard = inner.read().await;
                        guard.block_listeners.clone()
                    };
                    for listener in listeners {
                        listener.call(block_received_event.clone(), ThreadsafeFunctionCallMode::NonBlocking);
                    }
                }
                Some(peer_event) = event_receiver.recv() => {
                    match peer_event.event_type {
                        PeerEventType::Connected => {
                            let connected_event = PeerConnectedEvent {
                                peer_id: peer_event.peer_id,
                                host: peer_event.host,
                                port: peer_event.port as u32,
                            };
                            let listeners = {
                                let guard = inner.read().await;
                                guard.peer_connected_listeners.clone()
                            };
                            for listener in listeners {
                                listener.call(connected_event.clone(), ThreadsafeFunctionCallMode::NonBlocking);
                            }
                        }
                        PeerEventType::Disconnected => {
                            let disconnected_event = PeerDisconnectedEvent {
                                peer_id: peer_event.peer_id,
                                host: peer_event.host,
                                port: peer_event.port as u32,
                                message: peer_event.message,
                            };
                            let listeners = {
                                let guard = inner.read().await;
                                guard.peer_disconnected_listeners.clone()
                            };
                            for listener in listeners {
                                listener.call(disconnected_event.clone(), ThreadsafeFunctionCallMode::NonBlocking);
                            }
                        }
                        PeerEventType::Error => {
                            // Handle errors by treating them as disconnections
                            let disconnected_event = PeerDisconnectedEvent {
                                peer_id: peer_event.peer_id,
                                host: peer_event.host,
                                port: peer_event.port as u32,
                                message: peer_event.message,
                            };
                            let listeners = {
                                let guard = inner.read().await;
                                guard.peer_disconnected_listeners.clone()
                            };
                            for listener in listeners {
                                listener.call(disconnected_event.clone(), ThreadsafeFunctionCallMode::NonBlocking);
                            }
                        }
                    }
                }
                else => break,
            }
        }
    }

    #[napi]
    pub fn add_peer(&self, host: String, port: u16, network_id: String) -> Result<String> {
        let peer = PeerConnection::new(host.clone(), port, network_id);

        let rt = tokio::runtime::Handle::current();
        let inner = self.inner.clone();

        let peer_id = rt.block_on(async {
            let mut guard = inner.write().await;
            let peer_id = host.clone();

            guard.peers.insert(
                peer_id.clone(),
                PeerConnectionInfo {
                    connection: peer.clone(),
                    disconnect_tx: None,
                    is_connected: false,
                },
            );

            peer_id
        });

        info!("Added peer {} with ID {}", host, peer_id);

        // Automatically start connection for this peer
        self.start_peer_connection(peer_id.clone(), peer);

        Ok(peer_id)
    }

    #[napi]
    pub fn disconnect_peer(&self, peer_id: String) -> Result<bool> {
        let rt = tokio::runtime::Handle::current();
        let inner = self.inner.clone();

        let disconnected = rt.block_on(async {
            let mut guard = inner.write().await;
            if let Some(mut peer_info) = guard.peers.remove(&peer_id) {
                if let Some(disconnect_tx) = peer_info.disconnect_tx.take() {
                    let _ = disconnect_tx.send(());
                }
                true
            } else {
                false
            }
        });

        Ok(disconnected)
    }

    #[napi]
    pub fn disconnect_all_peers(&self) -> Result<()> {
        let rt = tokio::runtime::Handle::current();
        let inner = self.inner.clone();

        rt.block_on(async {
            let mut guard = inner.write().await;

            let peer_ids: Vec<String> = guard.peers.keys().cloned().collect();
            for peer_id in peer_ids {
                if let Some(mut peer_info) = guard.peers.remove(&peer_id) {
                    if let Some(disconnect_tx) = peer_info.disconnect_tx.take() {
                        let _ = disconnect_tx.send(());
                    }
                }
            }
        });

        info!("Disconnected all peers");
        Ok(())
    }

    #[napi]
    pub fn get_connected_peers(&self) -> Result<Vec<String>> {
        let rt = tokio::runtime::Handle::current();
        let inner = self.inner.clone();

        Ok(rt.block_on(async {
            let guard = inner.read().await;
            guard.peers.keys().cloned().collect()
        }))
    }

    #[napi]
    pub fn on(&self, event: String, callback: JsFunction) -> Result<()> {
        let rt = tokio::runtime::Handle::current();
        let inner = self.inner.clone();

        match event.as_str() {
            "blockReceived" => {
                let tsfn = callback.create_threadsafe_function(0, |ctx| {
                    let event: &BlockReceivedEvent = &ctx.value;
                    let mut obj = ctx.env.create_object()?;

                    obj.set_named_property("peerId", ctx.env.create_string(&event.peer_id)?)?;
                    obj.set_named_property("height", ctx.env.create_uint32(event.height)?)?;
                    obj.set_named_property("weight", ctx.env.create_string(&event.weight)?)?;
                    obj.set_named_property(
                        "headerHash",
                        ctx.env.create_string(&event.header_hash)?,
                    )?;
                    obj.set_named_property("timestamp", ctx.env.create_uint32(event.timestamp)?)?;

                    // Coin additions array
                    let mut additions_array = ctx
                        .env
                        .create_array_with_length(event.coin_additions.len())?;
                    for (i, coin) in event.coin_additions.iter().enumerate() {
                        let mut coin_obj = ctx.env.create_object()?;
                        coin_obj.set_named_property(
                            "parentCoinInfo",
                            ctx.env.create_string(&coin.parent_coin_info)?,
                        )?;
                        coin_obj.set_named_property(
                            "puzzleHash",
                            ctx.env.create_string(&coin.puzzle_hash)?,
                        )?;
                        coin_obj.set_named_property(
                            "amount",
                            ctx.env.create_string(&coin.amount.to_string())?,
                        )?;
                        additions_array.set_element(i as u32, coin_obj)?;
                    }
                    obj.set_named_property("coinAdditions", additions_array)?;

                    // Coin removals array
                    let mut removals_array = ctx
                        .env
                        .create_array_with_length(event.coin_removals.len())?;
                    for (i, coin) in event.coin_removals.iter().enumerate() {
                        let mut coin_obj = ctx.env.create_object()?;
                        coin_obj.set_named_property(
                            "parentCoinInfo",
                            ctx.env.create_string(&coin.parent_coin_info)?,
                        )?;
                        coin_obj.set_named_property(
                            "puzzleHash",
                            ctx.env.create_string(&coin.puzzle_hash)?,
                        )?;
                        coin_obj.set_named_property(
                            "amount",
                            ctx.env.create_string(&coin.amount.to_string())?,
                        )?;
                        removals_array.set_element(i as u32, coin_obj)?;
                    }
                    obj.set_named_property("coinRemovals", removals_array)?;

                    // Coin spends array
                    let mut spends_array =
                        ctx.env.create_array_with_length(event.coin_spends.len())?;
                    for (i, spend) in event.coin_spends.iter().enumerate() {
                        let mut spend_obj = ctx.env.create_object()?;

                        // Create coin object
                        let mut coin_obj = ctx.env.create_object()?;
                        coin_obj.set_named_property(
                            "parentCoinInfo",
                            ctx.env.create_string(&spend.coin.parent_coin_info)?,
                        )?;
                        coin_obj.set_named_property(
                            "puzzleHash",
                            ctx.env.create_string(&spend.coin.puzzle_hash)?,
                        )?;
                        coin_obj.set_named_property(
                            "amount",
                            ctx.env.create_string(&spend.coin.amount)?,
                        )?;
                        spend_obj.set_named_property("coin", coin_obj)?;

                        spend_obj.set_named_property(
                            "puzzleReveal",
                            ctx.env.create_string(&spend.puzzle_reveal)?,
                        )?;
                        spend_obj.set_named_property(
                            "solution",
                            ctx.env.create_string(&spend.solution)?,
                        )?;

                        spend_obj
                            .set_named_property("offset", ctx.env.create_uint32(spend.offset)?)?;

                        spends_array.set_element(i as u32, spend_obj)?;
                    }
                    obj.set_named_property("coinSpends", spends_array)?;

                    // Coin creations array
                    let mut creations_array = ctx
                        .env
                        .create_array_with_length(event.coin_creations.len())?;
                    for (i, coin) in event.coin_creations.iter().enumerate() {
                        let mut coin_obj = ctx.env.create_object()?;
                        coin_obj.set_named_property(
                            "parentCoinInfo",
                            ctx.env.create_string(&coin.parent_coin_info)?,
                        )?;
                        coin_obj.set_named_property(
                            "puzzleHash",
                            ctx.env.create_string(&coin.puzzle_hash)?,
                        )?;
                        coin_obj
                            .set_named_property("amount", ctx.env.create_string(&coin.amount)?)?;
                        creations_array.set_element(i as u32, coin_obj)?;
                    }
                    obj.set_named_property("coinCreations", creations_array)?;

                    obj.set_named_property(
                        "hasTransactionsGenerator",
                        ctx.env.get_boolean(event.has_transactions_generator)?,
                    )?;
                    obj.set_named_property(
                        "generatorSize",
                        ctx.env.create_uint32(event.generator_size)?,
                    )?;

                    Ok(vec![obj])
                })?;

                rt.block_on(async {
                    let mut guard = inner.write().await;
                    guard.block_listeners.push(tsfn);
                });
            }
            "peerConnected" => {
                let tsfn = callback.create_threadsafe_function(0, |ctx| {
                    let event: &PeerConnectedEvent = &ctx.value;
                    let mut obj = ctx.env.create_object()?;
                    obj.set_named_property("peerId", ctx.env.create_string(&event.peer_id)?)?;
                    obj.set_named_property("host", ctx.env.create_string(&event.host)?)?;
                    obj.set_named_property("port", ctx.env.create_uint32(event.port)?)?;
                    Ok(vec![obj])
                })?;

                rt.block_on(async {
                    let mut guard = inner.write().await;
                    guard.peer_connected_listeners.push(tsfn);
                });
            }
            "peerDisconnected" => {
                let tsfn = callback.create_threadsafe_function(0, |ctx| {
                    let event: &PeerDisconnectedEvent = &ctx.value;
                    let mut obj = ctx.env.create_object()?;
                    obj.set_named_property("peerId", ctx.env.create_string(&event.peer_id)?)?;
                    obj.set_named_property("host", ctx.env.create_string(&event.host)?)?;
                    obj.set_named_property("port", ctx.env.create_uint32(event.port)?)?;
                    if let Some(msg) = &event.message {
                        obj.set_named_property("message", ctx.env.create_string(msg)?)?;
                    }
                    Ok(vec![obj])
                })?;

                rt.block_on(async {
                    let mut guard = inner.write().await;
                    guard.peer_disconnected_listeners.push(tsfn);
                });
            }
            _ => {
                return Err(Error::new(
                    Status::InvalidArg,
                    format!("Unknown event type: {event}"),
                ))
            }
        }

        Ok(())
    }

    #[napi]
    pub fn off(&self, event: String, _callback: JsFunction) -> Result<()> {
        let rt = tokio::runtime::Handle::current();
        let inner = self.inner.clone();

        rt.block_on(async {
            let mut guard = inner.write().await;

            // For simplicity, we'll clear all listeners of the given type
            // In a full implementation, you'd want to match specific callbacks
            match event.as_str() {
                "blockReceived" => guard.block_listeners.clear(),
                "peerConnected" => guard.peer_connected_listeners.clear(),
                "peerDisconnected" => guard.peer_disconnected_listeners.clear(),
                _ => {
                    return Err(Error::new(
                        Status::InvalidArg,
                        format!("Unknown event type: {event}"),
                    ))
                }
            }

            Ok(())
        })
    }

    fn start_peer_connection(&self, peer_id: String, peer: PeerConnection) {
        let inner = self.inner.clone();

        tokio::spawn(async move {
            let (disconnect_tx, disconnect_rx) = oneshot::channel();

            // Store disconnect channel
            {
                let mut guard = inner.write().await;
                if let Some(peer_info) = guard.peers.get_mut(&peer_id) {
                    peer_info.disconnect_tx = Some(disconnect_tx);
                }
            }

            let host = peer.host().to_string();
            let port = peer.port();

            match peer.connect().await {
                Ok(mut ws_stream) => {
                    info!("Connected to peer {} (ID: {})", host, &peer_id);

                    if let Err(e) = peer.handshake(&mut ws_stream).await {
                        error!(
                            "Handshake failed for peer {} (ID: {}): {}",
                            host, &peer_id, e
                        );
                        let guard = inner.read().await;
                        let _ = guard
                            .event_sender
                            .send(PeerEvent {
                                event_type: PeerEventType::Error,
                                peer_id: peer_id.clone(),
                                host: host.clone(),
                                port,
                                message: Some(format!("Handshake failed: {e}")),
                            })
                            .await;
                        return;
                    }

                    // Send connected event after successful handshake
                    {
                        let guard = inner.read().await;
                        let _ = guard
                            .event_sender
                            .send(PeerEvent {
                                event_type: PeerEventType::Connected,
                                peer_id: peer_id.clone(),
                                host: host.clone(),
                                port,
                                message: None,
                            })
                            .await;
                    }

                    // Mark peer as connected
                    {
                        let mut guard = inner.write().await;
                        if let Some(peer_info) = guard.peers.get_mut(&peer_id) {
                            peer_info.is_connected = true;
                        }
                    }

                    // Create block sender for this peer
                    let block_sender = {
                        let guard = inner.read().await;
                        guard.block_sender.clone()
                    };

                    let (block_tx, mut block_rx) = mpsc::channel(100);

                    // Clone peer_id for the block listener task
                    let peer_id_for_listener = peer_id.clone();
                    let peer_id_for_blocks = peer_id.clone();

                    // Spawn block listener
                    let inner_for_listener = inner.clone();
                    let host_for_listener = host.clone();
                    tokio::spawn(async move {
                        tokio::select! {
                            result = PeerConnection::listen_for_blocks(ws_stream, block_tx) => {
                                match result {
                                    Ok(_) => info!("Peer {} (ID: {}) disconnected normally", host_for_listener, &peer_id_for_listener),
                                    Err(e) => {
                                        error!("Error listening to peer {} (ID: {}): {}", host_for_listener, &peer_id_for_listener, e);
                                        let guard = inner_for_listener.read().await;
                                        let _ = guard.event_sender.send(PeerEvent {
                                            event_type: PeerEventType::Error,
                                            peer_id: peer_id_for_listener.clone(),
                                            host: host_for_listener.clone(),
                                            port,
                                            message: Some(e.to_string()),
                                        }).await;
                                    }
                                }
                            }
                            _ = disconnect_rx => {
                                info!("Peer {} (ID: {}) disconnected by request", host_for_listener, &peer_id_for_listener);
                            }
                        }

                        // Send disconnected event
                        let guard = inner_for_listener.read().await;
                        let _ = guard
                            .event_sender
                            .send(PeerEvent {
                                event_type: PeerEventType::Disconnected,
                                peer_id: peer_id_for_listener.clone(),
                                host: host_for_listener.clone(),
                                port,
                                message: Some("Connection closed".to_string()),
                            })
                            .await;
                        drop(guard);

                        // Mark peer as disconnected
                        let mut guard = inner_for_listener.write().await;
                        if let Some(peer_info) = guard.peers.get_mut(&peer_id_for_listener) {
                            peer_info.is_connected = false;
                        }
                    });

                    // Forward parsed blocks with peer ID
                    while let Some(parsed_block) = block_rx.recv().await {
                        info!(
                            "Received parsed block {} with {} coin additions, {} coin removals, {} coin spends, {} coin creations",
                            parsed_block.height,
                            parsed_block.coin_additions.len(),
                            parsed_block.coin_removals.len(),
                            parsed_block.coin_spends.len(),
                            parsed_block.coin_creations.len()
                        );

                        let _ = block_sender
                            .send(ParsedBlockEvent {
                                peer_id: peer_id_for_blocks.clone(),
                                block: parsed_block,
                            })
                            .await;
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to connect to peer {} (ID: {}): {}",
                        host, &peer_id, e
                    );
                    let guard = inner.read().await;
                    let _ = guard
                        .event_sender
                        .send(PeerEvent {
                            event_type: PeerEventType::Error,
                            peer_id: peer_id.clone(),
                            host,
                            port,
                            message: Some(format!("Connection failed: {e}")),
                        })
                        .await;
                }
            }
        });
    }

    // Helper function to convert internal types to external types
    pub fn convert_parsed_block_to_external(
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

    #[napi]
    pub fn get_block_by_height(&self, peer_id: String, height: u32) -> Result<BlockReceivedEvent> {
        let rt = tokio::runtime::Handle::current();
        let inner = self.inner.clone();

        let block_result = rt.block_on(async {
            let guard = inner.read().await;

            if let Some(peer_info) = guard.peers.get(&peer_id) {
                let peer = peer_info.connection.clone();
                drop(guard); // Release the lock before connecting

                // Create a new connection for this request
                match peer.connect().await {
                    Ok(mut ws_stream) => {
                        // Perform handshake
                        if let Err(e) = peer.handshake(&mut ws_stream).await {
                            return Err(ChiaError::Protocol(format!("Handshake failed: {e}")));
                        }

                        // Request the block
                        peer.request_block_by_height(height as u64, &mut ws_stream)
                            .await
                    }
                    Err(e) => Err(e),
                }
            } else {
                Err(ChiaError::Connection(format!("Peer {peer_id} not found")))
            }
        });

        match block_result {
            Ok(block) => {
                // Parse the block using chia-generator-parser
                let parser = BlockParser::new();
                let parsed_block = parser.parse_full_block(&block).map_err(|e| {
                    Error::new(
                        Status::GenericFailure,
                        format!("Failed to parse block: {e}"),
                    )
                })?;

                // Convert to Block type
                Ok(Self::convert_parsed_block_to_external(
                    &parsed_block,
                    peer_id.clone(),
                ))
            }
            Err(e) => Err(Error::new(
                Status::GenericFailure,
                format!("Failed to get block: {e}"),
            )),
        }
    }

    #[napi]
    pub fn get_blocks_range(
        &self,
        peer_id: String,
        start_height: u32,
        end_height: u32,
    ) -> Result<Vec<BlockReceivedEvent>> {
        if start_height > end_height {
            return Err(Error::new(
                Status::InvalidArg,
                "start_height must be <= end_height",
            ));
        }

        let mut blocks = Vec::new();

        for height in start_height..=end_height {
            match self.get_block_by_height(peer_id.clone(), height) {
                Ok(block) => blocks.push(block),
                Err(e) => {
                    // Log error but continue with other blocks
                    error!("Failed to get block at height {}: {}", height, e);
                }
            }
        }

        Ok(blocks)
    }
}

impl Default for ChiaBlockListener {
    fn default() -> Self {
        Self::new()
    }
}
