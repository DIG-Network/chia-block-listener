-- =============================================================================
-- Migration 009: NFT Functions
-- =============================================================================
-- Creates NFT query and management functions with pagination support

-- =============================================================================
-- NFT Query Functions
-- =============================================================================

-- Function to get NFTs by owner puzzle hash with optional collection filter and pagination
CREATE OR REPLACE FUNCTION get_nfts_by_owner_puzzle_hash(
  target_owner_puzzle_hash TEXT,
  target_collection_id TEXT DEFAULT NULL,
  page INTEGER DEFAULT 1,
  page_size INTEGER DEFAULT 100
)
RETURNS TABLE(
  coin_id VARCHAR(64),
  launcher_id VARCHAR(64),
  collection_id TEXT,
  collection_name TEXT,
  edition_number INTEGER,
  edition_total INTEGER,
  data_uris TEXT[],
  metadata_uris TEXT[],
  royalty_basis_points INTEGER,
  created_block BIGINT,
  is_spent BOOLEAN
) AS $$
DECLARE
  owner_puzzle_hash_bytes BYTEA;
BEGIN
  -- Convert hex string to BYTEA
  owner_puzzle_hash_bytes := decode(replace(target_owner_puzzle_hash, '0x', ''), 'hex');
  
  RETURN QUERY
  SELECT 
    n.coin_id,
    n.launcher_id,
    n.collection_id,
    nc.name as collection_name,
    n.edition_number,
    n.edition_total,
    n.data_uris,
    n.metadata_uris,
    n.royalty_basis_points,
    n.created_block,
    EXISTS(SELECT 1 FROM spends WHERE spends.coin_id = n.coin_id) as is_spent
  FROM nfts n
  LEFT JOIN nft_collections nc ON n.collection_id = nc.collection_id
  WHERE n.owner_puzzle_hash = owner_puzzle_hash_bytes
  AND (target_collection_id IS NULL OR n.collection_id = target_collection_id)
  ORDER BY n.created_block DESC, n.edition_number
  LIMIT page_size OFFSET (page - 1) * page_size;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get NFTs by collection ID with pagination
CREATE OR REPLACE FUNCTION get_nfts_by_collection_id(
  target_collection_id TEXT,
  page INTEGER DEFAULT 1,
  page_size INTEGER DEFAULT 100
)
RETURNS TABLE(
  coin_id VARCHAR(64),
  launcher_id VARCHAR(64),
  owner_puzzle_hash_hex TEXT,
  current_owner VARCHAR(64),
  edition_number INTEGER,
  edition_total INTEGER,
  data_uris TEXT[],
  metadata_uris TEXT[],
  royalty_basis_points INTEGER,
  created_block BIGINT,
  is_spent BOOLEAN
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    n.coin_id,
    n.launcher_id,
    encode(n.owner_puzzle_hash, 'hex') as owner_puzzle_hash_hex,
    n.current_owner,
    n.edition_number,
    n.edition_total,
    n.data_uris,
    n.metadata_uris,
    n.royalty_basis_points,
    n.created_block,
    EXISTS(SELECT 1 FROM spends WHERE spends.coin_id = n.coin_id) as is_spent
  FROM nfts n
  WHERE n.collection_id = target_collection_id
  ORDER BY n.edition_number, n.created_block
  LIMIT page_size OFFSET (page - 1) * page_size;
END;
$$ LANGUAGE plpgsql STABLE;

-- =============================================================================
-- NFT Collection Query Functions
-- =============================================================================

-- Function to get NFT collection information by collection ID
CREATE OR REPLACE FUNCTION get_nft_collection_by_id(target_collection_id TEXT)
RETURNS TABLE(
  collection_id TEXT,
  name TEXT,
  description TEXT,
  total_supply INTEGER,
  creator_puzzle_hash BYTEA,
  metadata_json JSONB,
  first_seen_block BIGINT,
  created_at TIMESTAMP
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    nc.collection_id,
    nc.name,
    nc.description,
    nc.total_supply,
    nc.creator_puzzle_hash,
    nc.metadata_json,
    nc.first_seen_block,
    nc.created_at
  FROM nft_collections nc
  WHERE nc.collection_id = target_collection_id;
END;
$$ LANGUAGE plpgsql STABLE; 