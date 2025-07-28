-- =============================================================================
-- Migration 007: CAT Tables and Balance Views
-- =============================================================================
-- Creates CAT-related tables: cat_assets, cats, cat_balances materialized view with indexes

-- =============================================================================
-- CAT Tables
-- =============================================================================

-- Create CAT assets table for normalized CAT asset data
CREATE TABLE IF NOT EXISTS cat_assets (
  asset_id VARCHAR(64) PRIMARY KEY, -- CAT asset ID as natural primary key
  symbol TEXT,
  name TEXT,
  description TEXT,
  total_supply NUMERIC(20,0),
  decimals INTEGER DEFAULT 3, -- Most CATs use 3 decimals (mojos)
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  metadata_json JSONB, -- Store full asset metadata
  first_seen_block BIGINT
);

-- Create CATs table for individual CAT coin records
CREATE TABLE IF NOT EXISTS cats (
  coin_id VARCHAR(64) PRIMARY KEY,
  asset_id VARCHAR(64) NOT NULL, -- Formerly REFERENCES cat_assets(asset_id)
  owner_puzzle_hash BYTEA NOT NULL, -- p2PuzzleHash from CAT info
  inner_puzzle_hash BYTEA NOT NULL, -- innerPuzzleHash from CAT info
  amount NUMERIC(20,0) NOT NULL, -- CAT amount
  lineage_proof JSONB, -- Store lineage proof data
  created_block BIGINT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
  
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
CREATE INDEX IF NOT EXISTS idx_cats_created_block_ref ON cats(created_block) WHERE created_block IS NOT NULL;

-- Indexes to maintain cats -> cat_assets relationship
CREATE INDEX IF NOT EXISTS idx_cats_asset_id_ref ON cats(asset_id);

-- Composite indexes for efficient joins
CREATE INDEX IF NOT EXISTS idx_cats_coins_join ON cats(coin_id, asset_id);
CREATE INDEX IF NOT EXISTS idx_cats_blocks_join ON cats(created_block, coin_id) WHERE created_block IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_cats_assets_join ON cats(asset_id, owner_puzzle_hash);

-- =============================================================================
-- CAT Balances Materialized View
-- =============================================================================

-- CAT balances materialized view for fast CAT balance queries
-- Fixed to work even when cat_assets records don't exist and follows xch_balances pattern
DROP MATERIALIZED VIEW IF EXISTS cat_balances CASCADE;
DROP VIEW IF EXISTS cat_balances CASCADE;

-- Recreate the materialized view following the xch_balances pattern
-- Simple aggregation by asset_id + owner_puzzle_hash, only check spends (not pending_coins)
CREATE MATERIALIZED VIEW cat_balances AS
SELECT 
  encode(c.owner_puzzle_hash, 'hex') as owner_puzzle_hash_hex, 
  c.asset_id,
  COUNT(*) as coin_count,
  SUM(c.amount) as total_amount,
  -- Convert to decimal using standard 3 decimals for CATs 
  ROUND(SUM(c.amount) / 1000.0, 3) as total_balance,
  MIN(c.amount) as min_coin_amount,
  MAX(c.amount) as max_coin_amount,
  ROUND(AVG(c.amount)::numeric, 0) as avg_coin_amount,
  CURRENT_TIMESTAMP as last_updated
FROM cats c
GROUP BY c.asset_id, c.owner_puzzle_hash
ORDER BY c.owner_puzzle_hash, c.asset_id, total_amount DESC;

-- =============================================================================
-- CAT Indexes for Performance
-- =============================================================================
-- All indexes are optimized for pagination patterns used in CAT query functions

-- Recreate indexes on the materialized view (simplified structure)
CREATE UNIQUE INDEX idx_cat_balances_asset_owner ON cat_balances(asset_id, owner_puzzle_hash_hex);
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

-- Covering index for CAT ownership queries (avoids table lookups)
CREATE INDEX IF NOT EXISTS idx_cats_owner_covering ON cats(owner_puzzle_hash, asset_id) 
  INCLUDE (coin_id, amount, inner_puzzle_hash, created_block); 