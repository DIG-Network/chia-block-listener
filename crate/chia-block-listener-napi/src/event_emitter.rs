use chia_block_listener::types::{Event as CoreEvent, BlockReceivedEvent as CoreBlockEvent};
use chia_block_listener::BlockListener;
use napi::{
    bindgen_prelude::*,
    threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
    JsFunction,
};
use napi_derive::napi;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

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
    listener: Arc<BlockListener>,
    block_listeners: Vec<ThreadsafeFunction<BlockReceivedEvent, ErrorStrategy::Fatal>>,
    peer_connected_listeners: Vec<ThreadsafeFunction<PeerConnectedEvent, ErrorStrategy::Fatal>>,
    peer_disconnected_listeners:
        Vec<ThreadsafeFunction<PeerDisconnectedEvent, ErrorStrategy::Fatal>>,
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
        let core_listener = Arc::new(BlockListener::new(chia_block_listener::types::BlockListenerConfig::default()).expect("listener init"));

        let inner = Arc::new(RwLock::new(ChiaBlockListenerInner {
            listener: core_listener.clone(),
            block_listeners: Vec::new(),
            peer_connected_listeners: Vec::new(),
            peer_disconnected_listeners: Vec::new(),
        }));

        // Subscribe to core events and dispatch to JS listeners
        let mut rx = core_listener.subscribe();
        let inner_for_task = inner.clone();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(ev) => {
                        match ev {
                            CoreEvent::BlockReceived(core_block) => {
                                let js_block = convert_core_block_to_js(core_block);
                                let listeners = {
                                    let guard = inner_for_task.read().await;
                                    guard.block_listeners.clone()
                                };
                                for listener in listeners {
                                    listener.call(js_block.clone(), ThreadsafeFunctionCallMode::NonBlocking);
                                }
                            }
                            CoreEvent::PeerConnected(pc) => {
                                let js = PeerConnectedEvent { peer_id: pc.peer_id, host: pc.host, port: pc.port };
                                let listeners = {
                                    let guard = inner_for_task.read().await;
                                    guard.peer_connected_listeners.clone()
                                };
                                for l in listeners { l.call(js.clone(), ThreadsafeFunctionCallMode::NonBlocking); }
                            }
                            CoreEvent::PeerDisconnected(pd) => {
                                let js = PeerDisconnectedEvent { peer_id: pd.peer_id, host: pd.host, port: pd.port, message: pd.message };
                                let listeners = {
                                    let guard = inner_for_task.read().await;
                                    guard.peer_disconnected_listeners.clone()
                                };
                                for l in listeners { l.call(js.clone(), ThreadsafeFunctionCallMode::NonBlocking); }
                            }
                            CoreEvent::NewPeakHeight(_np) => {
                                // This adapter does not expose newPeakHeight on this class; handled in ChiaPeerPool if needed
                            }
                        }
                    }
                    Err(_lagged) => {
                        // Receiver lagged or closed; best-effort delivery
                        continue;
                    }
                }
            }
        });

        Self { inner }
    }

    #[napi]
    pub fn add_peer(&self, host: String, port: u16, network_id: String) -> Result<String> {
        let rt = tokio::runtime::Handle::current();
        let inner = self.inner.clone();
        let peer_id = rt.block_on(async move {
            let guard = inner.read().await;
            guard
                .listener
                .add_peer(host, port, network_id)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to add peer: {e}")))
        })?;
        info!("Added peer with ID {}", peer_id);
        Ok(peer_id)
    }

    #[napi]
    pub fn disconnect_peer(&self, peer_id: String) -> Result<bool> {
        let rt = tokio::runtime::Handle::current();
        let inner = self.inner.clone();
        rt.block_on(async move {
            let guard = inner.read().await;
            guard
                .listener
                .remove_peer(peer_id)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to disconnect peer: {e}")))
        })
    }

    #[napi]
    pub fn disconnect_all_peers(&self) -> Result<()> {
        let rt = tokio::runtime::Handle::current();
        let inner = self.inner.clone();
        rt.block_on(async move {
            let guard = inner.read().await;
            let peers = guard
                .listener
                .get_connected_peers()
                .await
                .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
            drop(guard);

            // Remove peers sequentially
            for pid in peers {
                let guard2 = inner.read().await;
                let _ = guard2.listener.remove_peer(pid).await;
            }
            Ok(())
        })
    }

    #[napi]
    pub fn get_connected_peers(&self) -> Result<Vec<String>> {
        let rt = tokio::runtime::Handle::current();
        let inner = self.inner.clone();
        rt.block_on(async move {
            let guard = inner.read().await;
            guard
                .listener
                .get_connected_peers()
                .await
                .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
        })
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

    // Helper function lives below: convert_core_block_to_js

    #[napi]
    pub fn get_block_by_height(&self, _peer_id: String, height: u32) -> Result<BlockReceivedEvent> {
        let rt = tokio::runtime::Handle::current();
        let inner = self.inner.clone();
        rt.block_on(async move {
            let guard = inner.read().await;
            let core_block: CoreBlockEvent = guard
                .listener
                .get_block_by_height(height as u64)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get block: {e}")))?;
            Ok(convert_core_block_to_js(core_block))
        })
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
                    // using napi::Error means we can't easily log here without a logger; silently continue
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

// Single source of truth for converting core block into JS DTO used by this adapter
fn convert_core_block_to_js(core: CoreBlockEvent) -> BlockReceivedEvent {
    BlockReceivedEvent {
        peer_id: core.peer_id,
        height: core.height,
        weight: core.weight,
        header_hash: core.header_hash,
        timestamp: core.timestamp,
        coin_additions: core
            .coin_additions
            .into_iter()
            .map(|c| CoinRecord {
                parent_coin_info: c.parent_coin_info,
                puzzle_hash: c.puzzle_hash,
                amount: c.amount,
            })
            .collect(),
        coin_removals: core
            .coin_removals
            .into_iter()
            .map(|c| CoinRecord {
                parent_coin_info: c.parent_coin_info,
                puzzle_hash: c.puzzle_hash,
                amount: c.amount,
            })
            .collect(),
        coin_spends: core
            .coin_spends
            .into_iter()
            .map(|s| CoinSpend {
                coin: CoinRecord {
                    parent_coin_info: s.coin.parent_coin_info,
                    puzzle_hash: s.coin.puzzle_hash,
                    amount: s.coin.amount,
                },
                puzzle_reveal: s.puzzle_reveal,
                solution: s.solution,
                offset: s.offset,
            })
            .collect(),
        coin_creations: core
            .coin_creations
            .into_iter()
            .map(|c| CoinRecord {
                parent_coin_info: c.parent_coin_info,
                puzzle_hash: c.puzzle_hash,
                amount: c.amount,
            })
            .collect(),
        has_transactions_generator: core.has_transactions_generator,
        generator_size: core.generator_size,
    }
}
