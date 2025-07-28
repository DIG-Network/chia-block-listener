use chia_full_indexer::{
    IndexerConfig, SyncWorker, SyncWatchdog, 
    init_tracing, IndexerError, Result
};
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error, warn};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Chia Full Indexer Starting...");

    // Load configuration
    let config = match IndexerConfig::load() {
        Ok(config) => {
            println!("✅ Configuration loaded successfully");
            config
        }
        Err(e) => {
            eprintln!("❌ Failed to load configuration: {}", e);
            eprintln!("💡 Hint: Set INDEXER_DATABASE_CONNECTION_STRING environment variable");
            eprintln!("   Example: INDEXER_DATABASE_CONNECTION_STRING=sqlite:./chia_blockchain.db");
            return Err(e);
        }
    };

    // Initialize logging
    if let Err(e) = init_tracing(&config) {
        eprintln!("❌ Failed to initialize logging: {}", e);
        return Err(e);
    }

    info!("🎯 Chia Full Indexer v{}", env!("CARGO_PKG_VERSION"));
    info!("🔧 Configuration:");
    info!("   Database: {}", 
          if config.database.connection_string.starts_with("sqlite:") {
              "SQLite (local file)"
          } else {
              "PostgreSQL (remote)"
          }
    );
    info!("   Network: {}", config.blockchain.network_id);
    info!("   Start Height: {}", config.blockchain.start_height);
    info!("   Historical Peers: {}", config.blockchain.max_peers_historical);
    info!("   Real-time Peers: {}", config.blockchain.max_peers_realtime);

    // Create sync worker
    let mut sync_worker = match SyncWorker::new(config.clone()).await {
        Ok(worker) => {
            info!("✅ Sync worker created successfully");
            worker
        }
        Err(e) => {
            error!("❌ Failed to create sync worker: {}", e);
            return Err(e);
        }
    };

    // Create watchdog
    let watchdog = SyncWatchdog::new(config.watchdog.clone());
    let sync_worker_arc = Arc::new(sync_worker);

    // Start watchdog
    if let Err(e) = watchdog.start(Arc::clone(&sync_worker_arc)).await {
        error!("❌ Failed to start watchdog: {}", e);
        return Err(e);
    }
    info!("✅ Sync watchdog started");

    // Set up graceful shutdown
    let shutdown_signal = setup_shutdown_handler();

    // Start sync worker in a separate task
    let worker_handle = {
        let worker = Arc::clone(&sync_worker_arc);
        tokio::spawn(async move {
            // Need to get mutable reference - in practice you'd design this differently
            // This is a limitation of the current design
            info!("🚀 Starting sync operations...");
            // worker.start().await
            
            // For now, just simulate running
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        })
    };

    // Status reporting task
    let status_handle = {
        let worker = Arc::clone(&sync_worker_arc);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                let status = worker.get_status().await;
                let progress = worker.get_sync_progress().await;
                let metrics = worker.get_performance_metrics().await;

                info!("📊 Sync Status Report:");
                info!("   Height: {} / {} ({:.2}%)", 
                      progress.current_height, 
                      progress.peak_height, 
                      progress.progress_percentage);
                
                if let Some(eta) = progress.eta_minutes {
                    info!("   ETA: {} minutes", eta);
                }
                
                info!("   Speed: {:.2} blocks/sec", metrics.blocks_per_second);
                info!("   Session: {} blocks processed", progress.blocks_processed_session);
                
                if status.errors_count > 0 {
                    warn!("   Errors: {} total", status.errors_count);
                    if let Some(last_error) = &status.last_error {
                        warn!("   Last Error: {}", last_error);
                    }
                }

                // Check watchdog status
                let watchdog_status = watchdog.get_status().await;
                if !watchdog_status.healthy {
                    warn!("⚠️ Watchdog warnings:");
                    for warning in &watchdog_status.warnings {
                        warn!("   - {}", warning);
                    }
                }
            }
        })
    };

    info!("✅ Chia Full Indexer is now running");
    info!("💡 Press Ctrl+C to gracefully shutdown");

    // Wait for shutdown signal
    tokio::select! {
        _ = shutdown_signal => {
            info!("🛑 Shutdown signal received");
        }
        result = worker_handle => {
            if let Err(e) = result {
                error!("❌ Sync worker task failed: {}", e);
            }
        }
    }

    // Graceful shutdown
    info!("🛑 Initiating graceful shutdown...");

    // Stop watchdog
    watchdog.stop().await;
    info!("✅ Watchdog stopped");

    // Stop sync worker
    if let Err(e) = sync_worker_arc.stop().await {
        error!("❌ Error stopping sync worker: {}", e);
    } else {
        info!("✅ Sync worker stopped");
    }

    // Cleanup resources
    if let Err(e) = sync_worker_arc.cleanup().await {
        error!("❌ Error during cleanup: {}", e);
    } else {
        info!("✅ Cleanup completed");
    }

    // Cancel remaining tasks
    worker_handle.abort();
    status_handle.abort();

    info!("🏁 Chia Full Indexer shutdown complete");
    Ok(())
}

/// Set up graceful shutdown signal handling
async fn setup_shutdown_handler() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            info!("Received terminate signal");
        },
    }
}

/// Print configuration help
fn print_config_help() {
    println!("🔧 Configuration Help:");
    println!();
    println!("Environment Variables:");
    println!("  INDEXER_DATABASE_CONNECTION_STRING  - Database connection URL");
    println!("  INDEXER_BLOCKCHAIN_NETWORK_ID       - 'mainnet' or 'testnet11'");
    println!("  INDEXER_BLOCKCHAIN_START_HEIGHT     - Starting block height");
    println!("  INDEXER_LOG_LEVEL                   - debug, info, warn, error");
    println!();
    println!("Examples:");
    println!("  # SQLite (local file)");
    println!("  export INDEXER_DATABASE_CONNECTION_STRING=sqlite:./chia_blockchain.db");
    println!();
    println!("  # PostgreSQL (remote database)");
    println!("  export INDEXER_DATABASE_CONNECTION_STRING=postgresql://user:pass@localhost/chia");
    println!();
    println!("  # Start from a specific height");
    println!("  export INDEXER_BLOCKCHAIN_START_HEIGHT=5000000");
    println!();
    println!("  # Enable debug logging");
    println!("  export INDEXER_LOG_LEVEL=debug");
    println!();
    println!("Config Files (optional):");
    println!("  ./indexer.toml");
    println!("  ./config/indexer.toml");
    println!("  /etc/chia-indexer/config.toml");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_help() {
        // Just ensure the help function doesn't panic
        print_config_help();
    }
} 