-- =============================================================================
-- Migration 006: NFT Tables and Indexes
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
  creator_puzzle_hash BYTEA,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  metadata_json JSONB, -- Store full collection metadata
  first_seen_block BIGINT
);

-- Create NFTs table for individual NFT records (current state)
CREATE TABLE IF NOT EXISTS nfts (
  coin_id VARCHAR(64) PRIMARY KEY,
  launcher_id VARCHAR(64) NOT NULL UNIQUE, -- NFT launcher ID
  collection_id TEXT, -- Formerly REFERENCES nft_collections(collection_id)
  owner_puzzle_hash BYTEA NOT NULL, -- p2PuzzleHash from NFT info
  current_owner VARCHAR(64), -- currentOwner from NFT info (can be null)
  edition_number INTEGER,
  edition_total INTEGER,
  data_uris TEXT[], -- Array of data URIs
  data_hash VARCHAR(64),
  metadata_uris TEXT[], -- Array of metadata URIs
  metadata_hash VARCHAR(64),
  license_uris TEXT[], -- Array of license URIs  
  license_hash VARCHAR(64),
  metadata_updater_puzzle_hash BYTEA,
  royalty_puzzle_hash BYTEA,
  royalty_basis_points INTEGER,
  created_block BIGINT,
  last_updated_block BIGINT, -- Track when NFT was last updated
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  metadata_json JSONB -- Store full NFT metadata
  
  -- Removed foreign key constraints:
  -- FOREIGN KEY (coin_id) REFERENCES coins(coin_id) ON DELETE CASCADE,
  -- FOREIGN KEY (created_block) REFERENCES blocks(height) ON DELETE CASCADE,
  -- FOREIGN KEY (last_updated_block) REFERENCES blocks(height) ON DELETE CASCADE
);

-- Create NFT ownership history table to track all ownership changes
CREATE TABLE IF NOT EXISTS nft_ownership_history (
  id SERIAL PRIMARY KEY,
  launcher_id VARCHAR(64) NOT NULL, -- NFT launcher ID
  coin_id VARCHAR(64) NOT NULL, -- The coin representing this NFT state
  collection_id TEXT, -- Formerly REFERENCES nft_collections(collection_id)
  previous_owner_puzzle_hash BYTEA, -- Previous owner (NULL for minting)
  new_owner_puzzle_hash BYTEA NOT NULL, -- New owner
  previous_owner VARCHAR(64), -- Previous currentOwner field
  new_owner VARCHAR(64), -- New currentOwner field  
  transfer_type VARCHAR(20) DEFAULT 'transfer', -- 'mint', 'transfer', 'burn'
  block_height BIGINT NOT NULL,
  block_timestamp TIMESTAMP,
  spend_coin_id VARCHAR(64), -- The coin that was spent to cause this transfer
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  metadata_json JSONB -- Store full NFT state at this point in time
  
  -- Removed foreign key constraints:
  -- FOREIGN KEY (block_height) REFERENCES blocks(height) ON DELETE CASCADE
);

-- =============================================================================
-- Additional Indexes for Table Relationships (replacing foreign keys)
-- =============================================================================

-- Indexes to maintain nfts -> coins relationship
CREATE INDEX IF NOT EXISTS idx_nfts_coin_id_ref ON nfts(coin_id);

-- Indexes to maintain nfts -> blocks relationship
CREATE INDEX IF NOT EXISTS idx_nfts_created_block_ref ON nfts(created_block) WHERE created_block IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_nfts_last_updated_block_ref ON nfts(last_updated_block) WHERE last_updated_block IS NOT NULL;

-- Indexes to maintain nft_ownership_history -> blocks relationship
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_block_height_ref ON nft_ownership_history(block_height);

-- Indexes to maintain nft_ownership_history -> coins relationship (for spend_coin_id)
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_spend_coin_id_ref ON nft_ownership_history(spend_coin_id) WHERE spend_coin_id IS NOT NULL;

-- Indexes to maintain nft_ownership_history -> nfts relationship
CREATE INDEX IF NOT EXISTS idx_nft_ownership_history_coin_id_ref ON nft_ownership_history(coin_id);

-- Composite indexes for efficient joins
CREATE INDEX IF NOT EXISTS idx_nfts_coins_join ON nfts(coin_id, launcher_id);
CREATE INDEX IF NOT EXISTS idx_nfts_blocks_created_join ON nfts(created_block, coin_id) WHERE created_block IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_nfts_blocks_updated_join ON nfts(last_updated_block, coin_id) WHERE last_updated_block IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_nft_history_blocks_join ON nft_ownership_history(block_height, launcher_id);
CREATE INDEX IF NOT EXISTS idx_nft_history_coins_join ON nft_ownership_history(coin_id, block_height);
CREATE INDEX IF NOT EXISTS idx_nft_history_spend_coins_join ON nft_ownership_history(spend_coin_id, block_height) WHERE spend_coin_id IS NOT NULL;

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

-- Covering index for NFT ownership queries (avoids table lookups)
CREATE INDEX IF NOT EXISTS idx_nfts_owner_covering ON nfts(owner_puzzle_hash) 
  INCLUDE (coin_id, launcher_id, collection_id, edition_number, edition_total, created_block); 

-- =============================================================================
-- CAT Balances Fix (from Migration 011)
-- =============================================================================
-- Fixes the cat_balances materialized view to work even when cat_assets records
-- don't exist and when coins table doesn't have matching records

-- Drop the existing materialized view
DROP MATERIALIZED VIEW IF EXISTS cat_balances CASCADE;

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

-- Recreate indexes on the materialized view (simplified structure)
CREATE UNIQUE INDEX idx_cat_balances_asset_owner ON cat_balances(asset_id, owner_puzzle_hash_hex);
CREATE INDEX idx_cat_balances_owner ON cat_balances(owner_puzzle_hash_hex);
CREATE INDEX idx_cat_balances_asset ON cat_balances(asset_id);
CREATE INDEX idx_cat_balances_total_amount ON cat_balances(asset_id, total_amount DESC);
CREATE INDEX idx_cat_balances_owner_total_amount_desc ON cat_balances(owner_puzzle_hash_hex, total_amount DESC); 