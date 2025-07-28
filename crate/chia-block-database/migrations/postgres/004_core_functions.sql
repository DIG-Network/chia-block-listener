-- =============================================================================
-- Migration 004: Core Functions
-- =============================================================================
-- Creates fundamental functions for balance queries, coin management, puzzle handling

-- Drop existing functions first to avoid return type conflicts
DROP FUNCTION IF EXISTS unspent_coins_by_puzzle_hash(TEXT);
DROP FUNCTION IF EXISTS get_balance_by_puzzle_hash(TEXT);

-- =============================================================================
-- System Preferences Functions
-- =============================================================================

-- Function to get a system preference value
CREATE OR REPLACE FUNCTION get_system_preference(
  preference_key VARCHAR(255)
)
RETURNS TEXT AS $$
DECLARE
  preference_value TEXT;
BEGIN
  SELECT meta_value INTO preference_value
  FROM system_preferences 
  WHERE meta_key = preference_key;
  
  RETURN preference_value;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to check if sync is paused
CREATE OR REPLACE FUNCTION is_sync_paused()
RETURNS BOOLEAN AS $$
DECLARE
  paused_value TEXT;
BEGIN
  SELECT get_system_preference('paused_sync') INTO paused_value;
  
  -- Return true if preference is 'true', 'yes', '1', or 'on' (case insensitive)
  RETURN LOWER(COALESCE(paused_value, 'false')) IN ('true', 'yes', '1', 'on');
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to check if process should exit (kill switch)
CREATE OR REPLACE FUNCTION should_exit_process()
RETURNS BOOLEAN AS $$
DECLARE
  exit_value TEXT;
BEGIN
  SELECT get_system_preference('exit_process') INTO exit_value;
  
  -- Return true if preference is 'true', 'yes', '1', or 'on' (case insensitive)
  RETURN LOWER(COALESCE(exit_value, 'false')) IN ('true', 'yes', '1', 'on');
END;
$$ LANGUAGE plpgsql STABLE;

-- =============================================================================
-- Balance Query Functions
-- =============================================================================

-- Function to get unspent coins by puzzle hash with pagination
CREATE OR REPLACE FUNCTION unspent_coins_by_puzzle_hash(
  target_puzzle_hash TEXT,
  page INTEGER DEFAULT 1,
  page_size INTEGER DEFAULT 100
)
RETURNS TABLE(
  coin_id VARCHAR(64),
  amount NUMERIC,
  created_block BIGINT,
  parent_coin_info BYTEA
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    uc.coin_id,
    uc.amount,
    uc.created_block,
    uc.parent_coin_info
  FROM unspent_coins uc
  WHERE encode(uc.puzzle_hash, 'hex') = target_puzzle_hash
  ORDER BY uc.amount DESC
  LIMIT page_size OFFSET (page - 1) * page_size;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get balance summary by puzzle hash
CREATE OR REPLACE FUNCTION get_balance_by_puzzle_hash(target_puzzle_hash TEXT)
RETURNS TABLE(
  total_mojos NUMERIC,
  total_xch NUMERIC,
  coin_count BIGINT
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    COALESCE(SUM(uc.amount), 0) as total_mojos,
    ROUND(COALESCE(SUM(uc.amount), 0) / 1000000000000.0, 12) as total_xch,
    COUNT(*) as coin_count
  FROM unspent_coins uc
  WHERE encode(uc.puzzle_hash, 'hex') = target_puzzle_hash;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to refresh the materialized view
CREATE OR REPLACE FUNCTION refresh_xch_balances()
RETURNS BOOLEAN AS $$
BEGIN
  REFRESH MATERIALIZED VIEW CONCURRENTLY xch_balances;
  RETURN TRUE;
EXCEPTION
  WHEN OTHERS THEN
    RETURN FALSE;
END;
$$ LANGUAGE plpgsql;

-- Function to get top balances with pagination
CREATE OR REPLACE FUNCTION get_balances(
  page INTEGER DEFAULT 1,
  page_size INTEGER DEFAULT 100
)
RETURNS TABLE(
  puzzle_hash_hex TEXT,
  coin_count BIGINT,
  total_mojos NUMERIC,
  total_xch NUMERIC,
  min_coin_amount NUMERIC,
  max_coin_amount NUMERIC,
  avg_coin_amount NUMERIC
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    xb.puzzle_hash_hex,
    xb.coin_count,
    xb.total_mojos,
    xb.total_xch,
    xb.min_coin_amount,
    xb.max_coin_amount,
    xb.avg_coin_amount
  FROM xch_balances xb
  ORDER BY xb.total_mojos DESC 
  LIMIT page_size OFFSET (page - 1) * page_size;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get balance for specific puzzle hash
CREATE OR REPLACE FUNCTION get_balance_by_puzzle_hash_detail(target_puzzle_hash TEXT)
RETURNS TABLE(
  puzzle_hash_hex TEXT,
  coin_count BIGINT,
  total_mojos NUMERIC,
  total_xch NUMERIC,
  min_coin_amount NUMERIC,
  max_coin_amount NUMERIC,
  avg_coin_amount NUMERIC
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    xb.puzzle_hash_hex,
    xb.coin_count,
    xb.total_mojos,
    xb.total_xch,
    xb.min_coin_amount,
    xb.max_coin_amount,
    xb.avg_coin_amount
  FROM xch_balances xb
  WHERE xb.puzzle_hash_hex = target_puzzle_hash;
END;
$$ LANGUAGE plpgsql STABLE;

-- =============================================================================
-- Puzzle Management Functions
-- =============================================================================

-- Function to get puzzle information by hash
CREATE OR REPLACE FUNCTION get_puzzle_by_hash(
  target_puzzle_hash BYTEA
)
RETURNS TABLE(
  id INTEGER,
  puzzle_hash BYTEA,
  puzzle_reveal BYTEA,
  reveal_size INTEGER,
  first_seen_block BIGINT
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    p.id,
    p.puzzle_hash,
    p.puzzle_reveal,
    p.reveal_size,
    p.first_seen_block
  FROM puzzles p
  WHERE p.puzzle_hash = target_puzzle_hash;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get puzzle reveal by hash
CREATE OR REPLACE FUNCTION get_puzzle_reveal(target_puzzle_hash_hex TEXT)
RETURNS TABLE(
  id INTEGER,
  puzzle_reveal BYTEA,
  reveal_size INTEGER
) AS $$
DECLARE
  target_hash BYTEA;
BEGIN
  target_hash := decode(replace(target_puzzle_hash_hex, '0x', ''), 'hex');
  
  RETURN QUERY
  SELECT p.id, p.puzzle_reveal, p.reveal_size
  FROM puzzles p
  WHERE p.puzzle_hash = target_hash;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get puzzle statistics
CREATE OR REPLACE FUNCTION get_puzzle_stats()
RETURNS TABLE(
  total_puzzles BIGINT,
  puzzles_with_reveals BIGINT,
  puzzles_without_reveals BIGINT,
  avg_reveal_size DECIMAL,
  total_reveal_bytes BIGINT
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    COUNT(*)::BIGINT as total_puzzles,
    COUNT(puzzle_reveal)::BIGINT as puzzles_with_reveals,
    (COUNT(*) - COUNT(puzzle_reveal))::BIGINT as puzzles_without_reveals,
    ROUND(AVG(reveal_size)::numeric, 2) as avg_reveal_size,
    COALESCE(SUM(reveal_size), 0)::BIGINT as total_reveal_bytes
  FROM puzzles;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get most common puzzles with pagination
CREATE OR REPLACE FUNCTION get_common_puzzles(
  page INTEGER DEFAULT 1,
  page_size INTEGER DEFAULT 100
)
RETURNS TABLE(
  puzzle_hash_hex TEXT,
  reveal_size INTEGER,
  first_seen_block BIGINT,
  usage_count BIGINT
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    encode(p.puzzle_hash, 'hex') as puzzle_hash_hex,
    p.reveal_size,
    p.first_seen_block,
    COUNT(s.coin_id) as usage_count
  FROM puzzles p
  LEFT JOIN spends s ON p.id = s.puzzle_id
  WHERE p.puzzle_reveal IS NOT NULL
  GROUP BY p.id, p.puzzle_hash, p.reveal_size, p.first_seen_block
  ORDER BY usage_count DESC, p.reveal_size DESC
  LIMIT page_size OFFSET (page - 1) * page_size;
END;
$$ LANGUAGE plpgsql STABLE;

-- =============================================================================
-- Solution Management Functions
-- =============================================================================

-- Function to get solution information by hash
CREATE OR REPLACE FUNCTION get_solution_by_hash(
  target_solution_hash BYTEA
)
RETURNS TABLE(
  id INTEGER,
  solution_hash BYTEA,
  solution BYTEA,
  solution_size INTEGER,
  first_seen_block BIGINT
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    s.id,
    s.solution_hash,
    s.solution,
    s.solution_size,
    s.first_seen_block
  FROM solutions s
  WHERE s.solution_hash = target_solution_hash;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get solution by hash hex
CREATE OR REPLACE FUNCTION get_solution_by_hash_hex(target_solution_hash_hex TEXT)
RETURNS TABLE(
  id INTEGER,
  solution BYTEA,
  solution_size INTEGER
) AS $$
DECLARE
  target_hash BYTEA;
BEGIN
  target_hash := decode(replace(target_solution_hash_hex, '0x', ''), 'hex');
  
  RETURN QUERY
  SELECT s.id, s.solution, s.solution_size
  FROM solutions s
  WHERE s.solution_hash = target_hash;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get solution statistics
CREATE OR REPLACE FUNCTION get_solution_stats()
RETURNS TABLE(
  total_solutions BIGINT,
  solutions_with_data BIGINT,
  solutions_without_data BIGINT,
  avg_solution_size DECIMAL,
  total_solution_bytes BIGINT
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    COUNT(*)::BIGINT as total_solutions,
    COUNT(solution)::BIGINT as solutions_with_data,
    (COUNT(*) - COUNT(solution))::BIGINT as solutions_without_data,
    ROUND(AVG(solution_size)::numeric, 2) as avg_solution_size,
    COALESCE(SUM(solution_size), 0)::BIGINT as total_solution_bytes
  FROM solutions;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get most common solutions with pagination
CREATE OR REPLACE FUNCTION get_common_solutions(
  page INTEGER DEFAULT 1,
  page_size INTEGER DEFAULT 100
)
RETURNS TABLE(
  solution_hash_hex TEXT,
  solution_size INTEGER,
  first_seen_block BIGINT,
  usage_count BIGINT
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    encode(sol.solution_hash, 'hex') as solution_hash_hex,
    sol.solution_size,
    sol.first_seen_block,
    COUNT(s.coin_id) as usage_count
  FROM solutions sol
  LEFT JOIN spends s ON sol.id = s.solution_id
  WHERE sol.solution IS NOT NULL
  GROUP BY sol.id, sol.solution_hash, sol.solution_size, sol.first_seen_block
  ORDER BY usage_count DESC, sol.solution_size DESC
  LIMIT page_size OFFSET (page - 1) * page_size;
END;
$$ LANGUAGE plpgsql STABLE;

-- =============================================================================
-- Coin Relationship Functions
-- =============================================================================

-- Function to get parent coin information
CREATE OR REPLACE FUNCTION get_parent_coin(child_coin_id TEXT)
RETURNS TABLE(
  parent_coin_id VARCHAR(64),
  parent_puzzle_hash_hex TEXT,
  parent_amount NUMERIC,
  parent_created_block BIGINT,
  parent_parent_coin_info_hex TEXT
) AS $$
DECLARE
  child_parent_coin_info BYTEA;
BEGIN
  -- First get the parent_coin_info from the child coin
  SELECT parent_coin_info INTO child_parent_coin_info
  FROM coins
  WHERE coin_id = child_coin_id;
  
  -- If child coin not found, return empty result
  IF child_parent_coin_info IS NULL THEN
    RETURN;
  END IF;
  
  -- Find the parent coin using the parent_coin_info
  RETURN QUERY
  SELECT 
    c.coin_id as parent_coin_id,
    encode(c.puzzle_hash, 'hex') as parent_puzzle_hash_hex,
    c.amount as parent_amount,
    c.created_block as parent_created_block,
    encode(c.parent_coin_info, 'hex') as parent_parent_coin_info_hex
  FROM coins c
  WHERE c.coin_id = encode(child_parent_coin_info, 'hex');
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get children coins of a parent with pagination
CREATE OR REPLACE FUNCTION get_children_coins(
  parent_coin_id TEXT,
  page INTEGER DEFAULT 1,
  page_size INTEGER DEFAULT 100
)
RETURNS TABLE(
  child_coin_id VARCHAR(64),
  child_puzzle_hash_hex TEXT,
  child_amount NUMERIC,
  child_created_block BIGINT
) AS $$
DECLARE
  parent_coin_info_bytes BYTEA;
BEGIN
  -- Convert parent coin_id to BYTEA for comparison
  parent_coin_info_bytes := decode(parent_coin_id, 'hex');
  
  -- Find all children coins
  RETURN QUERY
  SELECT 
    c.coin_id as child_coin_id,
    encode(c.puzzle_hash, 'hex') as child_puzzle_hash_hex,
    c.amount as child_amount,
    c.created_block as child_created_block
  FROM coins c
  WHERE c.parent_coin_info = parent_coin_info_bytes
  ORDER BY c.created_block DESC, c.amount DESC
  LIMIT page_size OFFSET (page - 1) * page_size;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get coin family tree (parent + children)
CREATE OR REPLACE FUNCTION get_coin_family(target_coin_id TEXT)
RETURNS TABLE(
  relation_type TEXT,
  coin_id VARCHAR(64),
  puzzle_hash_hex TEXT,
  amount NUMERIC,
  created_block BIGINT,
  is_spent BOOLEAN
) AS $$
BEGIN
  -- Return parent coin
  RETURN QUERY
  SELECT 
    'parent' as relation_type,
    parent_coin_id,
    parent_puzzle_hash_hex,
    parent_amount,
    parent_created_block,
    EXISTS(SELECT 1 FROM spends WHERE spends.coin_id = parent_coin_id) as is_spent
  FROM get_parent_coin(target_coin_id);
  
  -- Return the target coin itself
  RETURN QUERY
  SELECT 
    'self' as relation_type,
    c.coin_id,
    encode(c.puzzle_hash, 'hex') as puzzle_hash_hex,
    c.amount,
    c.created_block,
    EXISTS(SELECT 1 FROM spends WHERE spends.coin_id = c.coin_id) as is_spent
  FROM coins c
  WHERE c.coin_id = target_coin_id;
  
  -- Return children coins
  RETURN QUERY
  SELECT 
    'child' as relation_type,
    child_coin_id,
    child_puzzle_hash_hex,
    child_amount,
    child_created_block,
    EXISTS(SELECT 1 FROM spends WHERE spends.coin_id = child_coin_id) as is_spent
  FROM get_children_coins(target_coin_id);
END;
$$ LANGUAGE plpgsql STABLE; 