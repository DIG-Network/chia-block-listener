-- =============================================================================
-- Migration 006: NFT Tables and Indexes (SQLite)
-- =============================================================================
-- Creates NFT-related tables: nft_collections, nfts, nft_ownership_history with indexes

-- =============================================================================
-- NFT Tables
-- =============================================================================

-- Create NFT collections table for normalized collection data
CREATE TABLE IF NOT EXISTS nft_collections (
  collection_id TEXT PRIMARY KEY, -- Collection ID as natural primary key
  name TEXT,
  description TEXT,
  total_supply INTEGER,
  creator_puzzle_hash BLOB,
  created_at TEXT DEFAULT (datetime('now')),
  metadata_json TEXT, -- Store full collection metadata as JSON text
  first_seen_block INTEGER
);

-- Create NFTs table for individual NFT records (current state)
CREATE TABLE IF NOT EXISTS nfts (
  coin_id TEXT PRIMARY KEY,
  launcher_id TEXT NOT NULL UNIQUE, -- NFT launcher ID
  collection_id TEXT, -- Formerly REFERENCES nft_collections(collection_id)
  owner_puzzle_hash BLOB NOT NULL, -- p2PuzzleHash from NFT info
  current_owner TEXT, -- currentOwner from NFT info (can be null)
  edition_number INTEGER,
  edition_total INTEGER,
  data_uris TEXT, -- JSON array of data URIs
  data_hash TEXT,
  metadata_uris TEXT, -- JSON array of metadata URIs
  metadata_hash TEXT,
  license_uris TEXT, -- JSON array of license URIs  
  license_hash TEXT,
  metadata_updater_puzzle_hash BLOB,
  royalty_puzzle_hash BLOB,
  royalty_basis_points INTEGER,
  created_block INTEGER,
  last_updated_block INTEGER, -- Track when NFT was last updated
  created_at TEXT DEFAULT (datetime('now')),
  updated_at TEXT DEFAULT (datetime('now')),
  metadata_json TEXT -- Store full NFT metadata as JSON text
  
  -- Removed foreign key constraints:
  -- FOREIGN KEY (coin_id) REFERENCES coins(coin_id) ON DELETE CASCADE,
  -- FOREIGN KEY (created_block) REFERENCES blocks(height) ON DELETE CASCADE,
  -- FOREIGN KEY (last_updated_block) REFERENCES blocks(height) ON DELETE CASCADE
);

-- Create NFT ownership history table to track all ownership changes
CREATE TABLE IF NOT EXISTS nft_ownership_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  launcher_id TEXT NOT NULL, -- NFT launcher ID
  coin_id TEXT NOT NULL, -- The coin representing this NFT state
  collection_id TEXT, -- Formerly REFERENCES nft_collections(collection_id)
  previous_owner_puzzle_hash BLOB, -- Previous owner (NULL for minting)
  new_owner_puzzle_hash BLOB NOT NULL, -- New owner
  previous_owner TEXT, -- Previous currentOwner field
  new_owner TEXT, -- New currentOwner field  
  transfer_type TEXT DEFAULT 'transfer', -- 'mint', 'transfer', 'burn'
  block_height INTEGER NOT NULL,
  block_timestamp TEXT,
  spend_coin_id TEXT, -- The coin that was spent to cause this transfer
  created_at TEXT DEFAULT (datetime('now')),
  metadata_json TEXT -- Store full NFT state at this point in time as JSON text
  
  -- Removed foreign key constraints:
  -- FOREIGN KEY (block_height) REFERENCES blocks(height) ON DELETE CASCADE
);

-- =============================================================================
-- Additional Indexes for Table Relationships (replacing foreign keys)
-- =============================================================================

-- Indexes to maintain nfts -> coins relationship
CREATE INDEX IF NOT EXISTS idx_nfts_coin_id_ref ON nfts(coin_id);

-- Indexes to maintain nfts -> blocks relationship
CREATE INDEX IF NOT EXISTS idx_nfts_created_block_ref ON nfts(created_block);
CREATE INDEX IF NOT EXISTS idx_nfts_last_updated_block_ref ON nfts(last_updated_block);

-- Indexes to maintain nft_ownership_history -> blocks relationship
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_block_height_ref ON nft_ownership_history(block_height);

-- Indexes to maintain nft_ownership_history -> coins relationship (for spend_coin_id)
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_spend_coin_id_ref ON nft_ownership_history(spend_coin_id);

-- Indexes to maintain nft_ownership_history -> nfts relationship
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_coin_id_ref ON nft_ownership_history(coin_id);

-- Composite indexes for efficient joins
CREATE INDEX IF NOT EXISTS idx_nfts_coins_join ON nfts(coin_id, launcher_id);
CREATE INDEX IF NOT EXISTS idx_nfts_blocks_created_join ON nfts(created_block, coin_id);
CREATE INDEX IF NOT EXISTS idx_nfts_blocks_updated_join ON nfts(last_updated_block, coin_id);
CREATE INDEX IF NOT EXISTS idx_nft_history_blocks_join ON nft_ownership_history(block_height, launcher_id);
CREATE INDEX IF NOT EXISTS idx_nft_history_coins_join ON nft_ownership_history(coin_id, block_height);
CREATE INDEX IF NOT EXISTS idx_nft_history_spend_coins_join ON nft_ownership_history(spend_coin_id, block_height);

-- =============================================================================
-- NFT Indexes for Performance
-- =============================================================================
-- All indexes are optimized for pagination patterns used in NFT query functions

-- NFT Collection indexes
CREATE INDEX IF NOT EXISTS idx_nft_collections_collection_id ON nft_collections(collection_id);
CREATE INDEX IF NOT EXISTS idx_nft_collections_creator ON nft_collections(creator_puzzle_hash);
CREATE INDEX IF NOT EXISTS idx_nft_collections_first_seen ON nft_collections(first_seen_block);

-- NFT ownership history indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_launcher_id ON nft_ownership_history(launcher_id);
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_coin_id ON nft_ownership_history(coin_id);
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_new_owner ON nft_ownership_history(new_owner_puzzle_hash);
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_previous_owner ON nft_ownership_history(previous_owner_puzzle_hash);
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_block_height ON nft_ownership_history(block_height);
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_collection ON nft_ownership_history(collection_id);
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_transfer_type ON nft_ownership_history(transfer_type);

-- Pagination-optimized indexes for NFT ownership history
-- For get_nft_ownership_history: ORDER BY block_height ASC, id ASC
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_launcher_block_id_asc ON nft_ownership_history(launcher_id, block_height ASC, id ASC);

-- For get_nft_history_by_owner: ORDER BY block_height DESC, id DESC
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_owner_block_id_desc ON nft_ownership_history(new_owner_puzzle_hash, block_height DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_prev_owner_block_id_desc ON nft_ownership_history(previous_owner_puzzle_hash, block_height DESC, id DESC);

-- For get_nft_transfers_in_block_range: ORDER BY block_height ASC, id ASC
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_block_range_asc ON nft_ownership_history(block_height ASC, id ASC);

-- NFT indexes - critical for ownership and collection queries
CREATE INDEX IF NOT EXISTS idx_nfts_launcher_id ON nfts(launcher_id);
CREATE INDEX IF NOT EXISTS idx_nfts_owner_puzzle_hash ON nfts(owner_puzzle_hash);
CREATE INDEX IF NOT EXISTS idx_nfts_collection_id ON nfts(collection_id);
CREATE INDEX IF NOT EXISTS idx_nfts_current_owner ON nfts(current_owner);
CREATE INDEX IF NOT EXISTS idx_nfts_created_block ON nfts(created_block);

-- Composite indexes for NFT queries
CREATE INDEX IF NOT EXISTS idx_nfts_owner_collection ON nfts(owner_puzzle_hash, collection_id);
CREATE INDEX IF NOT EXISTS idx_nfts_collection_edition ON nfts(collection_id, edition_number);

-- Pagination-optimized indexes for NFT queries
-- For get_nfts_by_owner_puzzle_hash: ORDER BY created_block DESC, edition_number
CREATE INDEX IF NOT EXISTS idx_nfts_owner_created_block_desc_edition ON nfts(owner_puzzle_hash, created_block DESC, edition_number);

-- For get_nfts_by_collection_id: ORDER BY edition_number, created_block  
CREATE INDEX IF NOT EXISTS idx_nfts_collection_edition_created_block ON nfts(collection_id, edition_number, created_block);

-- Covering index for NFT ownership queries (includes commonly accessed columns)
CREATE INDEX IF NOT EXISTS idx_nfts_owner_covering ON nfts(owner_puzzle_hash, coin_id, launcher_id, collection_id, edition_number, edition_total, created_block);

-- =============================================================================
-- CAT Balances Fix (from Migration 011)
-- =============================================================================
-- Fixes the cat_balances table to work without requiring cat_assets records
-- and simplifies the structure following the xch_balances pattern

-- Drop the existing table and recreate with simplified structure
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

-- Recreate indexes on the table (simplified structure)
CREATE INDEX idx_cat_balances_asset_owner ON cat_balances(asset_id, owner_puzzle_hash_hex);
CREATE INDEX idx_cat_balances_owner ON cat_balances(owner_puzzle_hash_hex);
CREATE INDEX idx_cat_balances_asset ON cat_balances(asset_id);
CREATE INDEX idx_cat_balances_total_amount ON cat_balances(asset_id, total_amount DESC);
CREATE INDEX idx_cat_balances_owner_total_amount_desc ON cat_balances(owner_puzzle_hash_hex, total_amount DESC); 