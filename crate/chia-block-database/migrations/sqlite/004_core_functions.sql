-- =============================================================================
-- Migration 004: Core Functions (SQLite)
-- =============================================================================
-- Note: SQLite doesn't support stored procedures like PostgreSQL's PL/pgSQL.
-- The function logic will need to be implemented in the application layer.
-- This migration creates any necessary helper views and triggers.

-- =============================================================================
-- Helper Views for Function-like Behavior
-- =============================================================================

-- View to check if sync is paused
CREATE VIEW IF NOT EXISTS sync_paused_check AS
SELECT 
  CASE 
    WHEN LOWER(COALESCE(meta_value, 'false')) IN ('true', 'yes', '1', 'on') THEN 1
    ELSE 0
  END as is_paused
FROM system_preferences 
WHERE meta_key = 'paused_sync';

-- View to check if process should exit
CREATE VIEW IF NOT EXISTS process_exit_check AS
SELECT 
  CASE 
    WHEN LOWER(COALESCE(meta_value, 'false')) IN ('true', 'yes', '1', 'on') THEN 1
    ELSE 0
  END as should_exit
FROM system_preferences 
WHERE meta_key = 'exit_process';

-- =============================================================================
-- Triggers for Automated Updates
-- =============================================================================

-- Trigger to update system_preferences updated_at timestamp
CREATE TRIGGER IF NOT EXISTS update_system_preferences_timestamp
AFTER UPDATE ON system_preferences
FOR EACH ROW
BEGIN
  UPDATE system_preferences 
  SET updated_at = datetime('now') 
  WHERE id = NEW.id;
END;

-- =============================================================================
-- Functions Documentation
-- =============================================================================
-- The following functions from the PostgreSQL version need to be implemented
-- in the application layer since SQLite doesn't support stored procedures:
--
-- 1. get_system_preference(preference_key) -> TEXT
-- 2. is_sync_paused() -> BOOLEAN (use sync_paused_check view)
-- 3. should_exit_process() -> BOOLEAN (use process_exit_check view)
-- 4. unspent_coins_by_puzzle_hash(target_puzzle_hash, page, page_size) -> TABLE
-- 5. get_balance_by_puzzle_hash(target_puzzle_hash) -> TABLE
-- 6. refresh_xch_balances() -> BOOLEAN
-- 7. get_balances(page, page_size) -> TABLE
-- 8. get_balance_by_puzzle_hash_detail(target_puzzle_hash) -> TABLE
-- 9. get_puzzle_by_hash(target_puzzle_hash) -> TABLE
-- 10. get_puzzle_reveal(target_puzzle_hash_hex) -> TABLE
-- 11. get_puzzle_stats() -> TABLE
-- 12. get_common_puzzles(page, page_size) -> TABLE
-- 13. get_solution_by_hash(target_solution_hash) -> TABLE
-- 14. get_solution_by_hash_hex(target_solution_hash_hex) -> TABLE
-- 15. get_solution_stats() -> TABLE
-- 16. get_common_solutions(page, page_size) -> TABLE
-- 17. get_parent_coin(child_coin_id) -> TABLE
-- 18. get_children_coins(parent_coin_id, page, page_size) -> TABLE
-- 19. get_coin_family(target_coin_id) -> TABLE
--
-- These functions will be implemented as methods on the ChiaBlockDatabase class
-- to provide the same functionality as the PostgreSQL stored procedures. 