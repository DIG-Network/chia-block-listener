use crate::types::*;
use crate::error::{IndexerError, Result};
use crate::sync_worker::SyncWorker;
use chrono::{DateTime, Utc, Duration};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration as TokioDuration};
use tracing::{debug, info, warn, error};

/// Monitors sync worker health and detects stuck conditions
pub struct SyncWatchdog {
    config: WatchdogConfig,
    state: Arc<RwLock<WatchdogState>>,
    is_running: Arc<RwLock<bool>>,
}

impl SyncWatchdog {
    /// Create a new sync watchdog
    pub fn new(config: WatchdogConfig) -> Self {
        let initial_state = WatchdogState {
            last_synced_height: 0,
            last_peak_height: 0,
            last_activity_time: Utc::now(),
            last_peak_update_time: Utc::now(),
            last_progress_time: Utc::now(),
            gap_fill_start_time: None,
            is_gap_filling: false,
            consecutive_no_progress_checks: 0,
        };

        Self {
            config,
            state: Arc::new(RwLock::new(initial_state)),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the watchdog monitoring
    pub async fn start(&self, sync_worker: Arc<SyncWorker>) -> Result<()> {
        {
            let mut running = self.is_running.write().await;
            if *running {
                return Err(IndexerError::Other("Watchdog is already running".to_string()));
            }
            *running = true;
        }

        info!("Starting sync watchdog with {} second check interval", 
              self.config.check_interval_secs);

        let state = Arc::clone(&self.state);
        let is_running = Arc::clone(&self.is_running);
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut check_interval = interval(TokioDuration::from_secs(config.check_interval_secs));

            while *is_running.read().await {
                check_interval.tick().await;

                if let Err(e) = Self::perform_health_check(
                    &sync_worker,
                    &state,
                    &config,
                ).await {
                    error!("Watchdog health check failed: {}", e);
                }
            }

            info!("Sync watchdog stopped");
        });

        Ok(())
    }

    /// Stop the watchdog
    pub async fn stop(&self) {
        let mut running = self.is_running.write().await;
        *running = false;
        info!("Stopping sync watchdog");
    }

    /// Perform a health check on the sync worker
    async fn perform_health_check(
        sync_worker: &SyncWorker,
        state: &Arc<RwLock<WatchdogState>>,
        config: &WatchdogConfig,
    ) -> Result<()> {
        let now = Utc::now();
        let worker_status = sync_worker.get_status().await;

        // Update watchdog state
        let mut watchdog_state = state.write().await;
        let previous_synced_height = watchdog_state.last_synced_height;
        let previous_peak_height = watchdog_state.last_peak_height;

        // Update state from worker status
        watchdog_state.last_synced_height = worker_status.last_synced_height;
        
        // Check if peak height updated
        if worker_status.current_peak_height > watchdog_state.last_peak_height {
            watchdog_state.last_peak_height = worker_status.current_peak_height;
            watchdog_state.last_peak_update_time = now;
        }

        // Check if synced height progressed
        if worker_status.last_synced_height > previous_synced_height {
            watchdog_state.last_progress_time = now;
            watchdog_state.consecutive_no_progress_checks = 0;
            watchdog_state.last_activity_time = now;
        } else {
            watchdog_state.consecutive_no_progress_checks += 1;
        }

        // Update gap filling status
        let was_gap_filling = watchdog_state.is_gap_filling;
        watchdog_state.is_gap_filling = worker_status.is_historical_sync;

        if watchdog_state.is_gap_filling && !was_gap_filling {
            // Started gap filling
            watchdog_state.gap_fill_start_time = Some(now);
        } else if !watchdog_state.is_gap_filling && was_gap_filling {
            // Finished gap filling
            watchdog_state.gap_fill_start_time = None;
        }

        // Detect problems
        let warnings = Self::detect_health_issues(&watchdog_state, config, now);

        if !warnings.is_empty() {
            warn!("Sync health warnings detected:");
            for warning in &warnings {
                warn!("  - {}", warning);
            }

            // Log current status for debugging
            Self::log_detailed_status(&worker_status, &watchdog_state);

            // Check if we should trigger an exit
            let should_exit = Self::should_trigger_exit(&warnings, config);
            if should_exit {
                error!("Critical sync issues detected - requesting graceful shutdown");
                if let Err(e) = sync_worker.request_exit("Watchdog detected critical issues").await {
                    error!("Failed to request sync worker exit: {}", e);
                }
            }
        } else {
            debug!("Sync health check passed - all systems operating normally");
        }

        drop(watchdog_state); // Release the lock

        Ok(())
    }

    /// Detect health issues based on watchdog state
    fn detect_health_issues(
        state: &WatchdogState,
        config: &WatchdogConfig,
        now: DateTime<Utc>,
    ) -> Vec<String> {
        let mut warnings = Vec::new();

        let max_stuck_duration = Duration::seconds(config.max_stuck_time_secs as i64);
        let timeout_duration = Duration::minutes(config.timeout_minutes as i64);

        // Check for no progress
        let time_since_progress = now - state.last_progress_time;
        if time_since_progress > max_stuck_duration {
            warnings.push(format!(
                "No sync progress for {} minutes (last progress: {})",
                time_since_progress.num_minutes(),
                state.last_progress_time.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        // Check for no peak height updates
        let time_since_peak_update = now - state.last_peak_update_time;
        if time_since_peak_update > timeout_duration {
            warnings.push(format!(
                "No peak height updates for {} minutes (last update: {})",
                time_since_peak_update.num_minutes(),
                state.last_peak_update_time.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        // Check for excessive gap filling time
        if let Some(gap_start) = state.gap_fill_start_time {
            let gap_duration = now - gap_start;
            if gap_duration > timeout_duration {
                warnings.push(format!(
                    "Gap filling has been running for {} minutes",
                    gap_duration.num_minutes()
                ));
            }
        }

        // Check for consecutive no-progress checks
        if state.consecutive_no_progress_checks >= config.max_no_progress_checks as u32 {
            warnings.push(format!(
                "No progress detected for {} consecutive checks",
                state.consecutive_no_progress_checks
            ));
        }

        // Check for general inactivity
        let time_since_activity = now - state.last_activity_time;
        if time_since_activity > timeout_duration {
            warnings.push(format!(
                "No activity detected for {} minutes",
                time_since_activity.num_minutes()
            ));
        }

        warnings
    }

    /// Check if warnings warrant triggering an exit
    fn should_trigger_exit(warnings: &[String], config: &WatchdogConfig) -> bool {
        let critical_keywords = [
            "No sync progress",
            "No peak height updates",
            "consecutive checks",
            "No activity detected",
        ];

        let critical_warnings = warnings.iter()
            .filter(|warning| {
                critical_keywords.iter().any(|keyword| warning.contains(keyword))
            })
            .count();

        // Trigger exit if we have multiple critical warnings or specific conditions
        critical_warnings >= 2 ||
        warnings.iter().any(|w| w.contains(&format!("{} consecutive", config.max_no_progress_checks)))
    }

    /// Log detailed status for debugging
    fn log_detailed_status(worker_status: &SyncStatus, watchdog_state: &WatchdogState) {
        info!("=== Detailed Sync Status ===");
        info!("Worker Status:");
        info!("  - Running: {}", worker_status.is_running);
        info!("  - Historical Sync: {}", worker_status.is_historical_sync);
        info!("  - Last Synced: {}", worker_status.last_synced_height);
        info!("  - Peak Height: {}", worker_status.current_peak_height);
        info!("  - Queue Size: {}", worker_status.queue_size);
        info!("  - Active Downloads: {}", worker_status.active_downloads);
        if let Some(speed) = worker_status.sync_speed_blocks_per_sec {
            info!("  - Sync Speed: {:.2} blocks/sec", speed);
        }
        if let Some(progress) = worker_status.progress_percentage {
            info!("  - Progress: {:.2}%", progress);
        }

        info!("Watchdog State:");
        info!("  - Last Activity: {}", watchdog_state.last_activity_time.format("%Y-%m-%d %H:%M:%S"));
        info!("  - Last Progress: {}", watchdog_state.last_progress_time.format("%Y-%m-%d %H:%M:%S"));
        info!("  - Last Peak Update: {}", watchdog_state.last_peak_update_time.format("%Y-%m-%d %H:%M:%S"));
        info!("  - Gap Filling: {}", watchdog_state.is_gap_filling);
        if let Some(gap_start) = watchdog_state.gap_fill_start_time {
            info!("  - Gap Fill Started: {}", gap_start.format("%Y-%m-%d %H:%M:%S"));
        }
        info!("  - No Progress Checks: {}", watchdog_state.consecutive_no_progress_checks);
        info!("===============================");
    }

    /// Get current watchdog status
    pub async fn get_status(&self) -> WatchdogStatus {
        let state = self.state.read().await;
        let is_running = *self.is_running.read().await;
        let now = Utc::now();

        let warnings = Self::detect_health_issues(&state, &self.config, now);

        WatchdogStatus {
            is_running,
            healthy: warnings.is_empty(),
            warnings,
            state: state.clone(),
            time_since_activity: (now - state.last_activity_time).num_seconds() as u64,
            time_since_progress: (now - state.last_progress_time).num_seconds() as u64,
            time_since_peak_update: (now - state.last_peak_update_time).num_seconds() as u64,
        }
    }

    /// Update activity timestamp (called by sync worker)
    pub async fn update_activity(&self) {
        let mut state = self.state.write().await;
        state.last_activity_time = Utc::now();
    }

    /// Manually trigger a health check
    pub async fn manual_health_check(&self, sync_worker: &SyncWorker) -> Result<Vec<String>> {
        let state = Arc::clone(&self.state);
        let config = self.config.clone();
        
        Self::perform_health_check(sync_worker, &state, &config).await?;
        
        let state_guard = state.read().await;
        let warnings = Self::detect_health_issues(&state_guard, &config, Utc::now());
        Ok(warnings)
    }
}

/// Watchdog status information
#[derive(Debug, Clone)]
pub struct WatchdogStatus {
    pub is_running: bool,
    pub healthy: bool,
    pub warnings: Vec<String>,
    pub state: WatchdogState,
    pub time_since_activity: u64, // seconds
    pub time_since_progress: u64, // seconds
    pub time_since_peak_update: u64, // seconds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watchdog_creation() {
        let config = WatchdogConfig {
            timeout_minutes: 10,
            check_interval_secs: 60,
            max_stuck_time_secs: 600,
            max_no_progress_checks: 5,
        };

        let watchdog = SyncWatchdog::new(config);
        assert!(!watchdog.is_running.try_read().unwrap().clone());
    }

    #[test]
    fn test_health_issue_detection() {
        let config = WatchdogConfig {
            timeout_minutes: 10,
            check_interval_secs: 60,
            max_stuck_time_secs: 600,
            max_no_progress_checks: 5,
        };

        let now = Utc::now();
        let mut state = WatchdogState {
            last_synced_height: 1000,
            last_peak_height: 1000,
            last_activity_time: now - Duration::minutes(15), // 15 minutes ago
            last_peak_update_time: now - Duration::minutes(15),
            last_progress_time: now - Duration::minutes(15),
            gap_fill_start_time: None,
            is_gap_filling: false,
            consecutive_no_progress_checks: 0,
        };

        let warnings = SyncWatchdog::detect_health_issues(&state, &config, now);
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.contains("No activity")));

        // Test with recent activity
        state.last_activity_time = now - Duration::minutes(1);
        state.last_progress_time = now - Duration::minutes(1);
        state.last_peak_update_time = now - Duration::minutes(1);

        let warnings = SyncWatchdog::detect_health_issues(&state, &config, now);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_exit_triggering() {
        let config = WatchdogConfig {
            timeout_minutes: 10,
            check_interval_secs: 60,
            max_stuck_time_secs: 600,
            max_no_progress_checks: 5,
        };

        let warnings = vec![
            "No sync progress for 15 minutes".to_string(),
            "No peak height updates for 15 minutes".to_string(),
        ];

        assert!(SyncWatchdog::should_trigger_exit(&warnings, &config));

        let warnings = vec!["Minor warning".to_string()];
        assert!(!SyncWatchdog::should_trigger_exit(&warnings, &config));
    }
} 