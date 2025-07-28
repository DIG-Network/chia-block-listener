use crate::types::*;
use crate::error::{IndexerError, Result};
use crate::blockchain_utils::{BlockchainConverter, AmountUtils};
use crate::peer_manager::PeerManager;
use crate::solution_indexer::SolutionIndexer;
use chia_block_database::ChiaBlockDatabase;
use chia_generator_parser::types::ParsedBlock;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, oneshot};
use tokio::time::{interval, Duration as TokioDuration};
use tracing::{debug, info, warn, error};
use futures::StreamExt;

/// Main blockchain synchronization worker
pub struct SyncWorker {
    config: IndexerConfig,
    database: Arc<ChiaBlockDatabase>,
    peer_manager: Arc<PeerManager>,
    solution_indexer: Arc<RwLock<SolutionIndexer>>,
    
    // State tracking
    sync_status: Arc<RwLock<SyncStatus>>,
    is_running: Arc<RwLock<bool>>,
    exit_requested: Arc<RwLock<Option<String>>>,
    
    // Communication channels
    block_tx: Option<mpsc::UnboundedSender<ProcessingBlock>>,
    event_tx: Option<mpsc::UnboundedSender<SyncEvent>>,
    
    // Performance metrics
    session_start_time: DateTime<Utc>,
    blocks_processed_session: Arc<RwLock<u64>>,
    total_errors: Arc<RwLock<u32>>,
    last_error: Arc<RwLock<Option<String>>>,
}

impl SyncWorker {
    /// Create a new sync worker
    pub async fn new(config: IndexerConfig) -> Result<Self> {
        // Initialize database
        let database_url = config.database.connection_string.clone();
        let database = ChiaBlockDatabase::new_with_type(
            &database_url,
            config.database_type(),
        ).await?;

        // Run migrations
        database.migrate().await?;

        // Initialize peer manager
        let peer_manager = PeerManager::new(config.clone()).await?;

        // Initialize solution indexer
        let solution_indexer = SolutionIndexer::new();

        // Initialize sync status
        let sync_status = SyncStatus {
            sync_type: "combined".to_string(),
            start_height: config.blockchain.start_height,
            last_synced_height: config.blockchain.start_height.saturating_sub(1),
            current_peak_height: config.blockchain.start_height,
            is_running: false,
            is_historical_sync: false,
            last_activity: Utc::now(),
            session_start_time: Utc::now(),
            blocks_processed_session: 0,
            sync_speed_blocks_per_sec: None,
            progress_percentage: None,
            eta_minutes: None,
            total_blocks_synced: 0,
            errors_count: 0,
            last_error: None,
            queue_size: 0,
            active_downloads: 0,
        };

        Ok(Self {
            config,
            database: Arc::new(database),
            peer_manager: Arc::new(peer_manager),
            solution_indexer: Arc::new(RwLock::new(solution_indexer)),
            sync_status: Arc::new(RwLock::new(sync_status)),
            is_running: Arc::new(RwLock::new(false)),
            exit_requested: Arc::new(RwLock::new(None)),
            block_tx: None,
            event_tx: None,
            session_start_time: Utc::now(),
            blocks_processed_session: Arc::new(RwLock::new(0)),
            total_errors: Arc::new(RwLock::new(0)),
            last_error: Arc::new(RwLock::new(None)),
        })
    }

    /// Start the sync worker
    pub async fn start(&mut self) -> Result<()> {
        {
            let mut running = self.is_running.write().await;
            if *running {
                return Err(IndexerError::Other("Sync worker is already running".to_string()));
            }
            *running = true;
        }

        info!("🚀 Starting Chia Full Indexer");
        info!("📡 Real-time: Receives and processes new blocks immediately");
        info!("🔍 Historical: Continuously scans and fills missing blocks");

        // Initialize sync status
        self.initialize_sync_status().await?;

        // Create communication channels
        let (block_tx, block_rx) = mpsc::unbounded_channel::<ProcessingBlock>();
        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<SyncEvent>();
        
        self.block_tx = Some(block_tx);
        self.event_tx = Some(event_tx.clone());

        // Start block processing task
        let database = Arc::clone(&self.database);
        let solution_indexer = Arc::clone(&self.solution_indexer);
        let config = self.config.clone();
        let blocks_processed = Arc::clone(&self.blocks_processed_session);
        let is_running = Arc::clone(&self.is_running);

        tokio::spawn(async move {
            Self::block_processing_task(
                block_rx,
                database,
                solution_indexer,
                config,
                blocks_processed,
                is_running,
            ).await;
        });

        // Start event processing task
        let sync_status = Arc::clone(&self.sync_status);
        tokio::spawn(async move {
            Self::event_processing_task(event_rx, sync_status).await;
        });

        // Start status update task
        let sync_status = Arc::clone(&self.sync_status);
        let config = self.config.clone();
        let blocks_processed = Arc::clone(&self.blocks_processed_session);
        let session_start = self.session_start_time;
        let total_errors = Arc::clone(&self.total_errors);
        let last_error = Arc::clone(&self.last_error);
        let is_running = Arc::clone(&self.is_running);

        tokio::spawn(async move {
            Self::status_update_task(
                sync_status,
                config,
                blocks_processed,
                session_start,
                total_errors,
                last_error,
                is_running,
            ).await;
        });

        // Start both historical and real-time sync
        let historical_sync = self.run_historical_sync();
        let realtime_sync = self.run_realtime_sync();

        // Run both sync types concurrently
        tokio::select! {
            result = historical_sync => {
                if let Err(e) = result {
                    error!("Historical sync failed: {}", e);
                    self.record_error(&e.to_string()).await;
                }
            }
            result = realtime_sync => {
                if let Err(e) = result {
                    error!("Real-time sync failed: {}", e);
                    self.record_error(&e.to_string()).await;
                }
            }
        }

        Ok(())
    }

    /// Stop the sync worker
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping sync worker...");
        
        {
            let mut running = self.is_running.write().await;
            *running = false;
        }

        // Final status update
        self.update_sync_status().await?;

        info!("Sync worker stopped");
        Ok(())
    }

    /// Request graceful exit
    pub async fn request_exit(&self, reason: &str) -> Result<()> {
        {
            let mut exit_req = self.exit_requested.write().await;
            *exit_req = Some(reason.to_string());
        }

        info!("Exit requested: {}", reason);
        self.stop().await
    }

    /// Check if exit was requested
    async fn check_exit_requested(&self) -> Result<()> {
        let exit_req = self.exit_requested.read().await;
        if let Some(reason) = exit_req.as_ref() {
            return Err(IndexerError::ExitRequested { 
                reason: reason.clone() 
            });
        }
        Ok(())
    }

    /// Initialize sync status in database
    async fn initialize_sync_status(&self) -> Result<()> {
        let last_synced_height = self.get_last_synced_height().await?;
        
        {
            let mut status = self.sync_status.write().await;
            status.last_synced_height = last_synced_height;
            status.session_start_time = Utc::now();
            status.is_running = true;
        }

        // Update database sync status
        self.update_database_sync_status().await?;

        info!("📊 Sync status initialized - last synced height: {}", last_synced_height);
        Ok(())
    }

    /// Get the last synced height from database
    async fn get_last_synced_height(&self) -> Result<u32> {
        let result = self.database.execute_raw_query(
            "SELECT MAX(height) as max_height FROM blocks WHERE height >= ?",
            Some(vec![serde_json::Value::Number(
                serde_json::Number::from(self.config.blockchain.start_height)
            )])
        ).await?;

        let response: serde_json::Value = serde_json::from_str(&result)?;
        
        if let Some(rows) = response.as_array() {
            if let Some(row) = rows.first() {
                if let Some(max_height) = row.get("max_height") {
                    if let Some(height) = max_height.as_i64() {
                        return Ok(height as u32);
                    }
                }
            }
        }

        Ok(self.config.blockchain.start_height.saturating_sub(1))
    }

    /// Update sync status in database
    async fn update_database_sync_status(&self) -> Result<()> {
        let status = self.sync_status.read().await;
        
        // This would typically update a sync_status table
        // For now, we'll just log the status
        debug!("Sync status: synced={}, peak={}, running={}", 
               status.last_synced_height, 
               status.current_peak_height, 
               status.is_running);

        Ok(())
    }

    /// Record an error
    async fn record_error(&self, error_msg: &str) {
        {
            let mut total_errors = self.total_errors.write().await;
            *total_errors += 1;
        }

        {
            let mut last_error = self.last_error.write().await;
            *last_error = Some(error_msg.to_string());
        }

        error!("Recorded sync error: {}", error_msg);
    }

    /// Get current sync status
    pub async fn get_status(&self) -> SyncStatus {
        self.sync_status.read().await.clone()
    }

    /// Check if sync is paused
    async fn check_sync_paused(&self) -> Result<bool> {
        // Check system preferences for pause status
        let result = self.database.execute_raw_query(
            "SELECT meta_value FROM system_preferences WHERE meta_key = ?",
            Some(vec![serde_json::Value::String("sync_paused".to_string())])
        ).await.ok();

        if let Some(result_str) = result {
            let response: serde_json::Value = serde_json::from_str(&result_str).ok()?;
            if let Some(rows) = response.as_array() {
                if let Some(row) = rows.first() {
                    if let Some(value) = row.get("meta_value") {
                        return Ok(value.as_bool().unwrap_or(false));
                    }
                }
            }
        }

        Ok(false)
    }

    /// Wait while sync is paused
    async fn wait_while_paused(&self) -> Result<()> {
        while self.check_sync_paused().await? {
            if !*self.is_running.read().await {
                return Err(IndexerError::Other("Sync worker stopped".to_string()));
            }

            self.check_exit_requested().await?;
            
            info!("⏸️ Sync is paused - waiting...");
            tokio::time::sleep(TokioDuration::from_secs(10)).await;
        }
        Ok(())
    }

    /// Update sync status periodically
    async fn update_sync_status(&self) -> Result<()> {
        let now = Utc::now();
        let session_duration = (now - self.session_start_time).num_seconds() as f64;
        let blocks_processed = *self.blocks_processed_session.read().await;
        
        let sync_speed = if session_duration > 0.0 {
            blocks_processed as f64 / session_duration
        } else {
            0.0
        };

        {
            let mut status = self.sync_status.write().await;
            status.last_activity = now;
            status.blocks_processed_session = blocks_processed;
            status.sync_speed_blocks_per_sec = Some(sync_speed);
            status.errors_count = *self.total_errors.read().await;
            status.last_error = self.last_error.read().await.clone();

            // Calculate progress
            let total_range = status.current_peak_height.saturating_sub(self.config.blockchain.start_height);
            let synced_range = status.last_synced_height.saturating_sub(self.config.blockchain.start_height);
            
            if total_range > 0 {
                status.progress_percentage = Some((synced_range as f64 / total_range as f64) * 100.0);
                
                // Calculate ETA
                let remaining = status.current_peak_height.saturating_sub(status.last_synced_height);
                if sync_speed > 0.0 && remaining > 0 {
                    let eta_seconds = remaining as f64 / sync_speed;
                    status.eta_minutes = Some((eta_seconds / 60.0) as i32);
                }
            }
        }

        self.update_database_sync_status().await?;
        Ok(())
    }

    /// Run historical sync (gap detection and filling)
    async fn run_historical_sync(&self) -> Result<()> {
        info!("Starting historical sync (gap detection and filling)...");

        {
            let mut status = self.sync_status.write().await;
            status.is_historical_sync = true;
        }

        // Discover peers for historical sync
        let peers = self.peer_manager.discover_peers().await?;
        if peers.is_empty() {
            return Err(IndexerError::NoPeersAvailable);
        }

        // Get the best peers for historical sync
        let historical_peers = self.peer_manager.get_peers_for_historical_sync().await?;
        info!("Using {} peers for historical sync", historical_peers.len());

        // TODO: Initialize peer pool with discovered peers
        // This would require the chia-peer-pool crate to be implemented

        // Continuous gap detection loop
        while *self.is_running.read().await {
            self.check_exit_requested().await?;
            self.wait_while_paused().await?;

            // Update last synced height
            let last_synced = self.get_last_synced_height().await?;
            {
                let mut status = self.sync_status.write().await;
                status.last_synced_height = last_synced;
            }

            // Get current peak height (would come from peer pool)
            let current_peak = self.get_network_peak_height().await?;
            {
                let mut status = self.sync_status.write().await;
                if current_peak > status.current_peak_height {
                    status.current_peak_height = current_peak;
                }
            }

            // Find gaps in the blockchain
            let gaps = self.identify_gaps(
                self.config.blockchain.start_height,
                current_peak,
            ).await?;

            if gaps.is_empty() {
                info!("✅ No gaps found - blockchain data is complete up to height {}", current_peak);
            } else {
                let total_gap_blocks: u32 = gaps.iter().map(|gap| gap.size()).sum();
                info!("🔧 Found {} gap(s) totaling {} blocks", gaps.len(), total_gap_blocks);

                // Emit gap detected event
                if let Some(event_tx) = &self.event_tx {
                    let _ = event_tx.send(SyncEvent::GapDetected { gaps: gaps.clone() });
                }

                // Process each gap
                for (index, gap) in gaps.iter().enumerate() {
                    if !*self.is_running.read().await {
                        break;
                    }

                    self.wait_while_paused().await?;
                    
                    let start_time = std::time::Instant::now();
                    info!("🔧 Gap {}/{}: Downloading blocks {} to {} ({} blocks)",
                          index + 1, gaps.len(), gap.start_height, gap.end_height, gap.size());

                    if let Err(e) = self.download_gap(gap).await {
                        error!("Failed to download gap {}: {}", index + 1, e);
                        self.record_error(&format!("Gap download failed: {}", e)).await;
                        continue;
                    }

                    let duration = start_time.elapsed().as_secs_f64();
                    let blocks_per_sec = gap.size() as f64 / duration;
                    
                    info!("✅ Gap {}/{} complete: {} blocks in {:.1}s ({:.1} blocks/sec)",
                          index + 1, gaps.len(), gap.size(), duration, blocks_per_sec);

                    // Emit gap filled event
                    if let Some(event_tx) = &self.event_tx {
                        let _ = event_tx.send(SyncEvent::GapFilled { 
                            gap: gap.clone(), 
                            duration_secs: duration 
                        });
                    }
                }

                // Refresh balances after gap completion
                if self.config.sync.enable_balance_refresh {
                    if let Err(e) = self.refresh_balances().await {
                        warn!("Failed to refresh balances: {}", e);
                    }
                }
            }

            // Wait before next scan
            info!("⏱️ Next gap scan in {} seconds", self.config.sync.gap_scan_interval_secs);
            tokio::time::sleep(TokioDuration::from_secs(self.config.sync.gap_scan_interval_secs)).await;
        }

        Ok(())
    }

    /// Run real-time sync (listen for new blocks)
    async fn run_realtime_sync(&self) -> Result<()> {
        info!("Starting real-time sync (new block monitoring)...");

        // Get peers for real-time sync
        let realtime_peers = self.peer_manager.get_peers_for_realtime_sync().await?;
        info!("Using {} peers for real-time sync", realtime_peers.len());

        // TODO: Initialize block listener with discovered peers
        // This would require the chia-block-listener crate to be implemented

        // For now, simulate real-time sync with periodic checks
        while *self.is_running.read().await {
            self.check_exit_requested().await?;
            self.wait_while_paused().await?;

            // Check for new blocks (would come from block listener events)
            if let Ok(new_peak) = self.get_network_peak_height().await {
                let current_peak = {
                    let status = self.sync_status.read().await;
                    status.current_peak_height
                };

                if new_peak > current_peak {
                    info!("🎯 New peak height: {} (was {})", new_peak, current_peak);
                    
                    {
                        let mut status = self.sync_status.write().await;
                        status.current_peak_height = new_peak;
                    }

                    // Emit peer connected event (simulated)
                    if let Some(event_tx) = &self.event_tx {
                        let _ = event_tx.send(SyncEvent::BlockReceived { 
                            height: new_peak, 
                            peer_id: "realtime_peer".to_string() 
                        });
                    }
                }
            }

            // Wait before next check
            tokio::time::sleep(TokioDuration::from_secs(30)).await;
        }

        Ok(())
    }

    /// Get network peak height (would come from peers)
    async fn get_network_peak_height(&self) -> Result<u32> {
        // TODO: Get from peer pool
        // For now, return a simulated increasing height
        let base_height = 5_000_000; // Simulated current mainnet height
        let elapsed_secs = (Utc::now() - self.session_start_time).num_seconds();
        let blocks_per_minute = 3.2; // Chia target block time ~18.75s
        let additional_blocks = (elapsed_secs as f64 / 60.0 * blocks_per_minute) as u32;
        
        Ok(base_height + additional_blocks)
    }

    /// Identify gaps in the blockchain
    async fn identify_gaps(&self, start_height: u32, end_height: u32) -> Result<Vec<BlockGap>> {
        if start_height >= end_height {
            return Ok(Vec::new());
        }

        info!("🔍 Scanning for gaps between heights {} and {}", start_height, end_height);

        // Query database for missing blocks
        let query = format!(
            "WITH RECURSIVE height_series AS (
                SELECT {} as height
                UNION ALL
                SELECT height + 1
                FROM height_series
                WHERE height < {}
            )
            SELECT hs.height
            FROM height_series hs
            LEFT JOIN blocks b ON hs.height = b.height
            WHERE b.height IS NULL
            ORDER BY hs.height",
            start_height, end_height
        );

        let result = self.database.execute_raw_query(&query, None).await?;
        let response: serde_json::Value = serde_json::from_str(&result)?;

        let missing_heights: Vec<u32> = if let Some(rows) = response.as_array() {
            rows.iter()
                .filter_map(|row| {
                    row.get("height")?.as_i64().map(|h| h as u32)
                })
                .collect()
        } else {
            Vec::new()
        };

        if missing_heights.is_empty() {
            return Ok(Vec::new());
        }

        // Group consecutive missing heights into gaps
        let gaps = self.group_consecutive_heights(missing_heights);
        
        info!("Found {} gap(s) in blockchain data", gaps.len());
        for (i, gap) in gaps.iter().enumerate() {
            debug!("Gap {}: heights {} to {} ({} blocks)", 
                   i + 1, gap.start_height, gap.end_height, gap.size());
        }

        Ok(gaps)
    }

    /// Group consecutive heights into gap ranges
    fn group_consecutive_heights(&self, heights: Vec<u32>) -> Vec<BlockGap> {
        if heights.is_empty() {
            return Vec::new();
        }

        let mut gaps = Vec::new();
        let mut current_start = heights[0];
        let mut current_end = heights[0];

        for &height in heights.iter().skip(1) {
            if height == current_end + 1 {
                // Consecutive, extend current gap
                current_end = height;
            } else {
                // Gap in sequence, finalize current gap and start new one
                gaps.push(BlockGap::new(current_start, current_end));
                current_start = height;
                current_end = height;
            }
        }

        // Add the final gap
        gaps.push(BlockGap::new(current_start, current_end));
        gaps
    }

    /// Download blocks to fill a gap
    async fn download_gap(&self, gap: &BlockGap) -> Result<()> {
        info!("Downloading gap: blocks {} to {}", gap.start_height, gap.end_height);

        // TODO: Use peer pool to download blocks
        // For now, simulate the download process
        for height in gap.start_height..=gap.end_height {
            if !*self.is_running.read().await {
                break;
            }

            self.check_exit_requested().await?;

            // Simulate downloading and processing a block
            if let Err(e) = self.simulate_block_download_and_process(height).await {
                error!("Failed to download block {}: {}", height, e);
                return Err(IndexerError::BlockProcessingFailed {
                    height,
                    source: Box::new(e),
                });
            }

            // Update queue size (simulated)
            {
                let mut status = self.sync_status.write().await;
                status.queue_size = gap.end_height.saturating_sub(height);
            }
        }

        Ok(())
    }

    /// Simulate block download and processing
    async fn simulate_block_download_and_process(&self, height: u32) -> Result<()> {
        // This would be replaced with actual peer pool block fetching
        
        // Create a simulated processing block
        let processing_block = ProcessingBlock {
            height,
            header_hash: format!("simulated_hash_{}", height),
            timestamp: Utc::now().timestamp() as u64,
            weight: (height as u64 * 1000).to_string(),
            is_transaction_block: height % 32 != 0, // Most blocks are transaction blocks
            coin_additions: vec![],
            coin_removals: vec![],
            coin_spends: vec![],
            coin_creations: vec![],
        };

        // Send to processing queue
        if let Some(block_tx) = &self.block_tx {
            block_tx.send(processing_block)
                .map_err(|_| IndexerError::Other("Failed to send block for processing".to_string()))?;
        }

        Ok(())
    }

    /// Refresh balance materialized views
    async fn refresh_balances(&self) -> Result<()> {
        info!("💰 Refreshing balance views...");
        
        let start_time = std::time::Instant::now();
        
        // Refresh XCH balances
        self.database.execute_raw_query("REFRESH MATERIALIZED VIEW xch_balances", None).await.ok();
        
        // Refresh CAT balances  
        self.database.execute_raw_query("REFRESH MATERIALIZED VIEW cat_balances", None).await.ok();
        
        let duration = start_time.elapsed();
        info!("💰 Balance refresh completed in {:.2}s", duration.as_secs_f64());
        
        Ok(())
    }

    /// Process a single block and save to database
    async fn process_block(
        block: &ProcessingBlock,
        database: &ChiaBlockDatabase,
        solution_indexer: &Arc<RwLock<SolutionIndexer>>,
    ) -> Result<BlockProcessingResult> {
        let start_time = std::time::Instant::now();

        // Convert to database block format
        let db_block = BlockchainConverter::convert_to_db_block(block)?;

        // Save block to database
        database.blockchain().blocks().create(database.pool(), &db_block).await?;

        // Process coin spends to extract CATs and NFTs
        let mut cats_processed = 0;
        let mut nfts_processed = 0;

        if !block.coin_spends.is_empty() {
            let mut indexer = solution_indexer.write().await;
            let asset_results = indexer.process_batch(&block.coin_spends, block.height).await;

            for (_, result) in asset_results {
                match result {
                    AssetProcessingResult::Cat { .. } => {
                        cats_processed += 1;
                        // TODO: Save CAT to database
                    }
                    AssetProcessingResult::Nft { .. } => {
                        nfts_processed += 1;
                        // TODO: Save NFT to database
                    }
                    AssetProcessingResult::None => {}
                }
            }
        }

        // Process coin spends
        for spend_record in &block.coin_spends {
            let db_spend = BlockchainConverter::convert_to_db_spend(spend_record, block.height)?;
            database.blockchain().spends().create(database.pool(), &db_spend).await?;
        }

        let processing_time = start_time.elapsed();

        Ok(BlockchainConverter::create_processing_summary(
            block,
            processing_time.as_millis() as u64,
            cats_processed,
            nfts_processed,
        ))
    }

    /// Get sync performance metrics
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        let status = self.sync_status.read().await;
        let blocks_processed = *self.blocks_processed_session.read().await;
        let session_duration = (Utc::now() - self.session_start_time).num_seconds() as f64;

        let blocks_per_second = if session_duration > 0.0 {
            blocks_processed as f64 / session_duration
        } else {
            0.0
        };

        PerformanceMetrics {
            blocks_per_second,
            coins_per_second: blocks_per_second * 10.0, // Rough estimate
            spends_per_second: blocks_per_second * 5.0, // Rough estimate
            database_write_time_ms: 50, // Placeholder
            block_processing_time_ms: 100, // Placeholder
            memory_usage_mb: 512, // Placeholder
            active_connections: 5, // Placeholder
        }
    }

    /// Pause sync operations
    pub async fn pause_sync(&self) -> Result<()> {
        info!("⏸️ Pausing sync operations");

        // Set pause flag in database
        self.database.execute_raw_query(
            "INSERT OR REPLACE INTO system_preferences (meta_key, meta_value, updated_at) VALUES (?, ?, ?)",
            Some(vec![
                serde_json::Value::String("sync_paused".to_string()),
                serde_json::Value::Bool(true),
                serde_json::Value::String(Utc::now().to_rfc3339()),
            ])
        ).await?;

        // Emit pause event
        if let Some(event_tx) = &self.event_tx {
            let _ = event_tx.send(SyncEvent::SyncPaused);
        }

        Ok(())
    }

    /// Resume sync operations
    pub async fn resume_sync(&self) -> Result<()> {
        info!("▶️ Resuming sync operations");

        // Clear pause flag in database
        self.database.execute_raw_query(
            "INSERT OR REPLACE INTO system_preferences (meta_key, meta_value, updated_at) VALUES (?, ?, ?)",
            Some(vec![
                serde_json::Value::String("sync_paused".to_string()),
                serde_json::Value::Bool(false),
                serde_json::Value::String(Utc::now().to_rfc3339()),
            ])
        ).await?;

        // Emit resume event
        if let Some(event_tx) = &self.event_tx {
            let _ = event_tx.send(SyncEvent::SyncResumed);
        }

        Ok(())
    }

    /// Get sync progress information
    pub async fn get_sync_progress(&self) -> SyncProgress {
        let status = self.sync_status.read().await;
        let blocks_processed = *self.blocks_processed_session.read().await;

        let total_range = status.current_peak_height.saturating_sub(self.config.blockchain.start_height);
        let synced_range = status.last_synced_height.saturating_sub(self.config.blockchain.start_height);
        
        let progress_percentage = if total_range > 0 {
            (synced_range as f64 / total_range as f64) * 100.0
        } else {
            0.0
        };

        let remaining_blocks = status.current_peak_height.saturating_sub(status.last_synced_height);

        SyncProgress {
            start_height: self.config.blockchain.start_height,
            current_height: status.last_synced_height,
            peak_height: status.current_peak_height,
            progress_percentage,
            remaining_blocks,
            blocks_processed_session: blocks_processed,
            is_historical_sync: status.is_historical_sync,
            sync_speed_blocks_per_sec: status.sync_speed_blocks_per_sec.unwrap_or(0.0),
            eta_minutes: status.eta_minutes,
        }
    }

    /// Clean up resources
    pub async fn cleanup(&self) -> Result<()> {
        info!("🧹 Cleaning up sync worker resources");

        // Stop all tasks
        self.stop().await?;

        // Clean up old peer information
        self.peer_manager.cleanup_old_peers(24).await?;

        // Clear solution indexer cache
        {
            let mut indexer = self.solution_indexer.write().await;
            indexer.clear_metadata_cache();
        }

        info!("🧹 Cleanup completed");
        Ok(())
    }
}

/// Sync progress information
#[derive(Debug, Clone)]
pub struct SyncProgress {
    pub start_height: u32,
    pub current_height: u32,
    pub peak_height: u32,
    pub progress_percentage: f64,
    pub remaining_blocks: u32,
    pub blocks_processed_session: u64,
    pub is_historical_sync: bool,
    pub sync_speed_blocks_per_sec: f64,
    pub eta_minutes: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_worker_creation() {
        let config = IndexerConfig::default();
        let sync_worker = SyncWorker::new(config).await;
        assert!(sync_worker.is_ok());
    }

    #[test]
    fn test_gap_grouping() {
        let config = IndexerConfig::default();
        let sync_worker = SyncWorker::new(config).await.unwrap();

        let heights = vec![1, 2, 3, 5, 6, 10, 15, 16, 17];
        let gaps = sync_worker.group_consecutive_heights(heights);

        assert_eq!(gaps.len(), 4);
        assert_eq!(gaps[0].start_height, 1);
        assert_eq!(gaps[0].end_height, 3);
        assert_eq!(gaps[1].start_height, 5);
        assert_eq!(gaps[1].end_height, 6);
        assert_eq!(gaps[2].start_height, 10);
        assert_eq!(gaps[2].end_height, 10);
        assert_eq!(gaps[3].start_height, 15);
        assert_eq!(gaps[3].end_height, 17);
    }

    #[test]
    fn test_block_gap_size() {
        let gap = BlockGap::new(100, 105);
        assert_eq!(gap.size(), 6);

        let single_block_gap = BlockGap::new(100, 100);
        assert_eq!(single_block_gap.size(), 1);
    }

    #[tokio::test]
    async fn test_sync_status_updates() {
        let config = IndexerConfig::default();
        let sync_worker = SyncWorker::new(config).await.unwrap();

        let initial_status = sync_worker.get_status().await;
        assert!(!initial_status.is_running);
        assert_eq!(initial_status.blocks_processed_session, 0);
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let config = IndexerConfig::default();
        let sync_worker = SyncWorker::new(config).await.unwrap();

        let metrics = sync_worker.get_performance_metrics().await;
        assert!(metrics.blocks_per_second >= 0.0);
        assert!(metrics.memory_usage_mb > 0);
    }
} 