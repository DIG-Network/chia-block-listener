-- =============================================================================
-- Migration 002: Performance Indexes
-- =============================================================================
-- Creates comprehensive indexes for optimal query performance across all core tables
-- All indexes are optimized for pagination patterns used throughout the application

-- =============================================================================
-- Blocks Indexes
-- =============================================================================
CREATE INDEX IF NOT EXISTS idx_blocks_height ON blocks(height);
CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks(timestamp);
CREATE INDEX IF NOT EXISTS idx_blocks_weight ON blocks(weight);
CREATE INDEX IF NOT EXISTS idx_blocks_height_timestamp ON blocks(height, timestamp);
CREATE INDEX IF NOT EXISTS idx_blocks_timestamp_height ON blocks(timestamp, height);

-- Index for efficient block range queries (historical sync)
CREATE INDEX IF NOT EXISTS idx_blocks_height_range ON blocks(height) 
  INCLUDE (header_hash, timestamp, weight);

-- =============================================================================
-- Coins Indexes - Critical for Performance
-- =============================================================================
CREATE INDEX IF NOT EXISTS idx_coins_puzzle_hash ON coins(puzzle_hash);
CREATE INDEX IF NOT EXISTS idx_coins_parent_coin_info ON coins(parent_coin_info);
CREATE INDEX IF NOT EXISTS idx_coins_amount ON coins(amount);
CREATE INDEX IF NOT EXISTS idx_coins_created_block ON coins(created_block);

-- Composite indexes for common queries
CREATE INDEX IF NOT EXISTS idx_coins_puzzle_hash_amount ON coins(puzzle_hash, amount);
CREATE INDEX IF NOT EXISTS idx_coins_created_block_puzzle_hash ON coins(created_block, puzzle_hash);
CREATE INDEX IF NOT EXISTS idx_coins_created_block_amount ON coins(created_block, amount);

-- Optimized index for balance queries (puzzle_hash + amount DESC for efficient coin selection)
CREATE INDEX IF NOT EXISTS idx_coins_puzzle_hash_amount_desc ON coins(puzzle_hash, amount DESC);

-- Covering index for coins table to avoid table lookups in balance queries
CREATE INDEX IF NOT EXISTS idx_coins_puzzle_hash_covering ON coins(puzzle_hash) 
  INCLUDE (coin_id, amount, created_block, parent_coin_info);

-- Composite index for unspent coin lookups (major performance boost)
CREATE INDEX IF NOT EXISTS idx_coins_unspent_lookup ON coins(puzzle_hash, amount DESC, coin_id);

-- Index for materialized view refresh operations
CREATE INDEX IF NOT EXISTS idx_coins_mv_refresh ON coins(puzzle_hash, amount, coin_id);

-- Index for efficient coin parent-child relationships
CREATE INDEX IF NOT EXISTS idx_coins_parent_relationships ON coins(parent_coin_info, puzzle_hash, amount);

-- Optimized index for get_children_coins pagination (ORDER BY created_block, amount DESC)
CREATE INDEX IF NOT EXISTS idx_coins_parent_coin_info_created_block_amount_desc ON coins(parent_coin_info, created_block, amount DESC);

-- Covering index for balance calculations (avoids table lookups)  
CREATE INDEX IF NOT EXISTS idx_coins_balance_calc_covering ON coins(puzzle_hash, amount, created_block) 
  INCLUDE (coin_id, parent_coin_info);

-- Optimized for the unspent_coins view performance
CREATE INDEX IF NOT EXISTS idx_coins_unspent_view_opt ON coins(coin_id, puzzle_hash, amount, created_block, parent_coin_info);

-- Indexes for foreign key constraints and CASCADE operations
CREATE INDEX IF NOT EXISTS idx_coins_created_block_fk ON coins(created_block);

-- =============================================================================
-- Puzzles Indexes
-- =============================================================================
CREATE INDEX IF NOT EXISTS idx_puzzles_puzzle_hash ON puzzles(puzzle_hash);
CREATE INDEX IF NOT EXISTS idx_puzzles_first_seen_block ON puzzles(first_seen_block);
CREATE INDEX IF NOT EXISTS idx_puzzles_reveal_size ON puzzles(reveal_size);

-- Partial index for puzzles with reveals (optimizes get_common_puzzles and get_puzzle_stats)
CREATE INDEX IF NOT EXISTS idx_puzzles_with_reveals ON puzzles(reveal_size, first_seen_block) 
  WHERE puzzle_reveal IS NOT NULL;

-- Index for puzzle reveal lookups by size (analytics)
CREATE INDEX IF NOT EXISTS idx_puzzles_reveal_size_desc ON puzzles(reveal_size DESC) 
  WHERE puzzle_reveal IS NOT NULL;

-- Index for efficient puzzle normalization lookups  
CREATE INDEX IF NOT EXISTS idx_puzzles_hash_reveal_size ON puzzles(puzzle_hash, reveal_size, id);

-- Optimized index for large puzzle reveal queries
CREATE INDEX IF NOT EXISTS idx_puzzles_large_reveals ON puzzles(reveal_size DESC, id) 
  WHERE reveal_size > 1024;

-- Temporal index for time-based analytics
CREATE INDEX IF NOT EXISTS idx_puzzles_first_seen_block_reveal_size ON puzzles(first_seen_block, reveal_size) 
  WHERE puzzle_reveal IS NOT NULL;

-- Optimized index for get_common_puzzles pagination (supporting complex ORDER BY with computed usage_count)
-- This index helps with the JOIN to spends table and provides reveal_size for secondary sort
CREATE INDEX IF NOT EXISTS idx_puzzles_reveal_size_id_with_reveals ON puzzles(reveal_size DESC, id) 
  WHERE puzzle_reveal IS NOT NULL;

-- =============================================================================
-- Solutions Indexes
-- =============================================================================
CREATE INDEX IF NOT EXISTS idx_solutions_solution_hash ON solutions(solution_hash);
CREATE INDEX IF NOT EXISTS idx_solutions_first_seen_block ON solutions(first_seen_block);
CREATE INDEX IF NOT EXISTS idx_solutions_solution_size ON solutions(solution_size);

-- Partial index for solutions with data (optimizes solution analytics)
CREATE INDEX IF NOT EXISTS idx_solutions_with_data ON solutions(solution_size, first_seen_block) 
  WHERE solution IS NOT NULL;

-- Index for solution lookups by size (analytics)
CREATE INDEX IF NOT EXISTS idx_solutions_size_desc ON solutions(solution_size DESC) 
  WHERE solution IS NOT NULL;

-- Index for efficient solution normalization lookups  
CREATE INDEX IF NOT EXISTS idx_solutions_hash_size ON solutions(solution_hash, solution_size, id);

-- Optimized index for large solution queries
CREATE INDEX IF NOT EXISTS idx_solutions_large_data ON solutions(solution_size DESC, id) 
  WHERE solution_size > 1024;

-- Temporal index for time-based analytics
CREATE INDEX IF NOT EXISTS idx_solutions_first_seen_block_size ON solutions(first_seen_block, solution_size) 
  WHERE solution IS NOT NULL;

-- =============================================================================
-- Spends Indexes
-- =============================================================================
CREATE INDEX IF NOT EXISTS idx_spends_spent_block ON spends(spent_block);
CREATE INDEX IF NOT EXISTS idx_spends_coin_id_spent_block ON spends(coin_id, spent_block);
CREATE INDEX IF NOT EXISTS idx_spends_puzzle_id ON spends(puzzle_id);
CREATE INDEX IF NOT EXISTS idx_spends_solution_id ON spends(solution_id);
CREATE INDEX IF NOT EXISTS idx_spends_spent_block_puzzle_id ON spends(spent_block, puzzle_id);
CREATE INDEX IF NOT EXISTS idx_spends_spent_block_solution_id ON spends(spent_block, solution_id);

-- Add specialized index for LEFT JOIN performance on spends(coin_id) for unspent coins query
CREATE INDEX IF NOT EXISTS idx_spends_coin_id_covering ON spends(coin_id) INCLUDE (spent_block);

-- Composite index for spends joins in puzzle analytics
CREATE INDEX IF NOT EXISTS idx_spends_puzzle_id_coin_id ON spends(puzzle_id, coin_id);

-- Composite index for spends joins in solution analytics
CREATE INDEX IF NOT EXISTS idx_spends_solution_id_coin_id ON spends(solution_id, coin_id);

-- Covering index for spends analytics
CREATE INDEX IF NOT EXISTS idx_spends_analytics_covering ON spends(puzzle_id, solution_id, spent_block) 
  INCLUDE (coin_id);

-- Performance index for puzzle usage frequency analysis  
CREATE INDEX IF NOT EXISTS idx_spends_puzzle_frequency ON spends(puzzle_id) 
  INCLUDE (coin_id, spent_block);

-- Performance index for solution usage frequency analysis  
CREATE INDEX IF NOT EXISTS idx_spends_solution_frequency ON spends(solution_id) 
  INCLUDE (coin_id, spent_block);

-- Index for foreign key constraints
CREATE INDEX IF NOT EXISTS idx_spends_spent_block_fk ON spends(spent_block);
CREATE INDEX IF NOT EXISTS idx_spends_solution_id_fk ON spends(solution_id);

-- =============================================================================
-- Sync Status Indexes
-- =============================================================================
CREATE INDEX IF NOT EXISTS idx_sync_status_type ON sync_status(sync_type);

-- Index for efficient sync status operations
CREATE INDEX IF NOT EXISTS idx_sync_status_activity ON sync_status(last_activity, sync_type, is_running);

-- =============================================================================
-- System Preferences Indexes
-- =============================================================================
CREATE UNIQUE INDEX IF NOT EXISTS idx_system_preferences_meta_key ON system_preferences(meta_key);
CREATE INDEX IF NOT EXISTS idx_system_preferences_updated_at ON system_preferences(updated_at); 