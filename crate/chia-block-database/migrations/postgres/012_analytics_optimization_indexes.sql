-- =============================================================================
-- Migration 012: Analytics Optimization Indexes
-- =============================================================================
-- Creates specialized indexes to optimize all analytics queries in the functions modules
-- These indexes fill gaps not covered by previous migrations

-- =============================================================================
-- Address Analytics Optimization Indexes
-- =============================================================================

-- Index for address grouping with encoded puzzle hash (used extensively in address analytics)
CREATE INDEX IF NOT EXISTS idx_coins_encoded_puzzle_hash ON coins(encode(puzzle_hash, 'hex'), created_block, amount);

-- Index for address activity analysis with encoded puzzle hash
CREATE INDEX IF NOT EXISTS idx_coins_address_analysis ON coins(encode(puzzle_hash, 'hex'), created_block)
  INCLUDE (coin_id, amount);

-- Index for spend analysis by address (joining spends with coins on puzzle_hash)
CREATE INDEX IF NOT EXISTS idx_spends_address_analysis ON spends(spent_block)
  INCLUDE (coin_id);

-- Composite index for address balance evolution tracking
CREATE INDEX IF NOT EXISTS idx_address_balance_tracking ON coins(encode(puzzle_hash, 'hex'), created_block, amount, coin_id);

-- Index for dormant address detection (last activity analysis)
CREATE INDEX IF NOT EXISTS idx_address_last_activity ON coins(puzzle_hash)
  INCLUDE (created_block, amount, coin_id);

-- Index for new address discovery (MIN created_block analysis)
CREATE INDEX IF NOT EXISTS idx_new_address_discovery ON coins(created_block, encode(puzzle_hash, 'hex'));

-- =============================================================================
-- Network Analytics Optimization Indexes
-- =============================================================================

-- Index for network statistics aggregation (multiple table joins)
CREATE INDEX IF NOT EXISTS idx_network_stats_blocks ON blocks(height)
  INCLUDE (weight, header_hash, timestamp);

-- Index for UTXO analysis with NOT EXISTS optimization
CREATE INDEX IF NOT EXISTS idx_utxo_analysis_coins ON coins(coin_id, puzzle_hash, amount, created_block);

-- Index for unspent coin detection optimization
CREATE INDEX IF NOT EXISTS idx_unspent_detection ON coins(coin_id)
  INCLUDE (puzzle_hash, amount, created_block)
  WHERE NOT EXISTS (SELECT 1 FROM spends WHERE spends.coin_id = coins.coin_id);

-- Index for peak activity detection (hourly aggregations)
CREATE INDEX IF NOT EXISTS idx_peak_activity_hourly ON blocks(DATE_TRUNC('hour', timestamp), height)
  INCLUDE (timestamp);

-- =============================================================================
-- Transaction Analytics Optimization Indexes
-- =============================================================================

-- Index for spend velocity analysis (spend_block - created_block calculations)
CREATE INDEX IF NOT EXISTS idx_spend_velocity_calc ON spends(spent_block, coin_id)
  INCLUDE (puzzle_id, solution_id);

-- Index for transaction size distribution analysis
CREATE INDEX IF NOT EXISTS idx_transaction_size_analysis ON coins(amount, created_block)
  INCLUDE (coin_id, puzzle_hash);

-- Index for complex transaction pattern analysis (block-level aggregations)
CREATE INDEX IF NOT EXISTS idx_complex_transaction_patterns ON spends(spent_block)
  INCLUDE (coin_id, puzzle_id, solution_id);

-- Index for spending frequency by address analysis
CREATE INDEX IF NOT EXISTS idx_spending_frequency_analysis ON spends(spent_block)
  INCLUDE (coin_id);

-- Index for coin consolidation pattern detection
CREATE INDEX IF NOT EXISTS idx_consolidation_patterns ON spends(spent_block)
  INCLUDE (coin_id, puzzle_id, solution_id);

-- =============================================================================
-- Cross-Table Analytics Optimization Indexes
-- =============================================================================

-- Index for coins-spends join optimization in analytics
CREATE INDEX IF NOT EXISTS idx_coins_spends_analytics_join ON coins(coin_id, puzzle_hash, amount, created_block);

-- Index for blocks-coins join optimization in temporal analytics
CREATE INDEX IF NOT EXISTS idx_blocks_coins_temporal_join ON coins(created_block, puzzle_hash)
  INCLUDE (coin_id, amount);

-- Index for spends-coins join optimization in transaction analytics
CREATE INDEX IF NOT EXISTS idx_spends_coins_transaction_join ON spends(coin_id, spent_block)
  INCLUDE (puzzle_id, solution_id);

-- =============================================================================
-- Puzzle Analytics Optimization Indexes
-- =============================================================================

-- Index for puzzle usage frequency analysis
CREATE INDEX IF NOT EXISTS idx_puzzle_usage_frequency ON spends(puzzle_id, spent_block)
  INCLUDE (coin_id);

-- Index for puzzle complexity analysis
CREATE INDEX IF NOT EXISTS idx_puzzle_complexity_analysis ON puzzles(reveal_size, first_seen_block)
  INCLUDE (id, puzzle_hash);

-- Index for puzzle popularity analysis (coins with puzzle)
CREATE INDEX IF NOT EXISTS idx_puzzle_popularity ON coins(puzzle_hash, created_block)
  INCLUDE (coin_id, amount);

-- Index for unspent puzzle analysis
CREATE INDEX IF NOT EXISTS idx_unspent_puzzle_analysis ON coins(puzzle_hash, amount)
  INCLUDE (coin_id, created_block)
  WHERE NOT EXISTS (SELECT 1 FROM spends WHERE spends.coin_id = coins.coin_id);

-- =============================================================================
-- Solution Analytics Optimization Indexes
-- =============================================================================

-- Index for solution usage frequency analysis
CREATE INDEX IF NOT EXISTS idx_solution_usage_frequency ON spends(solution_id, spent_block)
  INCLUDE (coin_id);

-- Index for solution efficiency analysis
CREATE INDEX IF NOT EXISTS idx_solution_efficiency ON spends(solution_id)
  INCLUDE (spent_block, coin_id);

-- Index for high-value solution analysis
CREATE INDEX IF NOT EXISTS idx_high_value_solutions ON spends(solution_id)
  INCLUDE (coin_id, spent_block);

-- =============================================================================
-- Balance Analytics Optimization Indexes
-- =============================================================================

-- Index for balance distribution analysis
CREATE INDEX IF NOT EXISTS idx_balance_distribution ON coins(amount, puzzle_hash)
  INCLUDE (coin_id, created_block)
  WHERE NOT EXISTS (SELECT 1 FROM spends WHERE spends.coin_id = coins.coin_id);

-- Index for richest addresses analysis
CREATE INDEX IF NOT EXISTS idx_richest_addresses ON coins(puzzle_hash, amount DESC)
  INCLUDE (coin_id, created_block)
  WHERE NOT EXISTS (SELECT 1 FROM spends WHERE spends.coin_id = coins.coin_id);

-- Index for balance changes analysis
CREATE INDEX IF NOT EXISTS idx_balance_changes ON coins(puzzle_hash, created_block, amount);

-- Index for address balance range queries
CREATE INDEX IF NOT EXISTS idx_address_balance_range ON coins(amount, puzzle_hash)
  INCLUDE (coin_id, created_block);

-- =============================================================================
-- CAT Analytics Optimization Indexes
-- =============================================================================

-- Index for CAT owner analysis with encoded puzzle hash
CREATE INDEX IF NOT EXISTS idx_cats_owner_encoded ON cats(encode(owner_puzzle_hash, 'hex'), asset_id, created_block)
  INCLUDE (coin_id, amount);

-- Index for CAT balance calculation optimization
CREATE INDEX IF NOT EXISTS idx_cats_balance_calc ON cats(asset_id, owner_puzzle_hash, amount)
  INCLUDE (coin_id, created_block);

-- Index for CAT holder analysis by asset
CREATE INDEX IF NOT EXISTS idx_cats_holder_analysis ON cats(asset_id, amount DESC)
  INCLUDE (owner_puzzle_hash, coin_id, created_block);

-- Index for CAT asset summary aggregations
CREATE INDEX IF NOT EXISTS idx_cats_asset_summary ON cats(asset_id, owner_puzzle_hash)
  INCLUDE (amount, created_block);

-- =============================================================================
-- NFT Analytics Optimization Indexes
-- =============================================================================

-- Index for NFT owner analysis with encoded puzzle hash
CREATE INDEX IF NOT EXISTS idx_nfts_owner_encoded ON nfts(encode(owner_puzzle_hash, 'hex'), collection_id, created_block)
  INCLUDE (coin_id, launcher_id, edition_number);

-- Index for NFT collection analysis
CREATE INDEX IF NOT EXISTS idx_nfts_collection_analysis ON nfts(collection_id, edition_number, created_block)
  INCLUDE (coin_id, launcher_id, owner_puzzle_hash);

-- Index for NFT ownership history analysis by owner
CREATE INDEX IF NOT EXISTS idx_nft_history_owner_encoded ON nft_ownership_history(encode(new_owner_puzzle_hash, 'hex'), block_height DESC)
  INCLUDE (launcher_id, coin_id, collection_id, transfer_type);

-- Index for NFT ownership history analysis by previous owner
CREATE INDEX IF NOT EXISTS idx_nft_history_prev_owner_encoded ON nft_ownership_history(encode(previous_owner_puzzle_hash, 'hex'), block_height DESC)
  INCLUDE (launcher_id, coin_id, collection_id, transfer_type);

-- =============================================================================
-- Aggregation and Window Function Optimization Indexes
-- =============================================================================

-- Index for COUNT(DISTINCT) optimizations in analytics
CREATE INDEX IF NOT EXISTS idx_distinct_count_optimization ON coins(puzzle_hash, coin_id, created_block);

-- Index for SUM aggregations in balance analytics
CREATE INDEX IF NOT EXISTS idx_sum_aggregation_optimization ON coins(puzzle_hash, amount)
  INCLUDE (coin_id, created_block);

-- Index for ranking and window function optimization
CREATE INDEX IF NOT EXISTS idx_ranking_optimization ON coins(amount DESC, puzzle_hash)
  INCLUDE (coin_id, created_block);

-- Index for ROW_NUMBER() optimization in analytics
CREATE INDEX IF NOT EXISTS idx_row_number_optimization ON spends(spent_block, coin_id)
  INCLUDE (puzzle_id, solution_id);

-- =============================================================================
-- Materialized View Refresh Optimization Indexes
-- =============================================================================

-- Index for XCH balances materialized view refresh optimization
CREATE INDEX IF NOT EXISTS idx_xch_balances_refresh ON coins(puzzle_hash, amount, coin_id)
  WHERE NOT EXISTS (SELECT 1 FROM spends WHERE spends.coin_id = coins.coin_id);

-- Index for CAT balances materialized view refresh optimization
CREATE INDEX IF NOT EXISTS idx_cat_balances_refresh ON cats(asset_id, owner_puzzle_hash, amount)
  INCLUDE (coin_id);

-- =============================================================================
-- Date/Time Function Optimization for Analytics
-- =============================================================================

-- Index for DATE() function optimization in temporal analytics
CREATE INDEX IF NOT EXISTS idx_date_function_optimization ON blocks(DATE(timestamp), height, timestamp);

-- Index for EXTRACT(HOUR) function optimization
CREATE INDEX IF NOT EXISTS idx_hour_extract_optimization ON blocks(EXTRACT(HOUR FROM timestamp), DATE(timestamp), height);

-- Index for EXTRACT(DOW) function optimization (day of week)
CREATE INDEX IF NOT EXISTS idx_dow_extract_optimization ON blocks(EXTRACT(DOW FROM timestamp), DATE(timestamp), height);

-- Index for EXTRACT(MONTH) function optimization
CREATE INDEX IF NOT EXISTS idx_month_extract_optimization ON blocks(EXTRACT(MONTH FROM timestamp), EXTRACT(YEAR FROM timestamp), height);

-- =============================================================================
-- Analytical Query Pattern Optimization
-- =============================================================================

-- Index for MIN/MAX aggregations in analytics
CREATE INDEX IF NOT EXISTS idx_min_max_optimization ON coins(puzzle_hash, amount, created_block);

-- Index for AVG calculations in analytics
CREATE INDEX IF NOT EXISTS idx_avg_calculation_optimization ON coins(puzzle_hash, amount)
  INCLUDE (coin_id, created_block);

-- Index for PERCENTILE functions in analytics
CREATE INDEX IF NOT EXISTS idx_percentile_optimization ON coins(amount, puzzle_hash)
  INCLUDE (coin_id, created_block);

-- Index for STDDEV calculations in analytics
CREATE INDEX IF NOT EXISTS idx_stddev_optimization ON puzzles(reveal_size)
  INCLUDE (id, puzzle_hash, first_seen_block);

-- =============================================================================
-- Cross-Database Pattern Optimization
-- =============================================================================

-- Index for encode() function pattern optimization
CREATE INDEX IF NOT EXISTS idx_encode_pattern_coins ON coins(encode(puzzle_hash, 'hex'), created_block, amount);
CREATE INDEX IF NOT EXISTS idx_encode_pattern_cats ON cats(encode(owner_puzzle_hash, 'hex'), asset_id, amount);
CREATE INDEX IF NOT EXISTS idx_encode_pattern_nfts ON nfts(encode(owner_puzzle_hash, 'hex'), collection_id, created_block);

-- Index for decode() function pattern optimization
CREATE INDEX IF NOT EXISTS idx_decode_pattern_optimization ON coins(puzzle_hash)
  INCLUDE (coin_id, amount, created_block);

-- =============================================================================
-- Comments and Usage Notes
-- =============================================================================

-- These indexes are specifically designed to optimize:
-- 1. Address analytics functions (behavior patterns, concentration, activity)
-- 2. Network analytics functions (growth metrics, peak activity, UTXO analysis)
-- 3. Transaction analytics functions (velocity, patterns, consolidation)
-- 4. Temporal analytics functions (trends, evolution, heatmaps)
-- 5. Cross-table analytical queries with complex JOINs
-- 6. Aggregation functions (COUNT, SUM, AVG, MIN, MAX, PERCENTILE)
-- 7. Window functions and ranking operations
-- 8. Materialized view refresh operations
-- 9. Date/time function optimizations
-- 10. Database-specific function patterns (encode/decode)
--
-- These indexes complement the existing temporal indexes and should provide
-- significant performance improvements for all analytics queries in the
-- functions modules. 