-- =============================================================================
-- Migration 001: Core Blockchain Tables
-- =============================================================================
-- Creates the fundamental blockchain data tables: blocks, coins, puzzles, spends,
-- and sync_status

-- Create blocks table
CREATE TABLE IF NOT EXISTS blocks (
  height BIGINT PRIMARY KEY,
  weight BIGINT NOT NULL,
  header_hash BYTEA NOT NULL,
  timestamp TIMESTAMP NOT NULL
);

-- Create coins table with proper indexes
CREATE TABLE IF NOT EXISTS coins (
  coin_id VARCHAR(64) PRIMARY KEY,
  parent_coin_info BYTEA NOT NULL,
  puzzle_hash BYTEA NOT NULL,
  amount NUMERIC(20,0) NOT NULL, -- Supports up to 20 digits (more than enough for uint64 max: 18446744073709551615)
  created_block BIGINT
  -- Removed foreign key constraint: FOREIGN KEY (created_block) REFERENCES blocks(height) ON DELETE CASCADE
);

-- Create puzzles table for normalized puzzle reveals
CREATE TABLE IF NOT EXISTS puzzles (
  id SERIAL PRIMARY KEY,
  puzzle_hash BYTEA NOT NULL UNIQUE,
  puzzle_reveal BYTEA, -- Can be NULL if we don't have the reveal yet
  reveal_size INTEGER, -- Track size for analytics
  first_seen_block BIGINT -- Track when first encountered
);

-- Create solutions table for normalized solutions
CREATE TABLE IF NOT EXISTS solutions (
  id SERIAL PRIMARY KEY,
  solution_hash BYTEA NOT NULL UNIQUE,
  solution BYTEA, -- Can be NULL if we don't have the solution yet
  solution_size INTEGER, -- Track size for analytics
  first_seen_block BIGINT -- Track when first encountered
);

-- Create spends table
CREATE TABLE IF NOT EXISTS spends (
  coin_id VARCHAR(64) PRIMARY KEY,
  puzzle_id INTEGER, -- References puzzles table, can be NULL if no reveal yet
  solution_id INTEGER, -- References solutions table, can be NULL if no solution yet
  spent_block BIGINT NOT NULL
  -- Removed foreign key constraints:
  -- FOREIGN KEY (coin_id) REFERENCES coins(coin_id) ON DELETE CASCADE,
  -- FOREIGN KEY (spent_block) REFERENCES blocks(height) ON DELETE CASCADE,
  -- FOREIGN KEY (puzzle_id) REFERENCES puzzles(id) ON DELETE SET NULL,
  -- FOREIGN KEY (solution_id) REFERENCES solutions(id) ON DELETE SET NULL
);

-- Create sync_status table for monitoring
CREATE TABLE IF NOT EXISTS sync_status (
  id SERIAL PRIMARY KEY,
  sync_type VARCHAR(20) NOT NULL, -- 'historical', 'realtime', 'combined'
  current_peak_height BIGINT NOT NULL DEFAULT 0,
  last_synced_height BIGINT NOT NULL DEFAULT 0,
  start_height BIGINT NOT NULL DEFAULT 0,
  queue_size INTEGER NOT NULL DEFAULT 0,
  active_downloads INTEGER NOT NULL DEFAULT 0,
  is_running BOOLEAN NOT NULL DEFAULT false,
  is_historical_sync BOOLEAN NOT NULL DEFAULT false,
  last_activity TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  blocks_processed_session INTEGER NOT NULL DEFAULT 0,
  session_start_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  total_blocks_synced BIGINT NOT NULL DEFAULT 0,
  sync_speed_blocks_per_sec DECIMAL(10,2) NOT NULL DEFAULT 0,
  progress_percentage DECIMAL(5,2) NOT NULL DEFAULT 0,
  eta_minutes INTEGER,
  errors_count INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  UNIQUE(sync_type)
);

-- Initialize sync status if it doesn't exist
INSERT INTO sync_status (sync_type, sync_speed_blocks_per_sec, progress_percentage)
VALUES ('combined', 0, 0)
ON CONFLICT (sync_type) DO NOTHING;

-- =============================================================================
-- System Preferences Table - Store application-wide configuration
-- =============================================================================
CREATE TABLE IF NOT EXISTS system_preferences (
  id SERIAL PRIMARY KEY,
  meta_key VARCHAR(255) NOT NULL UNIQUE,
  meta_value TEXT NOT NULL,
  description TEXT,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

-- Insert default system preferences
INSERT INTO system_preferences (meta_key, meta_value, description) VALUES 
  ('paused_sync', 'true', 'When true, pauses all blockchain synchronization processes (gap sync and live sync)'),
  ('exit_process', 'false', 'When true, immediately exits the application process (kill switch)')
ON CONFLICT (meta_key) DO NOTHING;

-- =============================================================================
-- Additional Indexes for Table Relationships (replacing foreign keys)
-- =============================================================================

-- Indexes to maintain coins -> blocks relationship
CREATE INDEX IF NOT EXISTS idx_coins_created_block_ref ON coins(created_block);
CREATE INDEX IF NOT EXISTS idx_coins_created_block_exists ON coins(created_block) WHERE created_block IS NOT NULL;

-- Indexes to maintain spends -> coins relationship  
CREATE INDEX IF NOT EXISTS idx_spends_coin_id_ref ON spends(coin_id);

-- Indexes to maintain spends -> blocks relationship
CREATE INDEX IF NOT EXISTS idx_spends_spent_block_ref ON spends(spent_block);

-- Indexes to maintain spends -> puzzles relationship
CREATE INDEX IF NOT EXISTS idx_spends_puzzle_id_ref ON spends(puzzle_id) WHERE puzzle_id IS NOT NULL;

-- Indexes to maintain spends -> solutions relationship  
CREATE INDEX IF NOT EXISTS idx_spends_solution_id_ref ON spends(solution_id) WHERE solution_id IS NOT NULL;

-- Composite indexes for efficient joins
CREATE INDEX IF NOT EXISTS idx_coins_blocks_join ON coins(created_block, coin_id) WHERE created_block IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_spends_coins_join ON spends(coin_id, spent_block);
CREATE INDEX IF NOT EXISTS idx_spends_blocks_join ON spends(spent_block, coin_id);
CREATE INDEX IF NOT EXISTS idx_spends_puzzles_join ON spends(puzzle_id, coin_id) WHERE puzzle_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_spends_solutions_join ON spends(solution_id, coin_id) WHERE solution_id IS NOT NULL; 