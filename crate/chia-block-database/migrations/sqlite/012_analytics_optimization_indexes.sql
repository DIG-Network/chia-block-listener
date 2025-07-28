-- =============================================================================
-- Migration 012: Analytics Optimization Indexes (SQLite)
-- =============================================================================
-- Creates specialized indexes to optimize all analytics queries in the functions modules
-- These indexes fill gaps not covered by previous migrations, adapted for SQLite

-- =============================================================================
-- Address Analytics Optimization Indexes
-- =============================================================================

-- Index for address grouping with hex puzzle hash (used extensively in address analytics)
CREATE INDEX IF NOT EXISTS idx_coins_hex_puzzle_hash ON coins(hex(puzzle_hash), created_block, amount);

-- Index for address activity analysis with hex puzzle hash
CREATE INDEX IF NOT EXISTS idx_coins_address_analysis ON coins(hex(puzzle_hash), created_block, coin_id, amount);

-- Index for spend analysis by address (joining spends with coins on puzzle_hash)
CREATE INDEX IF NOT EXISTS idx_spends_address_analysis ON spends(spent_block, coin_id);

-- Composite index for address balance evolution tracking
CREATE INDEX IF NOT EXISTS idx_address_balance_tracking ON coins(hex(puzzle_hash), created_block, amount, coin_id);

-- Index for dormant address detection (last activity analysis)
CREATE INDEX IF NOT EXISTS idx_address_last_activity ON coins(puzzle_hash, created_block, amount, coin_id);

-- Index for new address discovery (MIN created_block analysis)
CREATE INDEX IF NOT EXISTS idx_new_address_discovery ON coins(created_block, hex(puzzle_hash));

-- =============================================================================
-- Network Analytics Optimization Indexes
-- =============================================================================

-- Index for network statistics aggregation (multiple table joins)
CREATE INDEX IF NOT EXISTS idx_network_stats_blocks ON blocks(height, weight, header_hash, timestamp);

-- Index for UTXO analysis optimization
CREATE INDEX IF NOT EXISTS idx_utxo_analysis_coins ON coins(coin_id, puzzle_hash, amount, created_block);

-- Index for unspent coin detection optimization (composite index for SQLite)
CREATE INDEX IF NOT EXISTS idx_unspent_detection ON coins(coin_id, puzzle_hash, amount, created_block);

-- Index for peak activity detection (hourly aggregations using datetime functions)
CREATE INDEX IF NOT EXISTS idx_peak_activity_hourly ON blocks(datetime(datetime(timestamp), 'start of day', '+' || strftime('%H', timestamp) || ' hours'), height, timestamp);

-- =============================================================================
-- Transaction Analytics Optimization Indexes
-- =============================================================================

-- Index for spend velocity analysis (spend_block - created_block calculations)
CREATE INDEX IF NOT EXISTS idx_spend_velocity_calc ON spends(spent_block, coin_id, puzzle_id, solution_id);

-- Index for transaction size distribution analysis
CREATE INDEX IF NOT EXISTS idx_transaction_size_analysis ON coins(amount, created_block, coin_id, puzzle_hash);

-- Index for complex transaction pattern analysis (block-level aggregations)
CREATE INDEX IF NOT EXISTS idx_complex_transaction_patterns ON spends(spent_block, coin_id, puzzle_id, solution_id);

-- Index for spending frequency by address analysis
CREATE INDEX IF NOT EXISTS idx_spending_frequency_analysis ON spends(spent_block, coin_id);

-- Index for coin consolidation pattern detection
CREATE INDEX IF NOT EXISTS idx_consolidation_patterns ON spends(spent_block, coin_id, puzzle_id, solution_id);

-- =============================================================================
-- Cross-Table Analytics Optimization Indexes
-- =============================================================================

-- Index for coins-spends join optimization in analytics
CREATE INDEX IF NOT EXISTS idx_coins_spends_analytics_join ON coins(coin_id, puzzle_hash, amount, created_block);

-- Index for blocks-coins join optimization in temporal analytics
CREATE INDEX IF NOT EXISTS idx_blocks_coins_temporal_join ON coins(created_block, puzzle_hash, coin_id, amount);

-- Index for spends-coins join optimization in transaction analytics
CREATE INDEX IF NOT EXISTS idx_spends_coins_transaction_join ON spends(coin_id, spent_block, puzzle_id, solution_id);

-- =============================================================================
-- Puzzle Analytics Optimization Indexes
-- =============================================================================

-- Index for puzzle usage frequency analysis
CREATE INDEX IF NOT EXISTS idx_puzzle_usage_frequency ON spends(puzzle_id, spent_block, coin_id);

-- Index for puzzle complexity analysis
CREATE INDEX IF NOT EXISTS idx_puzzle_complexity_analysis ON puzzles(reveal_size, first_seen_block, id, puzzle_hash);

-- Index for puzzle popularity analysis (coins with puzzle)
CREATE INDEX IF NOT EXISTS idx_puzzle_popularity ON coins(puzzle_hash, created_block, coin_id, amount);

-- Index for unspent puzzle analysis (composite index for SQLite)
CREATE INDEX IF NOT EXISTS idx_unspent_puzzle_analysis ON coins(puzzle_hash, amount, coin_id, created_block);

-- =============================================================================
-- Solution Analytics Optimization Indexes
-- =============================================================================

-- Index for solution usage frequency analysis
CREATE INDEX IF NOT EXISTS idx_solution_usage_frequency ON spends(solution_id, spent_block, coin_id);

-- Index for solution efficiency analysis
CREATE INDEX IF NOT EXISTS idx_solution_efficiency ON spends(solution_id, spent_block, coin_id);

-- Index for high-value solution analysis
CREATE INDEX IF NOT EXISTS idx_high_value_solutions ON spends(solution_id, coin_id, spent_block);

-- =============================================================================
-- Balance Analytics Optimization Indexes
-- =============================================================================

-- Index for balance distribution analysis (composite for SQLite)
CREATE INDEX IF NOT EXISTS idx_balance_distribution ON coins(amount, puzzle_hash, coin_id, created_block);

-- Index for richest addresses analysis
CREATE INDEX IF NOT EXISTS idx_richest_addresses ON coins(puzzle_hash, amount DESC, coin_id, created_block);

-- Index for balance changes analysis
CREATE INDEX IF NOT EXISTS idx_balance_changes ON coins(puzzle_hash, created_block, amount);

-- Index for address balance range queries
CREATE INDEX IF NOT EXISTS idx_address_balance_range ON coins(amount, puzzle_hash, coin_id, created_block);

-- =============================================================================
-- CAT Analytics Optimization Indexes
-- =============================================================================

-- Index for CAT owner analysis with hex puzzle hash
CREATE INDEX IF NOT EXISTS idx_cats_owner_hex ON cats(hex(owner_puzzle_hash), asset_id, created_block, coin_id, amount);

-- Index for CAT balance calculation optimization
CREATE INDEX IF NOT EXISTS idx_cats_balance_calc ON cats(asset_id, owner_puzzle_hash, amount, coin_id, created_block);

-- Index for CAT holder analysis by asset
CREATE INDEX IF NOT EXISTS idx_cats_holder_analysis ON cats(asset_id, amount DESC, owner_puzzle_hash, coin_id, created_block);

-- Index for CAT asset summary aggregations
CREATE INDEX IF NOT EXISTS idx_cats_asset_summary ON cats(asset_id, owner_puzzle_hash, amount, created_block);

-- =============================================================================
-- NFT Analytics Optimization Indexes
-- =============================================================================

-- Index for NFT owner analysis with hex puzzle hash
CREATE INDEX IF NOT EXISTS idx_nfts_owner_hex ON nfts(hex(owner_puzzle_hash), collection_id, created_block, coin_id, launcher_id, edition_number);

-- Index for NFT collection analysis
CREATE INDEX IF NOT EXISTS idx_nfts_collection_analysis ON nfts(collection_id, edition_number, created_block, coin_id, launcher_id, owner_puzzle_hash);

-- Index for NFT ownership history analysis by owner (using hex function)
CREATE INDEX IF NOT EXISTS idx_nft_history_owner_hex ON nft_ownership_history(hex(new_owner_puzzle_hash), block_height DESC, launcher_id, coin_id, collection_id, transfer_type);

-- Index for NFT ownership history analysis by previous owner
CREATE INDEX IF NOT EXISTS idx_nft_history_prev_owner_hex ON nft_ownership_history(hex(previous_owner_puzzle_hash), block_height DESC, launcher_id, coin_id, collection_id, transfer_type);

-- =============================================================================
-- Aggregation and Window Function Optimization Indexes
-- =============================================================================

-- Index for COUNT(DISTINCT) optimizations in analytics
CREATE INDEX IF NOT EXISTS idx_distinct_count_optimization ON coins(puzzle_hash, coin_id, created_block);

-- Index for SUM aggregations in balance analytics
CREATE INDEX IF NOT EXISTS idx_sum_aggregation_optimization ON coins(puzzle_hash, amount, coin_id, created_block);

-- Index for ranking and sorting optimization
CREATE INDEX IF NOT EXISTS idx_ranking_optimization ON coins(amount DESC, puzzle_hash, coin_id, created_block);

-- Index for ROW_NUMBER() optimization in analytics
CREATE INDEX IF NOT EXISTS idx_row_number_optimization ON spends(spent_block, coin_id, puzzle_id, solution_id);

-- =============================================================================
-- Date/Time Function Optimization for Analytics
-- =============================================================================

-- Index for DATE() function optimization in temporal analytics
CREATE INDEX IF NOT EXISTS idx_date_function_optimization ON blocks(DATE(timestamp), height, timestamp);

-- Index for strftime('%H') function optimization (hour extraction)
CREATE INDEX IF NOT EXISTS idx_hour_extract_optimization ON blocks(strftime('%H', timestamp), DATE(timestamp), height);

-- Index for strftime('%w') function optimization (day of week)
CREATE INDEX IF NOT EXISTS idx_dow_extract_optimization ON blocks(strftime('%w', timestamp), DATE(timestamp), height);

-- Index for strftime('%m') function optimization (month)
CREATE INDEX IF NOT EXISTS idx_month_extract_optimization ON blocks(strftime('%m', timestamp), strftime('%Y', timestamp), height);

-- Index for strftime('%d') function optimization (day of month)
CREATE INDEX IF NOT EXISTS idx_day_extract_optimization ON blocks(strftime('%d', timestamp), DATE(timestamp), height);

-- =============================================================================
-- SQLite-Specific Function Optimizations
-- =============================================================================

-- Index for julianday() calculations in time analysis
CREATE INDEX IF NOT EXISTS idx_julianday_optimization ON blocks(julianday(timestamp), height);

-- Index for datetime() calculations
CREATE INDEX IF NOT EXISTS idx_datetime_optimization ON blocks(datetime(timestamp), height);

-- Index for hex() function optimization on coins
CREATE INDEX IF NOT EXISTS idx_hex_function_coins ON coins(hex(puzzle_hash), created_block, amount);

-- Index for hex() function optimization on cats
CREATE INDEX IF NOT EXISTS idx_hex_function_cats ON cats(hex(owner_puzzle_hash), asset_id, amount);

-- Index for hex() function optimization on nfts
CREATE INDEX IF NOT EXISTS idx_hex_function_nfts ON nfts(hex(owner_puzzle_hash), collection_id, created_block);

-- Index for hex() function optimization on nft ownership history
CREATE INDEX IF NOT EXISTS idx_hex_function_nft_history ON nft_ownership_history(hex(new_owner_puzzle_hash), block_height);

-- =============================================================================
-- Analytical Query Pattern Optimization
-- =============================================================================

-- Index for MIN/MAX aggregations in analytics
CREATE INDEX IF NOT EXISTS idx_min_max_optimization ON coins(puzzle_hash, amount, created_block);

-- Index for AVG calculations in analytics
CREATE INDEX IF NOT EXISTS idx_avg_calculation_optimization ON coins(puzzle_hash, amount, coin_id, created_block);

-- Index for sorting and ordering optimizations
CREATE INDEX IF NOT EXISTS idx_sorting_optimization ON coins(amount, puzzle_hash, coin_id, created_block);

-- Index for solution complexity analysis (SQLite doesn't have STDDEV, but we can optimize size analysis)
CREATE INDEX IF NOT EXISTS idx_solution_complexity_optimization ON solutions(solution_size, id, solution_hash, first_seen_block);

-- Index for puzzle complexity analysis
CREATE INDEX IF NOT EXISTS idx_puzzle_complexity_optimization ON puzzles(reveal_size, id, puzzle_hash, first_seen_block);

-- =============================================================================
-- Balance and Activity Table Optimization
-- =============================================================================

-- Index for XCH balances table refresh optimization
CREATE INDEX IF NOT EXISTS idx_xch_balances_refresh_optimization ON coins(puzzle_hash, amount, coin_id);

-- Index for CAT balances table refresh optimization
CREATE INDEX IF NOT EXISTS idx_cat_balances_refresh_optimization ON cats(asset_id, owner_puzzle_hash, amount, coin_id);

-- =============================================================================
-- Cross-Reference and Join Optimization
-- =============================================================================

-- Index for spends-puzzles join optimization
CREATE INDEX IF NOT EXISTS idx_spends_puzzles_join_opt ON spends(puzzle_id, coin_id, spent_block);

-- Index for spends-solutions join optimization
CREATE INDEX IF NOT EXISTS idx_spends_solutions_join_opt ON spends(solution_id, coin_id, spent_block);

-- Index for coins-blocks join optimization
CREATE INDEX IF NOT EXISTS idx_coins_blocks_join_opt ON coins(created_block, puzzle_hash, amount, coin_id);

-- Index for cats-spends join optimization (for spent CAT analysis)
CREATE INDEX IF NOT EXISTS idx_cats_spends_join_opt ON cats(coin_id, asset_id, owner_puzzle_hash, amount);

-- Index for nfts-spends join optimization (for spent NFT analysis)
CREATE INDEX IF NOT EXISTS idx_nfts_spends_join_opt ON nfts(coin_id, launcher_id, owner_puzzle_hash, collection_id);

-- =============================================================================
-- Comments and Usage Notes
-- =============================================================================

-- These SQLite-specific indexes are designed to optimize:
-- 1. Address analytics functions using hex() instead of encode()
-- 2. Network analytics functions with SQLite date/time functions
-- 3. Transaction analytics functions with composite indexes
-- 4. Temporal analytics functions using strftime() and datetime()
-- 5. Cross-table analytical queries adapted for SQLite
-- 6. Aggregation functions optimized for SQLite's capabilities
-- 7. Window functions and sorting operations
-- 8. Table refresh operations for balance tables
-- 9. SQLite-specific function patterns (hex, julianday, datetime)
-- 10. Join optimizations using composite indexes instead of INCLUDE
--
-- Key SQLite adaptations:
-- - Uses hex() instead of encode() for binary-to-text conversion
-- - Uses composite indexes instead of INCLUDE clauses
-- - Uses strftime() instead of EXTRACT() for date/time parts
-- - Uses datetime() functions for date arithmetic
-- - Simplified partial index conditions due to SQLite limitations
-- - Uses julianday() for date calculations and comparisons 