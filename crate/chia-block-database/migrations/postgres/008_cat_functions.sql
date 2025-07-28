-- =============================================================================
-- Migration 008: CAT Functions
-- =============================================================================
-- Creates CAT balance and query functions with pagination support

-- =============================================================================
-- CAT Balance Functions
-- =============================================================================

-- Function to refresh the CAT balances materialized view
CREATE OR REPLACE FUNCTION refresh_cat_balances()
RETURNS BOOLEAN AS $$
BEGIN
  REFRESH MATERIALIZED VIEW CONCURRENTLY cat_balances;
  RETURN TRUE;
EXCEPTION
  WHEN OTHERS THEN
    RETURN FALSE;
END;
$$ LANGUAGE plpgsql;

-- Function to get all CAT balances for a specific owner with pagination
CREATE OR REPLACE FUNCTION get_cat_balances_by_owner(
  target_puzzle_hash TEXT,
  page INTEGER DEFAULT 1,
  page_size INTEGER DEFAULT 100
)
RETURNS TABLE(
  asset_id VARCHAR(64),
  symbol TEXT,
  asset_name TEXT,
  owner_puzzle_hash_hex TEXT,
  coin_count BIGINT,
  total_amount NUMERIC,
  total_balance NUMERIC,
  min_coin_amount NUMERIC,
  max_coin_amount NUMERIC,
  avg_coin_amount NUMERIC,
  last_activity_block BIGINT,
  decimals INTEGER
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    cb.asset_id,
    cb.symbol,
    cb.asset_name,
    cb.owner_puzzle_hash_hex,
    cb.coin_count,
    cb.total_amount,
    cb.total_balance,
    cb.min_coin_amount,
    cb.max_coin_amount,
    cb.avg_coin_amount,
    cb.last_activity_block,
    cb.decimals
  FROM cat_balances cb
  WHERE cb.owner_puzzle_hash_hex = target_puzzle_hash
  ORDER BY cb.total_amount DESC
  LIMIT page_size OFFSET (page - 1) * page_size;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get CAT balance for specific asset and owner
CREATE OR REPLACE FUNCTION get_cat_balance_by_asset_and_owner(target_asset_id TEXT, target_puzzle_hash TEXT)
RETURNS TABLE(
  asset_id VARCHAR(64),
  symbol TEXT,
  asset_name TEXT,
  owner_puzzle_hash_hex TEXT,
  coin_count BIGINT,
  total_amount NUMERIC,
  total_balance NUMERIC,
  min_coin_amount NUMERIC,
  max_coin_amount NUMERIC,
  avg_coin_amount NUMERIC,
  last_activity_block BIGINT,
  decimals INTEGER
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    cb.asset_id,
    cb.symbol,
    cb.asset_name,
    cb.owner_puzzle_hash_hex,
    cb.coin_count,
    cb.total_amount,
    cb.total_balance,
    cb.min_coin_amount,
    cb.max_coin_amount,
    cb.avg_coin_amount,
    cb.last_activity_block,
    cb.decimals
  FROM cat_balances cb
  WHERE cb.asset_id = target_asset_id 
    AND cb.owner_puzzle_hash_hex = target_puzzle_hash;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get top holders for a specific CAT asset with pagination
CREATE OR REPLACE FUNCTION get_cat_holders_by_asset(
  target_asset_id TEXT,
  page INTEGER DEFAULT 1,
  page_size INTEGER DEFAULT 100
)
RETURNS TABLE(
  owner_puzzle_hash_hex TEXT,
  coin_count BIGINT,
  total_amount NUMERIC,
  total_balance NUMERIC,
  min_coin_amount NUMERIC,
  max_coin_amount NUMERIC,
  avg_coin_amount NUMERIC,
  last_activity_block BIGINT
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    cb.owner_puzzle_hash_hex,
    cb.coin_count,
    cb.total_amount,
    cb.total_balance,
    cb.min_coin_amount,
    cb.max_coin_amount,
    cb.avg_coin_amount,
    cb.last_activity_block
  FROM cat_balances cb
  WHERE cb.asset_id = target_asset_id
  ORDER BY cb.total_amount DESC
  LIMIT page_size OFFSET (page - 1) * page_size;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get summary of all CAT assets (total supply, holder count, etc.) with pagination
CREATE OR REPLACE FUNCTION get_cat_asset_summary(
  page INTEGER DEFAULT 1,
  page_size INTEGER DEFAULT 100
)
RETURNS TABLE(
  asset_id VARCHAR(64),
  symbol TEXT,
  asset_name TEXT,
  holder_count BIGINT,
  total_circulating_supply NUMERIC,
  total_circulating_balance NUMERIC,
  top_holder_amount NUMERIC,
  avg_holder_balance NUMERIC,
  decimals INTEGER
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    cb.asset_id,
    cb.symbol,
    cb.asset_name,
    COUNT(*)::BIGINT as holder_count,
    SUM(cb.total_amount) as total_circulating_supply,
    SUM(cb.total_balance) as total_circulating_balance,
    MAX(cb.total_amount) as top_holder_amount,
    ROUND(AVG(cb.total_balance)::numeric, COALESCE(MAX(cb.decimals), 3)) as avg_holder_balance,
    MAX(cb.decimals) as decimals
  FROM cat_balances cb
  GROUP BY cb.asset_id, cb.symbol, cb.asset_name
  ORDER BY total_circulating_supply DESC
  LIMIT page_size OFFSET (page - 1) * page_size;
END;
$$ LANGUAGE plpgsql STABLE;

-- =============================================================================
-- CAT Asset Query Functions
-- =============================================================================

-- Function to get CAT asset information by asset ID
CREATE OR REPLACE FUNCTION get_cat_asset_by_id(target_asset_id TEXT)
RETURNS TABLE(
  asset_id VARCHAR(64),
  symbol TEXT,
  name TEXT,
  description TEXT,
  total_supply NUMERIC,
  decimals INTEGER,
  metadata_json JSONB,
  first_seen_block BIGINT,
  created_at TIMESTAMP
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    ca.asset_id,
    ca.symbol,
    ca.name,
    ca.description,
    ca.total_supply,
    ca.decimals,
    ca.metadata_json,
    ca.first_seen_block,
    ca.created_at
  FROM cat_assets ca
  WHERE ca.asset_id = target_asset_id;
END;
$$ LANGUAGE plpgsql STABLE;

-- =============================================================================
-- CAT Query Functions
-- =============================================================================

-- Function to get CATs by owner puzzle hash and asset ID with pagination
CREATE OR REPLACE FUNCTION get_cats_by_owner_puzzle_hash(
  target_owner_puzzle_hash TEXT,
  target_asset_id TEXT,
  page INTEGER DEFAULT 1,
  page_size INTEGER DEFAULT 100
)
RETURNS TABLE(
  coin_id VARCHAR(64),
  asset_id VARCHAR(64),
  asset_name TEXT,
  asset_symbol TEXT,
  amount NUMERIC,
  inner_puzzle_hash_hex TEXT,
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
    c.coin_id,
    ca.asset_id,
    ca.name as asset_name,
    ca.symbol as asset_symbol,
    c.amount,
    encode(c.inner_puzzle_hash, 'hex') as inner_puzzle_hash_hex,
    c.created_block,
    EXISTS(SELECT 1 FROM spends WHERE spends.coin_id = c.coin_id) as is_spent
  FROM cats c
  INNER JOIN cat_assets ca ON c.asset_id = ca.asset_id
  WHERE c.owner_puzzle_hash = owner_puzzle_hash_bytes
  AND ca.asset_id = target_asset_id
  ORDER BY c.created_block DESC, c.amount DESC
  LIMIT page_size OFFSET (page - 1) * page_size;
END;
$$ LANGUAGE plpgsql STABLE; 