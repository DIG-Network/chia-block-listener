use crate::event_emitter::BlockReceivedEvent;
use chia_block_listener::types::{
    BlockReceivedEvent as CoreBlockReceivedEvent, NewPeakHeightEvent, PeerConnectedEvent,
    PeerDisconnectedEvent,
};
use chia_block_listener::{BlockListener, BlockListenerConfig};
use napi::bindgen_prelude::*;
use napi::{
    threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
    JsFunction,
};
use napi_derive::napi;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[napi]
pub struct ChiaPeerPool {
    listener: Arc<BlockListener>,
    listeners: Arc<RwLock<EventListeners>>,
}

struct EventListeners {
    peer_connected_listeners: Vec<ThreadsafeFunction<PeerConnectedEvent, ErrorStrategy::Fatal>>,
    peer_disconnected_listeners:
        Vec<ThreadsafeFunction<PeerDisconnectedEvent, ErrorStrategy::Fatal>>,
    new_peak_height_listeners: Vec<ThreadsafeFunction<NewPeakHeightEvent, ErrorStrategy::Fatal>>,
}

#[napi]
impl ChiaPeerPool {
    #[napi(constructor)]
    pub fn new() -> Self {
        info!("Creating new ChiaPeerPool (N-API adapter over core Listener)");
        let listeners = Arc::new(RwLock::new(EventListeners {
            peer_connected_listeners: Vec::new(),
            peer_disconnected_listeners: Vec::new(),
            new_peak_height_listeners: Vec::new(),
        }));

        let listener =
            Arc::new(BlockListener::new(BlockListenerConfig::default()).expect("listener init"));

        // Subscribe to core events and forward relevant ones to JS listeners
        let mut rx = listener.subscribe();
        let listeners_for_task = listeners.clone();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(ev) => {
                        match ev {
                            chia_block_listener::types::Event::PeerConnected(pc) => {
                                let js = PeerConnectedEvent {
                                    peer_id: pc.peer_id,
                                    host: pc.host,
                                    port: pc.port,
                                };
                                let guard = listeners_for_task.read().await;
                                for l in &guard.peer_connected_listeners {
                                    l.call(js.clone(), ThreadsafeFunctionCallMode::NonBlocking);
                                }
                            }
                            chia_block_listener::types::Event::PeerDisconnected(pd) => {
                                let js = PeerDisconnectedEvent {
                                    peer_id: pd.peer_id,
                                    host: pd.host,
                                    port: pd.port,
                                    message: pd.message,
                                };
                                let guard = listeners_for_task.read().await;
                                for l in &guard.peer_disconnected_listeners {
                                    l.call(js.clone(), ThreadsafeFunctionCallMode::NonBlocking);
                                }
                            }
                            chia_block_listener::types::Event::NewPeakHeight(np) => {
                                let guard = listeners_for_task.read().await;
                                for l in &guard.new_peak_height_listeners {
                                    l.call(np.clone(), ThreadsafeFunctionCallMode::NonBlocking);
                                }
                            }
                            chia_block_listener::types::Event::BlockReceived(_) => {
                                // ChiaPeerPool adapter does not forward blockReceived; that lives on ChiaBlockListener
                            }
                        }
                    }
                    Err(_lagged) => {
                        // best-effort delivery; continue
                        continue;
                    }
                }
            }
        });

        Self {
            listener,
            listeners,
        }
    }

    #[napi(js_name = "addPeer")]
    pub async fn add_peer(&self, host: String, port: u16, network_id: String) -> Result<String> {
        self.listener
            .add_peer(host, port, network_id)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to add peer: {e}")))
    }

    #[napi(js_name = "getBlockByHeight")]
    pub async fn get_block_by_height(&self, height: u32) -> Result<BlockReceivedEvent> {
        let core_block: CoreBlockReceivedEvent = self
            .listener
            .get_block_by_height(height as u64)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get block: {e}")))?;

        Ok(convert_core_block_to_js(core_block))
    }

    #[napi(js_name = "removePeer")]
    pub async fn remove_peer(&self, peer_id: String) -> Result<bool> {
        self.listener.remove_peer(peer_id).await.map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to remove peer: {e}"),
            )
        })
    }

    #[napi]
    pub async fn shutdown(&self) -> Result<()> {
        self.listener.shutdown().await.map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to shutdown pool: {e}"),
            )
        })
    }

    #[napi(js_name = "getConnectedPeers")]
    pub async fn get_connected_peers(&self) -> Result<Vec<String>> {
        self.listener.get_connected_peers().await.map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to get connected peers: {e}"),
            )
        })
    }

    #[napi(js_name = "getPeakHeight")]
    pub async fn get_peak_height(&self) -> Result<Option<u32>> {
        Ok(self.listener.get_highest_peak().await)
    }

    #[napi]
    pub fn on(&self, event: String, callback: JsFunction) -> Result<()> {
        let rt = tokio::runtime::Handle::current();

        match event.as_str() {
            "peerConnected" => {
                let tsfn = callback.create_threadsafe_function(0, |ctx| {
                    let event: &PeerConnectedEvent = &ctx.value;
                    let mut obj = ctx.env.create_object()?;
                    obj.set_named_property("peerId", ctx.env.create_string(&event.peer_id)?)?;
                    obj.set_named_property("host", ctx.env.create_string(&event.host)?)?;
                    obj.set_named_property("port", ctx.env.create_uint32(event.port)?)?;
                    Ok(vec![obj])
                })?;

                let mut guard = rt.block_on(self.listeners.write());
                guard.peer_connected_listeners.push(tsfn);
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

                let mut guard = rt.block_on(self.listeners.write());
                guard.peer_disconnected_listeners.push(tsfn);
            }
            "newPeakHeight" => {
                let tsfn = callback.create_threadsafe_function(0, |ctx| {
                    let event: &NewPeakHeightEvent = &ctx.value;
                    let mut obj = ctx.env.create_object()?;
                    match event.old_peak {
                        Some(old) => {
                            obj.set_named_property("oldPeak", ctx.env.create_uint32(old)?)?
                        }
                        None => obj.set_named_property("oldPeak", ctx.env.get_null()?)?,
                    }
                    obj.set_named_property("newPeak", ctx.env.create_uint32(event.new_peak)?)?;
                    obj.set_named_property("peerId", ctx.env.create_string(&event.peer_id)?)?;
                    Ok(vec![obj])
                })?;

                let mut guard = rt.block_on(self.listeners.write());
                guard.new_peak_height_listeners.push(tsfn);
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
        let listeners = self.listeners.clone();

        rt.block_on(async {
            let mut guard = listeners.write().await;

            match event.as_str() {
                "peerConnected" => {
                    guard.peer_connected_listeners.clear();
                }
                "peerDisconnected" => {
                    guard.peer_disconnected_listeners.clear();
                }
                "newPeakHeight" => {
                    guard.new_peak_height_listeners.clear();
                }
                _ => {}
            }
        });

        Ok(())
    }
}

fn convert_core_block_to_js(core: CoreBlockReceivedEvent) -> BlockReceivedEvent {
    BlockReceivedEvent {
        peer_id: core.peer_id,
        height: core.height,
        weight: core.weight,
        header_hash: core.header_hash,
        timestamp: core.timestamp,
        coin_additions: core
            .coin_additions
            .into_iter()
            .map(|c| crate::event_emitter::CoinRecord {
                parent_coin_info: c.parent_coin_info,
                puzzle_hash: c.puzzle_hash,
                amount: c.amount,
            })
            .collect(),
        coin_removals: core
            .coin_removals
            .into_iter()
            .map(|c| crate::event_emitter::CoinRecord {
                parent_coin_info: c.parent_coin_info,
                puzzle_hash: c.puzzle_hash,
                amount: c.amount,
            })
            .collect(),
        coin_spends: core
            .coin_spends
            .into_iter()
            .map(|s| crate::event_emitter::CoinSpend {
                coin: crate::event_emitter::CoinRecord {
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
            .map(|c| crate::event_emitter::CoinRecord {
                parent_coin_info: c.parent_coin_info,
                puzzle_hash: c.puzzle_hash,
                amount: c.amount,
            })
            .collect(),
        has_transactions_generator: core.has_transactions_generator,
        generator_size: core.generator_size,
    }
}
