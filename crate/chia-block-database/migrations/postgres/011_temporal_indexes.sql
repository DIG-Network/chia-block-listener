-- =============================================================================
-- Migration 011: Temporal Indexes for Time-Based Analytics
-- =============================================================================
-- Creates specialized indexes optimized for temporal analytics, trend analysis,
-- and time-based aggregations

-- =============================================================================
-- Block Temporal Indexes
-- =============================================================================

-- Index for hourly block production analysis (EXTRACT(HOUR FROM timestamp))
CREATE INDEX IF NOT EXISTS idx_blocks_hour_analysis ON blocks(EXTRACT(HOUR FROM timestamp), height);

-- Index for daily block production analysis (DATE(timestamp))
CREATE INDEX IF NOT EXISTS idx_blocks_daily_analysis ON blocks(DATE(timestamp), height);

-- Index for block interval analysis (LAG/LEAD window functions)
CREATE INDEX IF NOT EXISTS idx_blocks_interval_analysis ON blocks(height, timestamp);

-- Index for block production variance analysis
CREATE INDEX IF NOT EXISTS idx_blocks_production_metrics ON blocks(timestamp, weight, height);

-- Index for time range queries with block metrics
CREATE INDEX IF NOT EXISTS idx_blocks_time_range_metrics ON blocks(timestamp, height, weight);

-- =============================================================================
-- Coin Temporal Indexes for Age and Lifecycle Analysis
-- =============================================================================

-- Index for coin age distribution analysis (current_height - created_block)
CREATE INDEX IF NOT EXISTS idx_coins_age_analysis ON coins(created_block, amount, coin_id);

-- Index for coin creation timeline analysis
CREATE INDEX IF NOT EXISTS idx_coins_creation_timeline ON coins(created_block, puzzle_hash, amount);

-- Index for unspent coin age analysis (optimized for NOT EXISTS queries)
CREATE INDEX IF NOT EXISTS idx_coins_unspent_age ON coins(created_block, amount) 
  WHERE NOT EXISTS (SELECT 1 FROM spends WHERE spends.coin_id = coins.coin_id);

-- Index for coin value creation over time
CREATE INDEX IF NOT EXISTS idx_coins_value_timeline ON coins(created_block, amount, puzzle_hash);

-- Index for address first activity tracking
CREATE INDEX IF NOT EXISTS idx_coins_address_first_activity ON coins(puzzle_hash, created_block);

-- =============================================================================
-- Spend Temporal Indexes for Transaction Timing Analysis
-- =============================================================================

-- Index for spend velocity analysis (spent_block - created_block)
CREATE INDEX IF NOT EXISTS idx_spends_velocity_analysis ON spends(spent_block, coin_id) 
  INCLUDE (puzzle_id, solution_id);

-- Index for spend timing patterns
CREATE INDEX IF NOT EXISTS idx_spends_timing_patterns ON spends(spent_block) 
  INCLUDE (coin_id, puzzle_id, solution_id);

-- Index for hourly spend analysis
CREATE INDEX IF NOT EXISTS idx_spends_hourly_analysis ON spends(spent_block);

-- Composite index for spend frequency analysis by address
CREATE INDEX IF NOT EXISTS idx_spends_address_frequency ON spends(spent_block)
  INCLUDE (coin_id);

-- =============================================================================
-- Balance Evolution Temporal Indexes
-- =============================================================================

-- Index for tracking balance changes over time (coins received)
CREATE INDEX IF NOT EXISTS idx_coins_balance_evolution_received ON coins(puzzle_hash, created_block, amount, coin_id);

-- Index for tracking balance changes over time (coins spent)
CREATE INDEX IF NOT EXISTS idx_spends_balance_evolution_spent ON spends(spent_block)
  INCLUDE (coin_id);

-- Combined index for complete balance history
CREATE INDEX IF NOT EXISTS idx_balance_history_combined ON coins(puzzle_hash, created_block)
  INCLUDE (amount, coin_id);

-- Index for address activity timeline
CREATE INDEX IF NOT EXISTS idx_address_activity_timeline ON coins(puzzle_hash, created_block, amount);

-- =============================================================================
-- Network Growth Temporal Indexes
-- =============================================================================

-- Index for daily network growth metrics
CREATE INDEX IF NOT EXISTS idx_network_daily_growth ON blocks(DATE(timestamp))
  INCLUDE (height, timestamp);

-- Index for puzzle introduction timeline
CREATE INDEX IF NOT EXISTS idx_puzzles_introduction_timeline ON puzzles(first_seen_block, reveal_size)
  INCLUDE (puzzle_hash, id);

-- Index for solution introduction timeline  
CREATE INDEX IF NOT EXISTS idx_solutions_introduction_timeline ON solutions(first_seen_block, solution_size)
  INCLUDE (solution_hash, id);

-- Index for new address discovery timeline
CREATE INDEX IF NOT EXISTS idx_new_addresses_timeline ON coins(created_block, puzzle_hash);

-- =============================================================================
-- CAT Temporal Indexes
-- =============================================================================

-- Index for CAT creation timeline
CREATE INDEX IF NOT EXISTS idx_cats_creation_timeline ON cats(created_block, asset_id, amount);

-- Index for CAT asset introduction timeline  
CREATE INDEX IF NOT EXISTS idx_cat_assets_timeline ON cat_assets(first_seen_block, asset_id);

-- Index for CAT activity over time
CREATE INDEX IF NOT EXISTS idx_cats_activity_timeline ON cats(created_block, owner_puzzle_hash, asset_id);

-- Index for CAT value movement timeline
CREATE INDEX IF NOT EXISTS idx_cats_value_timeline ON cats(created_block, asset_id, amount);

-- =============================================================================
-- NFT Temporal Indexes
-- =============================================================================

-- Index for NFT creation timeline
CREATE INDEX IF NOT EXISTS idx_nfts_creation_timeline ON nfts(created_block, collection_id);

-- Index for NFT update timeline
CREATE INDEX IF NOT EXISTS idx_nfts_update_timeline ON nfts(last_updated_block, launcher_id);

-- Index for NFT collection timeline
CREATE INDEX IF NOT EXISTS idx_nft_collections_timeline ON nft_collections(first_seen_block, collection_id);

-- Index for NFT ownership change timeline
CREATE INDEX IF NOT EXISTS idx_nft_ownership_timeline ON nft_ownership_history(block_height, launcher_id);

-- Index for NFT transfer timeline by type
CREATE INDEX IF NOT EXISTS idx_nft_transfer_timeline ON nft_ownership_history(block_height, transfer_type);

-- Index for NFT activity heatmap
CREATE INDEX IF NOT EXISTS idx_nft_activity_heatmap ON nft_ownership_history(block_height)
  INCLUDE (launcher_id, transfer_type);

-- =============================================================================
-- Temporal Range Query Optimization Indexes
-- =============================================================================

-- Index for efficient time range queries on blocks
CREATE INDEX IF NOT EXISTS idx_blocks_range_optimized ON blocks(timestamp, height)
  INCLUDE (header_hash, weight);

-- Index for time range coin queries
CREATE INDEX IF NOT EXISTS idx_coins_range_optimized ON coins(created_block, puzzle_hash)
  INCLUDE (amount, coin_id);

-- Index for time range spend queries
CREATE INDEX IF NOT EXISTS idx_spends_range_optimized ON spends(spent_block, coin_id)
  INCLUDE (puzzle_id, solution_id);

-- =============================================================================
-- Activity Heatmap Temporal Indexes
-- =============================================================================

-- Index for hourly activity patterns
CREATE INDEX IF NOT EXISTS idx_activity_hourly_pattern ON blocks(EXTRACT(HOUR FROM timestamp), DATE(timestamp));

-- Index for weekly activity patterns  
CREATE INDEX IF NOT EXISTS idx_activity_weekly_pattern ON blocks(EXTRACT(DOW FROM timestamp), DATE(timestamp));

-- Index for monthly activity patterns
CREATE INDEX IF NOT EXISTS idx_activity_monthly_pattern ON blocks(EXTRACT(MONTH FROM timestamp), EXTRACT(YEAR FROM timestamp));

-- =============================================================================
-- Peak Activity Detection Indexes
-- =============================================================================

-- Index for identifying peak activity periods
CREATE INDEX IF NOT EXISTS idx_peak_activity_detection ON blocks(timestamp)
  INCLUDE (height);

-- Index for transaction volume peaks
CREATE INDEX IF NOT EXISTS idx_transaction_peaks ON spends(spent_block)
  INCLUDE (coin_id);

-- Index for value movement peaks
CREATE INDEX IF NOT EXISTS idx_value_peaks ON coins(created_block, amount);

-- =============================================================================
-- Temporal Aggregation Optimization Indexes
-- =============================================================================

-- Index for efficient daily aggregations
CREATE INDEX IF NOT EXISTS idx_daily_aggregation_opt ON blocks(DATE(timestamp), height, timestamp);

-- Index for efficient hourly aggregations
CREATE INDEX IF NOT EXISTS idx_hourly_aggregation_opt ON blocks(DATE_TRUNC('hour', timestamp), height);

-- Index for efficient weekly aggregations
CREATE INDEX IF NOT EXISTS idx_weekly_aggregation_opt ON blocks(DATE_TRUNC('week', timestamp), height);

-- Index for efficient monthly aggregations
CREATE INDEX IF NOT EXISTS idx_monthly_aggregation_opt ON blocks(DATE_TRUNC('month', timestamp), height);

-- =============================================================================
-- Time-Based Filtering Optimization
-- =============================================================================

-- Index for "last N days" queries
CREATE INDEX IF NOT EXISTS idx_recent_activity ON blocks(timestamp DESC, height DESC)
  WHERE timestamp >= NOW() - INTERVAL '30 days';

-- Index for "last N blocks" queries  
CREATE INDEX IF NOT EXISTS idx_recent_blocks ON blocks(height DESC, timestamp DESC)
  WHERE height >= (SELECT MAX(height) - 1000 FROM blocks);

-- Index for historical analysis queries
CREATE INDEX IF NOT EXISTS idx_historical_analysis ON blocks(timestamp, height)
  WHERE timestamp < NOW() - INTERVAL '7 days';

-- =============================================================================
-- Sync Status Temporal Indexes
-- =============================================================================

-- Index for sync progress timeline
CREATE INDEX IF NOT EXISTS idx_sync_progress_timeline ON sync_status(last_activity, current_peak_height);

-- Index for sync performance analysis
CREATE INDEX IF NOT EXISTS idx_sync_performance_timeline ON sync_status(session_start_time, blocks_processed_session);

-- =============================================================================
-- Comments and Usage Notes
-- =============================================================================

-- These temporal indexes are designed to optimize:
-- 1. Time-based aggregations (hourly, daily, weekly, monthly)
-- 2. Coin age distribution analysis
-- 3. Balance evolution tracking
-- 4. Network growth metrics
-- 5. Activity heatmap generation
-- 6. Peak activity detection
-- 7. Temporal range queries
-- 8. Asset (CAT/NFT) timeline analysis
-- 9. Transaction timing analysis
-- 10. Historical trend analysis
--
-- Usage tips:
-- - Use DATE() functions for daily aggregations
-- - Use EXTRACT(HOUR FROM timestamp) for hourly patterns
-- - Use DATE_TRUNC() for flexible time period grouping
-- - Combine temporal indexes with covering indexes for best performance
-- - Consider partial indexes for recent activity queries 