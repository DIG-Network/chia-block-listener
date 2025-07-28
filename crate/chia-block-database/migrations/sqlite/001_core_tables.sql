-- =============================================================================
-- Migration 001: Core Blockchain Tables (SQLite)
-- =============================================================================
-- Creates the fundamental blockchain data tables: blocks, coins, puzzles, spends,
-- and sync_status

-- Create blocks table
CREATE TABLE IF NOT EXISTS blocks (
  height INTEGER PRIMARY KEY,
  weight INTEGER NOT NULL,
  header_hash BLOB NOT NULL,
  timestamp TEXT NOT NULL
);

-- Create coins table with proper indexes
CREATE TABLE IF NOT EXISTS coins (
  coin_id TEXT PRIMARY KEY,
  parent_coin_info BLOB NOT NULL,
  puzzle_hash BLOB NOT NULL,
  amount INTEGER NOT NULL, -- SQLite INTEGER supports 64-bit signed integers
  created_block INTEGER
  -- Removed foreign key constraint: FOREIGN KEY (created_block) REFERENCES blocks(height) ON DELETE CASCADE
);

-- Create puzzles table for normalized puzzle reveals
CREATE TABLE IF NOT EXISTS puzzles (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  puzzle_hash BLOB NOT NULL UNIQUE,
  puzzle_reveal BLOB, -- Can be NULL if we don't have the reveal yet
  reveal_size INTEGER, -- Track size for analytics
  first_seen_block INTEGER -- Track when first encountered
);

-- Create solutions table for normalized solutions
CREATE TABLE IF NOT EXISTS solutions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  solution_hash BLOB NOT NULL UNIQUE,
  solution BLOB, -- Can be NULL if we don't have the solution yet
  solution_size INTEGER, -- Track size for analytics
  first_seen_block INTEGER -- Track when first encountered
);

-- Create spends table
CREATE TABLE IF NOT EXISTS spends (
  coin_id TEXT PRIMARY KEY,
  puzzle_id INTEGER, -- References puzzles table, can be NULL if no reveal yet
  solution_id INTEGER, -- References solutions table, can be NULL if no solution yet
  spent_block INTEGER NOT NULL
  -- Removed foreign key constraints:
  -- FOREIGN KEY (coin_id) REFERENCES coins(coin_id) ON DELETE CASCADE,
  -- FOREIGN KEY (spent_block) REFERENCES blocks(height) ON DELETE CASCADE,
  -- FOREIGN KEY (puzzle_id) REFERENCES puzzles(id) ON DELETE SET NULL,
  -- FOREIGN KEY (solution_id) REFERENCES solutions(id) ON DELETE SET NULL
);

-- Create sync_status table for monitoring
CREATE TABLE IF NOT EXISTS sync_status (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  sync_type TEXT NOT NULL, -- 'historical', 'realtime', 'combined'
  current_peak_height INTEGER NOT NULL DEFAULT 0,
  last_synced_height INTEGER NOT NULL DEFAULT 0,
  start_height INTEGER NOT NULL DEFAULT 0,
  queue_size INTEGER NOT NULL DEFAULT 0,
  active_downloads INTEGER NOT NULL DEFAULT 0,
  is_running INTEGER NOT NULL DEFAULT 0, -- SQLite uses INTEGER for BOOLEAN
  is_historical_sync INTEGER NOT NULL DEFAULT 0,
  last_activity TEXT NOT NULL DEFAULT (datetime('now')),
  blocks_processed_session INTEGER NOT NULL DEFAULT 0,
  session_start_time TEXT NOT NULL DEFAULT (datetime('now')),
  total_blocks_synced INTEGER NOT NULL DEFAULT 0,
  sync_speed_blocks_per_sec REAL NOT NULL DEFAULT 0,
  progress_percentage REAL NOT NULL DEFAULT 0,
  eta_minutes INTEGER,
  errors_count INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  UNIQUE(sync_type)
);

-- Initialize sync status if it doesn't exist
INSERT OR IGNORE INTO sync_status (sync_type, sync_speed_blocks_per_sec, progress_percentage)
VALUES ('combined', 0, 0);

-- =============================================================================
-- System Preferences Table - Store application-wide configuration
-- =============================================================================
CREATE TABLE IF NOT EXISTS system_preferences (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  meta_key TEXT NOT NULL UNIQUE,
  meta_value TEXT NOT NULL,
  description TEXT,
  created_at TEXT DEFAULT (datetime('now')),
  updated_at TEXT DEFAULT (datetime('now'))
);

-- Insert default system preferences
INSERT OR IGNORE INTO system_preferences (meta_key, meta_value, description) VALUES 
  ('paused_sync', 'true', 'When true, pauses all blockchain synchronization processes (gap sync and live sync)'),
  ('exit_process', 'false', 'When true, immediately exits the application process (kill switch)');

-- =============================================================================
-- Additional Indexes for Table Relationships (replacing foreign keys)
-- =============================================================================

-- Indexes to maintain coins -> blocks relationship
CREATE INDEX IF NOT EXISTS idx_coins_created_block_ref ON coins(created_block);

-- Indexes to maintain spends -> coins relationship  
CREATE INDEX IF NOT EXISTS idx_spends_coin_id_ref ON spends(coin_id);

-- Indexes to maintain spends -> blocks relationship
CREATE INDEX IF NOT EXISTS idx_spends_spent_block_ref ON spends(spent_block);

-- Indexes to maintain spends -> puzzles relationship
CREATE INDEX IF NOT EXISTS idx_spends_puzzle_id_ref ON spends(puzzle_id);

-- Indexes to maintain spends -> solutions relationship  
CREATE INDEX IF NOT EXISTS idx_spends_solution_id_ref ON spends(solution_id);

-- Composite indexes for efficient joins
CREATE INDEX IF NOT EXISTS idx_coins_blocks_join ON coins(created_block, coin_id);
CREATE INDEX IF NOT EXISTS idx_spends_coins_join ON spends(coin_id, spent_block);
CREATE INDEX IF NOT EXISTS idx_spends_blocks_join ON spends(spent_block, coin_id);
CREATE INDEX IF NOT EXISTS idx_spends_puzzles_join ON spends(puzzle_id, coin_id);
CREATE INDEX IF NOT EXISTS idx_spends_solutions_join ON spends(solution_id, coin_id); 