-- =============================================================================
-- Migration 003: Views and Tables (SQLite)
-- =============================================================================
-- Creates optimized views for common queries: unspent_coins, xch_balances, sync_metrics
-- Note: SQLite doesn't support materialized views, so we use regular tables that can be refreshed

-- =============================================================================
-- Unspent Coins View
-- =============================================================================
-- Create optimized unspent_coins view using NOT EXISTS for efficient unspent coin lookups
CREATE VIEW IF NOT EXISTS unspent_coins AS
SELECT 
  c.coin_id,
  c.puzzle_hash,
  c.amount,
  c.created_block,
  c.parent_coin_info
FROM coins c
WHERE NOT EXISTS (SELECT 1 FROM spends s WHERE s.coin_id = c.coin_id);

-- =============================================================================
-- XCH Balances Table (replaces materialized view)
-- =============================================================================
-- Create table for XCH balances since SQLite doesn't support materialized views
DROP TABLE IF EXISTS xch_balances;

CREATE TABLE IF NOT EXISTS xch_balances (
  puzzle_hash_hex TEXT PRIMARY KEY,
  coin_count INTEGER NOT NULL,
  total_mojos INTEGER NOT NULL,
  total_xch REAL NOT NULL,
  min_coin_amount INTEGER NOT NULL,
  max_coin_amount INTEGER NOT NULL,
  avg_coin_amount INTEGER NOT NULL,
  last_updated TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Create indexes on the table for fast lookups
CREATE INDEX IF NOT EXISTS idx_xch_balances_total_mojos ON xch_balances(total_mojos DESC);

-- =============================================================================
-- Sync Metrics View
-- =============================================================================
-- Create sync_metrics view for monitoring with accurate gap-based progress
CREATE VIEW IF NOT EXISTS sync_metrics AS
WITH sync_data AS (
  SELECT 
    s.*,
    -- Calculate actual blocks we have in our range
    (SELECT COUNT(*) FROM blocks WHERE height >= s.start_height AND height <= s.current_peak_height) as actual_blocks_count,
    -- Calculate total blocks expected in our range
    MAX(s.current_peak_height - s.start_height + 1, 1) as expected_blocks_count
  FROM sync_status s
)
SELECT 
  sync_type,
  current_peak_height,
  last_synced_height,
  -- Blocks remaining = expected - actual
  (expected_blocks_count - actual_blocks_count) as blocks_remaining,
  start_height,
  actual_blocks_count as blocks_completed,
  -- Accurate progress = (actual blocks / expected blocks) * 100
  ROUND((actual_blocks_count * 100.0 / expected_blocks_count), 2) as progress_percentage,
  queue_size,
  active_downloads,
  is_running,
  is_historical_sync,
  sync_speed_blocks_per_sec,
  eta_minutes,
  blocks_processed_session,
  -- Calculate session minutes using datetime functions
  ROUND((julianday('now') - julianday(session_start_time)) * 24 * 60, 2) as session_minutes,
  CASE 
    WHEN (julianday('now') - julianday(session_start_time)) * 24 * 60 > 0
        THEN ROUND((blocks_processed_session / ((julianday('now') - julianday(session_start_time)) * 24 * 60)), 2)
    ELSE 0
  END as session_blocks_per_minute,
  actual_blocks_count as total_blocks_synced,
  expected_blocks_count,
  (expected_blocks_count - actual_blocks_count) as total_missing_blocks,
  errors_count,
  last_error,
  -- Calculate seconds since last activity
  ROUND((julianday('now') - julianday(last_activity)) * 24 * 60 * 60, 0) as seconds_since_last_activity,
  last_activity
FROM sync_data
ORDER BY last_activity DESC; 