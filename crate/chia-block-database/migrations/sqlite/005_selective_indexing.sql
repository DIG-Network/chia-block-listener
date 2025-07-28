-- =============================================================================
-- Migration 005: Selective Indexing System (SQLite)
-- =============================================================================
-- Creates control tables for selective indexing of collections, assets, and puzzle hashes

-- =============================================================================
-- Selective Indexing Control Tables
-- =============================================================================

-- Control which NFT collections to index (if empty, index all)
CREATE TABLE IF NOT EXISTS indexed_collection_ids (
  collection_id TEXT PRIMARY KEY,
  enabled INTEGER DEFAULT 1, -- SQLite uses INTEGER for BOOLEAN
  created_at TEXT DEFAULT (datetime('now')),
  notes TEXT
);

-- Control which CAT assets to index (if empty, index all)
CREATE TABLE IF NOT EXISTS indexed_asset_ids (
  asset_id TEXT PRIMARY KEY,
  enabled INTEGER DEFAULT 1, -- SQLite uses INTEGER for BOOLEAN
  created_at TEXT DEFAULT (datetime('now')),
  notes TEXT
);

-- Control which puzzle hashes to index (if empty, index all)
CREATE TABLE IF NOT EXISTS indexed_puzzle_hashes (
  puzzle_hash_hex TEXT PRIMARY KEY,
  enabled INTEGER DEFAULT 1, -- SQLite uses INTEGER for BOOLEAN
  created_at TEXT DEFAULT (datetime('now')),
  notes TEXT
); 