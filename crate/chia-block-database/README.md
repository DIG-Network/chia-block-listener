# Chia Block Database

A Rust library for managing Chia blockchain data with support for both SQLite and PostgreSQL databases.

## Features

- **Dual Database Support**: Works with both SQLite and PostgreSQL
- **Automatic Migrations**: Manages database schema migrations automatically
- **Migration Tracking**: Tracks applied migrations to avoid re-running them
- **WAL Mode**: Automatically enables WAL mode for SQLite for better performance
- **Foreign Key Support**: Ensures referential integrity with foreign key constraints
- **Optimized Indexes**: Comprehensive indexing strategy for optimal query performance
- **Organized Namespaces**: Logical grouping of operations for better code organization
- **Puzzle/Solution Abstraction**: Automatic normalization of puzzles and solutions

## Architecture

The library is organized into several layers:

### 1. CRUD Layer (`src/crud/`)
Direct database operations for each table:
- `BlocksCrud` - Block operations
- `CoinsCrud` - Coin operations  
- `SpendsCrud` - Spend operations (with puzzle/solution abstraction)
- `NftsCrud` - NFT operations
- `CatsCrud` - CAT operations
- `SystemPreferencesCrud` - System preference operations
- `SyncStatusCrud` - Sync status operations

### 2. Functions Layer (`src/functions/`)
Application functions that either call PostgreSQL stored procedures or implement logic for SQLite:
- `CoreFunctions` - Core blockchain functions
- `PuzzleFunctions` - Puzzle management and analytics
- `SolutionFunctions` - Solution management and analytics  
- `BalanceFunctions` - Balance calculation functions
- `CatFunctions` - CAT-specific functions
- `NftFunctions` - NFT-specific functions

### 3. Namespace Layer (`src/namespaces/`)
Organized access to related functionality:
- `BlockchainNamespace` - Blocks, coins, spends operations
- `AssetsNamespace` - CATs and NFTs operations
- `AnalyticsNamespace` - Puzzles, solutions, balances analytics
- `SystemNamespace` - Sync status and system preferences

## Quick Start

### Basic Usage

```rust
use chia_block_database::{ChiaBlockDatabase, DatabaseType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a SQLite database (auto-detected from URL)
    let db = ChiaBlockDatabase::new("sqlite:blockchain.db").await?;
    
    // Or create a PostgreSQL database
    let db = ChiaBlockDatabase::new("postgres://user:pass@localhost/chia").await?;
    
    // Access operations through organized namespaces
    let pool = db.pool();
    
    // Blockchain operations
    let block = db.blockchain().blocks().get_by_height(pool, 1000).await?;
    let coins = db.blockchain().coins().get_by_puzzle_hash(pool, &puzzle_hash, 1, 100).await?;
    
    // Spends with automatic puzzle/solution management
    use chia_block_database::crud::blocks::Block;
    use chia_block_database::crud::spends::Spend;
    
    let spend = Spend {
        coin_id: "abc123".to_string(),
        puzzle_hash: Some(puzzle_hash.to_vec()),
        puzzle_reveal: Some(puzzle_reveal.to_vec()),
        solution_hash: None, // Will be calculated automatically
        solution: Some(solution_data.to_vec()),
        spent_block: 1000,
    };
    db.blockchain().spends().create(pool, &spend).await?;
    
    // Asset operations
    let cat_balances = db.assets().cat_functions(); // Access CAT functions
    let nft_functions = db.assets().nft_functions(); // Access NFT functions
    
    // Analytics operations  
    let puzzle_stats = db.analytics().puzzles(); // Access puzzle analytics
    let solution_stats = db.analytics().solutions(); // Access solution analytics
    let balance_functions = db.analytics().balances(); // Access balance functions
    
    // System operations
    let sync_status = db.system().sync_status(); // Access sync status
    let preferences = db.system().preferences(); // Access system preferences
    
    // Close the database when done
    db.close().await;
    
    Ok(())
}
```

### Key Abstractions

#### Puzzle/Solution Normalization
The `SpendsCrud` automatically handles puzzle and solution normalization:

```rust
// When creating a spend, puzzles and solutions are automatically normalized
let spend = Spend {
    coin_id: "coin123".to_string(),
    puzzle_hash: Some(puzzle_hash),
    puzzle_reveal: Some(puzzle_reveal), // Stored once in puzzles table
    solution: Some(solution_data),      // Stored once in solutions table  
    // ... other fields
};

// The CRUD layer handles:
// 1. Checking if puzzle/solution already exists
// 2. Creating new entries if needed
// 3. Referencing by ID in the spends table
db.blockchain().spends().create(pool, &spend).await?;
```

#### Database Type Abstraction
Functions automatically use the correct implementation:

```rust
// Same code works for both SQLite and PostgreSQL
let blocks = db.blockchain().blocks().get_range(pool, 1000, 2000, 1, 100).await?;

// Internally handles:
// - PostgreSQL: Uses $1, $2, $3 parameter binding
// - SQLite: Uses ?1, ?2, ?3 parameter binding
// - Timestamp formatting differences
// - SQL syntax differences
```

## Database Schema

The database includes tables for:

- **Blocks**: Blockchain height, weight, header hash, timestamp
- **Coins**: Coin records with puzzle hash, amount, parent relationships
- **Puzzles**: Normalized puzzle reveals with size tracking (automatically managed)
- **Solutions**: Normalized solutions with size tracking (automatically managed)
- **Spends**: Spend records linking coins to blocks via puzzle/solution IDs
- **NFTs**: NFT collections, individual NFTs, and ownership history
- **CATs**: CAT assets and individual CAT coins
- **System Preferences**: Application configuration
- **Sync Status**: Blockchain synchronization monitoring

## Migration System

The library automatically runs database migrations on startup. Migrations are stored in:

- `migrations/postgres/` - PostgreSQL-specific migrations with stored procedures
- `migrations/sqlite/` - SQLite-compatible migrations

Each migration is tracked in the `migration_history` table to ensure they're only run once.

## Database Differences

### PostgreSQL Features
- Full PL/pgSQL stored procedures (marked as `STABLE` for Supabase GraphQL)
- Materialized views for performance
- Advanced indexing with INCLUDE clauses
- Native array types (`TEXT[]`)
- JSONB for optimized JSON storage

### SQLite Adaptations
- Functions implemented in application layer
- Regular tables instead of materialized views
- Composite indexes instead of INCLUDE clauses
- JSON strings instead of array types
- WAL mode enabled automatically
- Foreign keys enabled automatically

## Configuration

### SQLite Options
SQLite databases automatically get these optimizations:
- `journal_mode=WAL` for better concurrency
- `synchronous=NORMAL` for performance
- `cache_size=64000` for memory efficiency
- `foreign_keys=ON` for referential integrity

### Connection Pooling
The library uses sqlx connection pooling with:
- Maximum 10 connections per pool
- Automatic connection management
- Support for both database types

## Error Handling

```rust
use chia_block_database::{DatabaseError, Result};

match ChiaBlockDatabase::new("invalid_url").await {
    Ok(db) => { /* use database */ }
    Err(DatabaseError::InvalidUrl(url)) => {
        eprintln!("Invalid database URL: {}", url);
    }
    Err(DatabaseError::Migration(msg)) => {
        eprintln!("Migration failed: {}", msg);
    }
    Err(e) => {
        eprintln!("Database error: {}", e);
    }
}
```

## Testing

Run tests with:

```bash
cargo test
```

Tests include:
- Database creation and migration
- WAL mode verification for SQLite
- Migration idempotency
- Basic table structure validation

## Extending the Library

### Adding New CRUD Operations

1. Add methods to existing CRUD structs in `src/crud/`
2. Implement both PostgreSQL and SQLite variants
3. Handle parameter binding differences (`$1` vs `?1`)

### Adding New Functions

1. Create functions in `src/functions/` modules
2. For PostgreSQL: Call existing stored procedures
3. For SQLite: Implement equivalent logic in Rust
4. Add to appropriate namespace in `src/namespaces/`

### Adding New Tables

1. Add migrations to both `migrations/postgres/` and `migrations/sqlite/`
2. Create CRUD module in `src/crud/`
3. Add function module in `src/functions/`
4. Add to appropriate namespace or create new one

## License

This project is licensed under the same terms as the parent Chia Block Listener project. 