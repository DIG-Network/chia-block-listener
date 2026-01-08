use crate::error::ChiaError;
use crate::peer_pool::ChiaPeerPool;
use crate::types::{Event, ListenerConfig};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub struct Listener {
    pool: Arc<ChiaPeerPool>,
    tx: broadcast::Sender<Event>,
    cancel: CancellationToken,
    dispatcher: Mutex<Option<JoinHandle<()>>>,
}

impl Listener {
    pub fn new(config: ListenerConfig) -> Result<Self, ChiaError> {
        let (tx, _rx) = broadcast::channel(config.buffer);

        // Internal event sink between pool -> listener dispatcher
        let (sink_tx, mut sink_rx) = mpsc::channel::<Event>(config.buffer);
        let cancel = CancellationToken::new();

        // Pool constructed with event sink and shared cancellation
        let pool = Arc::new(ChiaPeerPool::new_with_event_sink(sink_tx, cancel.clone()));

        // Dispatcher: forwards from sink to broadcast without blocking pool
        let tx_clone = tx.clone();
        let cancel_clone = cancel.clone();
        let dispatcher = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_clone.cancelled() => {
                        break;
                    }
                    maybe = sink_rx.recv() => {
                        match maybe {
                            Some(ev) => { let _ = tx_clone.send(ev); }
                            None => break,
                        }
                    }
                }
            }
        });

        Ok(Self { pool, tx, cancel, dispatcher: Mutex::new(Some(dispatcher)) })
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
        if self.cancel.is_cancelled() {
            return Err(ChiaError::Other("shutting down".into()));
        }
        self.pool.add_peer(host, port, network_id).await
    }

    pub async fn remove_peer(&self, peer_id: String) -> Result<bool, ChiaError> {
        if self.cancel.is_cancelled() {
            return Err(ChiaError::Other("shutting down".into()));
        }
        self.pool.remove_peer(peer_id).await
    }

    pub async fn get_connected_peers(&self) -> Result<Vec<String>, ChiaError> {
        if self.cancel.is_cancelled() {
            return Err(ChiaError::Other("shutting down".into()));
        }
        self.pool.get_connected_peers().await
    }

    pub async fn get_highest_peak(&self) -> Option<u32> {
        self.pool.get_highest_peak().await
    }

    pub async fn get_block_by_height(
        &self,
        height: u64,
    ) -> Result<crate::types::BlockReceivedEvent, ChiaError> {
        if self.cancel.is_cancelled() {
            return Err(ChiaError::Other("shutting down".into()));
        }
        self.pool.get_block_by_height(height).await
    }

    // Structured shutdown
    pub async fn shutdown(&self) -> Result<(), ChiaError> {
        // Signal cancellation quickly and initiate pool shutdown
        self.cancel.cancel();
        self.pool.shutdown().await
    }

    pub async fn shutdown_and_wait(&self) -> Result<(), ChiaError> {
        // First signal shutdown
        self.shutdown().await?;

        // Take and await dispatcher task cooperatively (no abort)
        if let Some(handle) = self.dispatcher.lock().ok().and_then(|mut g| g.take()) {
            let _ = handle.await;
        }

        // Await pool tasks
        self.pool.shutdown_and_wait().await
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        // Ensure background tasks are signalled to stop if user drops without explicit shutdown
        self.cancel.cancel();
        // Do not await here; Drop must be non-blocking.
    }
}
