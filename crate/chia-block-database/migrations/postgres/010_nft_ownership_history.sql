-- =============================================================================
-- Migration 010: NFT Ownership History Functions
-- =============================================================================
-- Creates NFT ownership history tracking and query functions with pagination support

-- =============================================================================
-- NFT Ownership History Functions
-- =============================================================================

-- Function to get complete ownership history for an NFT with pagination
CREATE OR REPLACE FUNCTION get_nft_ownership_history(
  target_launcher_id TEXT,
  page INTEGER DEFAULT 1,
  page_size INTEGER DEFAULT 100
)
RETURNS TABLE(
  id INTEGER,
  launcher_id VARCHAR(64),
  coin_id VARCHAR(64),
  collection_id TEXT,
  previous_owner_puzzle_hash_hex TEXT,
  new_owner_puzzle_hash_hex TEXT,
  previous_owner VARCHAR(64),
  new_owner VARCHAR(64),
  transfer_type VARCHAR(20),
  block_height BIGINT,
  block_timestamp TIMESTAMP,
  spend_coin_id VARCHAR(64),
  created_at TIMESTAMP
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    h.id,
    h.launcher_id,
    h.coin_id,
    h.collection_id,
    encode(h.previous_owner_puzzle_hash, 'hex') as previous_owner_puzzle_hash_hex,
    encode(h.new_owner_puzzle_hash, 'hex') as new_owner_puzzle_hash_hex,
    h.previous_owner,
    h.new_owner,
    h.transfer_type,
    h.block_height,
    h.block_timestamp,
    h.spend_coin_id,
    h.created_at
  FROM nft_ownership_history h
  WHERE h.launcher_id = target_launcher_id
  ORDER BY h.block_height ASC, h.id ASC
  LIMIT page_size OFFSET (page - 1) * page_size;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get all NFTs owned by a specific address at any point in history with pagination
CREATE OR REPLACE FUNCTION get_nft_history_by_owner(
  target_owner_puzzle_hash TEXT,
  page INTEGER DEFAULT 1,
  page_size INTEGER DEFAULT 100
)
RETURNS TABLE(
  launcher_id VARCHAR(64),
  coin_id VARCHAR(64),
  collection_id TEXT,
  transfer_type VARCHAR(20),
  block_height BIGINT,
  block_timestamp TIMESTAMP,
  was_previous_owner BOOLEAN,
  was_new_owner BOOLEAN
) AS $$
DECLARE
  owner_puzzle_hash_bytes BYTEA;
BEGIN
  -- Convert hex string to bytes
  owner_puzzle_hash_bytes := decode(target_owner_puzzle_hash, 'hex');
  
  RETURN QUERY
  SELECT 
    h.launcher_id,
    h.coin_id,
    h.collection_id,
    h.transfer_type,
    h.block_height,
    h.block_timestamp,
    (h.previous_owner_puzzle_hash = owner_puzzle_hash_bytes) as was_previous_owner,
    (h.new_owner_puzzle_hash = owner_puzzle_hash_bytes) as was_new_owner
  FROM nft_ownership_history h
  WHERE h.previous_owner_puzzle_hash = owner_puzzle_hash_bytes
     OR h.new_owner_puzzle_hash = owner_puzzle_hash_bytes
  ORDER BY h.block_height DESC, h.id DESC
  LIMIT page_size OFFSET (page - 1) * page_size;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get who owned an NFT at a specific block height
CREATE OR REPLACE FUNCTION get_nft_owner_at_block(target_launcher_id TEXT, target_block_height BIGINT)
RETURNS TABLE(
  owner_puzzle_hash_hex TEXT,
  current_owner VARCHAR(64),
  block_height BIGINT,
  transfer_type VARCHAR(20)
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    encode(h.new_owner_puzzle_hash, 'hex') as owner_puzzle_hash_hex,
    h.new_owner as current_owner,
    h.block_height,
    h.transfer_type
  FROM nft_ownership_history h
  WHERE h.launcher_id = target_launcher_id
    AND h.block_height <= target_block_height
  ORDER BY h.block_height DESC, h.id DESC
  LIMIT 1;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get all NFT transfers within a block range with pagination
CREATE OR REPLACE FUNCTION get_nft_transfers_in_block_range(
  start_block BIGINT,
  end_block BIGINT,
  page INTEGER DEFAULT 1,
  page_size INTEGER DEFAULT 100
)
RETURNS TABLE(
  launcher_id VARCHAR(64),
  coin_id VARCHAR(64),
  collection_id TEXT,
  previous_owner_puzzle_hash_hex TEXT,
  new_owner_puzzle_hash_hex TEXT,
  transfer_type VARCHAR(20),
  block_height BIGINT,
  block_timestamp TIMESTAMP
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    h.launcher_id,
    h.coin_id,
    h.collection_id,
    encode(h.previous_owner_puzzle_hash, 'hex') as previous_owner_puzzle_hash_hex,
    encode(h.new_owner_puzzle_hash, 'hex') as new_owner_puzzle_hash_hex,
    h.transfer_type,
    h.block_height,
    h.block_timestamp
  FROM nft_ownership_history h
  WHERE h.block_height >= start_block 
    AND h.block_height <= end_block
  ORDER BY h.block_height ASC, h.id ASC
  LIMIT page_size OFFSET (page - 1) * page_size;
END;
$$ LANGUAGE plpgsql STABLE; 