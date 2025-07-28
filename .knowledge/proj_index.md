# Project Architecture Index

## Module Hierarchy

### Root Crates
- `chia-block-listener/` - Main NAPI module for Node.js
- `crate/chia-generator-parser/` - Block parsing utilities
- `crate/chia-block-database/` - Database storage and write operations
- `crate/chia-graphql/` - GraphQL API for blockchain exploration

### chia-block-listener (Main Module)
- `src/lib.rs` - NAPI module exports
- `src/peer.rs` - WebSocket peer connection logic
- `src/event_emitter.rs` - Event system for JS integration
- `src/error.rs` - Error types
- `src/protocol.rs` - Chia protocol message handling
- `src/tls.rs` - TLS certificate handling

### chia-generator-parser
- `src/lib.rs` - Block parsing exports
- `src/error.rs` - Parser error types
- Direct integration with chia-protocol types

### chia-block-database
- `src/lib.rs` - Database exports
- `src/database.rs` - Core database operations
- `src/functions/` - Database query functions (being migrated to GraphQL)
- `src/namespaces/` - Function organization
- `migrations/` - Database schema migrations

### chia-graphql
- `src/lib.rs` - GraphQL API exports
- `src/schema/` - GraphQL schema definitions
  - `query.rs` - Root query object
  - `types/` - GraphQL type definitions
- `src/resolvers/` - Query implementations
  - `core.rs` - Basic blockchain queries (✅ Complete)
  - `addresses.rs` - Address analytics (✅ Complete)
  - Others - (⏳ In progress)
- `src/database.rs` - Database configuration
- `src/error.rs` - GraphQL error handling

## Key Interfaces

### NAPI Bindings
- `EventEmitter` - JS event interface
- Block event types (PeerConnected, BlockReceived, etc.)

### GraphQL API
- `ChiaGraphql` - Main GraphQL instance
- Query namespaces: core, addresses, balances, cats, nfts, etc.
- Support for both PostgreSQL and SQLite

## Dependencies Between Modules
- `chia-block-listener` → `chia-generator-parser` (block parsing)
- `chia-block-listener` → `chia-block-database` (data storage)
- `chia-graphql` → Database (read queries)
- All → `chia-protocol` types 