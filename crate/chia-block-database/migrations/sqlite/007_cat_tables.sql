-- =============================================================================
-- Migration 007: CAT Tables and Balance Views (SQLite)
-- =============================================================================
-- Creates CAT-related tables: cat_assets, cats, cat_balances table with indexes

-- =============================================================================
-- CAT Tables
-- =============================================================================

-- Create CAT assets table for normalized CAT asset data
CREATE TABLE IF NOT EXISTS cat_assets (
  asset_id TEXT PRIMARY KEY, -- CAT asset ID as natural primary key
  symbol TEXT,
  name TEXT,
  description TEXT,
  total_supply INTEGER,
  decimals INTEGER DEFAULT 3, -- Most CATs use 3 decimals (mojos)
  created_at TEXT DEFAULT (datetime('now')),
  metadata_json TEXT, -- Store full asset metadata as JSON text
  first_seen_block INTEGER
);

-- Create CATs table for individual CAT coin records
CREATE TABLE IF NOT EXISTS cats (
  coin_id TEXT PRIMARY KEY,
  asset_id TEXT NOT NULL, -- Formerly REFERENCES cat_assets(asset_id)
  owner_puzzle_hash BLOB NOT NULL, -- p2PuzzleHash from CAT info
  inner_puzzle_hash BLOB NOT NULL, -- innerPuzzleHash from CAT info
  amount INTEGER NOT NULL, -- CAT amount
  lineage_proof TEXT, -- Store lineage proof data as JSON text
  created_block INTEGER,
  created_at TEXT DEFAULT (datetime('now'))
  
  -- Removed foreign key constraints:
  -- FOREIGN KEY (coin_id) REFERENCES coins(coin_id) ON DELETE CASCADE,
  -- FOREIGN KEY (created_block) REFERENCES blocks(height) ON DELETE CASCADE
);

-- =============================================================================
-- Additional Indexes for Table Relationships (replacing foreign keys)
-- =============================================================================

-- Indexes to maintain cats -> coins relationship
CREATE INDEX IF NOT EXISTS idx_cats_coin_id_ref ON cats(coin_id);

-- Indexes to maintain cats -> blocks relationship
CREATE INDEX IF NOT EXISTS idx_cats_created_block_ref ON cats(created_block);

-- Indexes to maintain cats -> cat_assets relationship
CREATE INDEX IF NOT EXISTS idx_cats_asset_id_ref ON cats(asset_id);

-- Composite indexes for efficient joins
CREATE INDEX IF NOT EXISTS idx_cats_coins_join ON cats(coin_id, asset_id);
CREATE INDEX IF NOT EXISTS idx_cats_blocks_join ON cats(created_block, coin_id);
CREATE INDEX IF NOT EXISTS idx_cats_assets_join ON cats(asset_id, owner_puzzle_hash);

-- =============================================================================
-- CAT Balances Table (replaces materialized view)
-- =============================================================================

-- CAT balances table for fast CAT balance queries (SQLite doesn't support materialized views)
-- Fixed to work without requiring cat_assets records and follows xch_balances pattern
DROP TABLE IF EXISTS cat_balances;

-- Recreate the table following the xch_balances pattern
-- Simple aggregation by asset_id + owner_puzzle_hash, only check spends
CREATE TABLE cat_balances (
  owner_puzzle_hash_hex TEXT NOT NULL,
  asset_id TEXT NOT NULL,
  coin_count INTEGER NOT NULL,
  total_amount INTEGER NOT NULL,
  -- Convert to decimal using standard 3 decimals for CATs 
  total_balance REAL NOT NULL,
  min_coin_amount INTEGER NOT NULL,
  max_coin_amount INTEGER NOT NULL,
  avg_coin_amount INTEGER NOT NULL,
  last_updated TEXT NOT NULL DEFAULT (datetime('now')),
  
  -- Composite primary key
  PRIMARY KEY (asset_id, owner_puzzle_hash_hex)
);

-- =============================================================================
-- CAT Indexes for Performance
-- =============================================================================
-- All indexes are optimized for pagination patterns used in CAT query functions

-- Recreate indexes on the table (simplified structure)
CREATE INDEX idx_cat_balances_asset_owner ON cat_balances(asset_id, owner_puzzle_hash_hex);
CREATE INDEX idx_cat_balances_owner ON cat_balances(owner_puzzle_hash_hex);
CREATE INDEX idx_cat_balances_asset ON cat_balances(asset_id);
CREATE INDEX idx_cat_balances_total_amount ON cat_balances(asset_id, total_amount DESC);
CREATE INDEX idx_cat_balances_owner_total_amount_desc ON cat_balances(owner_puzzle_hash_hex, total_amount DESC);

-- CAT Asset indexes
CREATE INDEX IF NOT EXISTS idx_cat_assets_asset_id ON cat_assets(asset_id);
CREATE INDEX IF NOT EXISTS idx_cat_assets_symbol ON cat_assets(symbol);
CREATE INDEX IF NOT EXISTS idx_cat_assets_first_seen ON cat_assets(first_seen_block);

-- CAT indexes - critical for ownership and asset queries
CREATE INDEX IF NOT EXISTS idx_cats_asset_id ON cats(asset_id);
CREATE INDEX IF NOT EXISTS idx_cats_owner_puzzle_hash ON cats(owner_puzzle_hash);
CREATE INDEX IF NOT EXISTS idx_cats_inner_puzzle_hash ON cats(inner_puzzle_hash);
CREATE INDEX IF NOT EXISTS idx_cats_created_block ON cats(created_block);
CREATE INDEX IF NOT EXISTS idx_cats_amount ON cats(amount);

-- Composite indexes for CAT queries
CREATE INDEX IF NOT EXISTS idx_cats_owner_asset ON cats(owner_puzzle_hash, asset_id);
CREATE INDEX IF NOT EXISTS idx_cats_asset_amount ON cats(asset_id, amount DESC);

-- Pagination-optimized index for get_cats_by_owner_puzzle_hash: ORDER BY created_block DESC, amount DESC
CREATE INDEX IF NOT EXISTS idx_cats_owner_asset_created_block_desc_amount_desc ON cats(owner_puzzle_hash, asset_id, created_block DESC, amount DESC);

-- Covering index for CAT ownership queries (includes commonly accessed columns)
CREATE INDEX IF NOT EXISTS idx_cats_owner_covering ON cats(owner_puzzle_hash, asset_id, coin_id, amount, inner_puzzle_hash, created_block); 