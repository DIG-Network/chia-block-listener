use crate::error::ChiaError;
use crate::peer_pool::ChiaPeerPool;
use crate::types::{Event, ListenerConfig};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Listener {
    pool: Arc<ChiaPeerPool>,
    tx: broadcast::Sender<Event>,
}

impl Listener {
    pub fn new(config: ListenerConfig) -> Result<Self, ChiaError> {
        let (tx, _rx) = broadcast::channel(config.buffer);
        let pool = Arc::new(ChiaPeerPool::new());

        // Wire peer-connected/disconnected/new-peak from pool into broadcast channel
        {
            let tx_connected = tx.clone();
            let tx_disconnected = tx.clone();
            let tx_new_peak = tx.clone();

            pool.set_event_callbacks(
                Box::new(move |ev| {
                    let _ = tx_connected.send(Event::PeerConnected(ev));
                }),
                Box::new(move |ev| {
                    let _ = tx_disconnected.send(Event::PeerDisconnected(ev));
                }),
                Box::new(move |ev| {
                    let _ = tx_new_peak.send(Event::NewPeakHeight(ev));
                }),
            );
        }

        // Wire block-received callback into broadcast channel (for historical pulls)
        {
            let tx_block = tx.clone();
            pool.set_block_received_callback(Box::new(move |blk| {
                let _ = tx_block.send(Event::BlockReceived(blk));
            }));
        }

        Ok(Self { pool, tx })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }

    pub async fn add_peer(
        &self,
        host: String,
        port: u16,
        network_id: String,
    ) -> Result<String, ChiaError> {
        self.pool.add_peer(host, port, network_id).await
    }

    pub async fn remove_peer(&self, peer_id: String) -> Result<bool, ChiaError> {
        self.pool.remove_peer(peer_id).await
    }

    pub async fn get_connected_peers(&self) -> Result<Vec<String>, ChiaError> {
        self.pool.get_connected_peers().await
    }

    pub async fn get_highest_peak(&self) -> Option<u32> {
        self.pool.get_highest_peak().await
    }

    pub async fn get_block_by_height(
        &self,
        height: u64,
    ) -> Result<crate::types::BlockReceivedEvent, ChiaError> {
        self.pool.get_block_by_height(height).await
    }

    // Structured shutdown: currently both methods await completion
    pub async fn shutdown(&self) -> Result<(), ChiaError> {
        self.pool.shutdown().await
    }

    pub async fn shutdown_and_wait(&self) -> Result<(), ChiaError> {
        self.pool.shutdown().await
    }
}
