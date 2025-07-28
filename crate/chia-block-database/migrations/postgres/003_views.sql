-- =============================================================================
-- Migration 003: Views and Materialized Views
-- =============================================================================
-- Creates optimized views for common queries: unspent_coins, xch_balances, sync_metrics

-- =============================================================================
-- Unspent Coins View
-- =============================================================================
-- Create optimized unspent_coins view using NOT EXISTS for efficient unspent coin lookups
CREATE OR REPLACE VIEW unspent_coins AS
SELECT 
  c.coin_id,
  c.puzzle_hash,
  c.amount,
  c.created_block,
  c.parent_coin_info
FROM coins c
WHERE NOT EXISTS (SELECT 1 FROM spends s WHERE s.coin_id = c.coin_id);

-- =============================================================================
-- XCH Balances Materialized View
-- =============================================================================
-- Create optimized xch_balances using a more efficient approach
-- Drop existing view/materialized view first (handle both cases)
DROP MATERIALIZED VIEW IF EXISTS xch_balances CASCADE;
DROP VIEW IF EXISTS xch_balances CASCADE;

-- Create materialized view for much better performance
CREATE MATERIALIZED VIEW IF NOT EXISTS xch_balances AS
SELECT 
  encode(c.puzzle_hash, 'hex') as puzzle_hash_hex,
  COUNT(*) as coin_count,
  SUM(c.amount) as total_mojos,
  ROUND(SUM(c.amount) / 1000000000000.0, 12) as total_xch,
  MIN(c.amount) as min_coin_amount,
  MAX(c.amount) as max_coin_amount,
  ROUND(AVG(c.amount)::numeric, 0) as avg_coin_amount, -- Round to avoid decimal places for mojo amounts
  CURRENT_TIMESTAMP as last_updated
FROM coins c
WHERE NOT EXISTS (SELECT 1 FROM spends s WHERE s.coin_id = c.coin_id)
GROUP BY c.puzzle_hash
ORDER BY total_mojos DESC;

-- Create index on the materialized view for fast lookups
CREATE UNIQUE INDEX IF NOT EXISTS idx_xch_balances_puzzle_hash ON xch_balances(puzzle_hash_hex);
CREATE INDEX IF NOT EXISTS idx_xch_balances_total_mojos ON xch_balances(total_mojos DESC);

-- =============================================================================
-- Sync Metrics View
-- =============================================================================
-- Create sync_metrics view for monitoring with accurate gap-based progress
CREATE OR REPLACE VIEW sync_metrics AS
WITH sync_data AS (
  SELECT 
    s.*,
    -- Calculate actual blocks we have in our range
    (SELECT COUNT(*) FROM blocks WHERE height >= s.start_height AND height <= s.current_peak_height) as actual_blocks_count,
    -- Calculate total blocks expected in our range
    GREATEST(s.current_peak_height - s.start_height + 1, 1) as expected_blocks_count
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
      ROUND((actual_blocks_count * 100.0 / expected_blocks_count)::numeric, 2) as progress_percentage,
  queue_size,
  active_downloads,
  is_running,
  is_historical_sync,
  sync_speed_blocks_per_sec,
  eta_minutes,
  blocks_processed_session,
  EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - session_start_time)) / 60 as session_minutes,
  CASE 
    WHEN EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - session_start_time)) > 0
            THEN ROUND((blocks_processed_session / (EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - session_start_time)) / 60))::numeric, 2)
    ELSE 0
  END as session_blocks_per_minute,
  actual_blocks_count as total_blocks_synced,
  expected_blocks_count,
  (expected_blocks_count - actual_blocks_count) as total_missing_blocks,
  errors_count,
  last_error,
  EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - last_activity)) as seconds_since_last_activity,
  last_activity
FROM sync_data
ORDER BY last_activity DESC; 