# Chia Block Listener Project Context

## Current State
- Rust-based WebSocket client connecting to Chia full nodes
- Uses NAPI for Node.js bindings
- Integrated chia-generator-parser crate for block parsing
- Can monitor real-time blocks with full coin_spends extraction working
- **COMPLETED: Comprehensive GraphQL API crate for complete blockchain exploration**

## Architecture
- `src/peer.rs`: WebSocket connection and block handling
- `src/event_emitter.rs`: NAPI bindings and JS event system
- `src/tls.rs`: Comprehensive TLS certificate handling and WebSocket connector creation
- `crate/chia-generator-parser/`: Block parsing using chia-protocol types
- `crate/chia-block-database/`: Database storage and write operations
- `crate/chia-peer-tls/`: **NEW** Shared TLS and peer connection functionality for reuse across components
- **`crate/chia-graphql/`: COMPLETE GraphQL API with 11 resolvers and 65+ query methods**
- Direct integration with chia-protocol crate (v0.26.0)

## GraphQL Implementation Status: **COMPLETE** ✅
- **ALL 11 RESOLVERS IMPLEMENTED**: Complete GraphQL crate structure with async-graphql
- **Type System**: Comprehensive GraphQL types for all blockchain data and analytics
- **Resolvers Completed (11/11)**:
  - **Core queries** (5 methods): Basic coin, block, and balance queries
  - **Address analytics** (13 methods): Comprehensive address analysis and patterns  
  - **Balance analytics** (6 methods): Distribution analysis and wealth metrics
  - **CAT analytics** (11 methods): Token analysis, trading, and market metrics
  - **NFT queries** (5 methods): Collection analysis, ownership tracking, marketplace data
  - **Network analytics** (5 methods): Blockchain statistics using **Chia coinset model**
  - **Temporal analytics** (4 methods): Time-based trends and seasonal patterns
  - **Transaction analytics** (5 methods): Spend velocity, complexity, and fee analysis
  - **Puzzle resolver** (4 methods): CLVM puzzle analysis and complexity metrics
  - **Solution resolver** (5 methods): Solution analysis, usage patterns, and complexity
  - **Analytics wrapper** (7 sub-resolvers): Unified access to all specialized analytics
- **Database Support**: Both PostgreSQL and SQLite via AnyPool with dialect-specific queries
- **Design Principle**: GraphQL is 100% READ-ONLY - no mutations allowed
  - All write/mutation operations remain exclusively in chia-block-database
  - Removed all refresh/mutation methods from GraphQL resolvers
  - GraphQL focuses purely on data exploration and comprehensive analytics
- **Architecture-Specific**: Uses proper **Chia coinset model** terminology throughout
  - `unspent_coin_analysis` instead of `utxo_analysis`
  - All analysis reflects Chia's unique coin creation/spend model
  - No UTXO references anywhere in the API

## Recent Progress
- Successfully refactored to use chia-protocol types directly
- Eliminated manual byte parsing and redundant serialization
- Built and tested with real network connections
- Created real-time block monitor (coin-monitor.js)
- **COMPLETED: Implemented full CLVM execution using chia-consensus**
- **COMPLETED: TypeScript definitions now include all event types**
- **COMPLETED: Comprehensive GraphQL API with 65+ read-only query methods across 11 resolvers**
- **COMPLETED: Fixed UTXO terminology to use proper Chia coinset model**
- **COMPLETED: Resolved all compilation errors and type conflicts**

## Completed Implementations
- Full CLVM execution using run_block_generator2 for coin extraction
- Real puzzle reveals and solutions from generator bytecode
- Proper TypeScript event type definitions with constants
- Event interfaces for PeerConnected, PeerDisconnected, BlockReceived
- Removed generatorBytecode from block events (not needed)
- Log file streaming for coin-monitor.js
- Removed processTransactionGenerator method (replaced by chia-generator-parser)
- Changed peer IDs from numeric to IP:port strings for better identification
- Added typed event emitters with full TypeScript support via post-build script
- Removed redundant Block type - all APIs now use BlockReceivedEvent consistently
- **COMPLETE: GraphQL schema with 11 resolvers and 65+ read-only methods**
- **COMPLETE: Proper Chia coinset model terminology throughout GraphQL API**
- **COMPLETE: All resolver compilation and type system working correctly**

## Dependencies
- chia-protocol = "0.26.0"
- chia-traits = "0.26.0"  
- chia-ssl = "0.26.0"
- chia-consensus = "0.26.0" (for CLVM execution)
- chia-bls = "0.26.0" (for signature handling)
- clvmr = "0.8.0" (for low-level CLVM operations)
- clvm-utils = "0.26.0" (for tree hashing)
- hex = "0.4" (for hex encoding/decoding)
- async-graphql = "7.0" (for GraphQL API)
- sqlx = "0.8" (for database queries)

## Testing
- example-get-block-by-height.js: Requests specific blocks
- coin-monitor.js: Monitors real-time blocks
- GraphQL playground: Available at http://localhost:8080/playground when running example server
- **Ready for**: Integration tests for all 11 GraphQL resolvers

## Production Readiness
- **GraphQL API**: Complete with error handling, pagination, type safety
- **Database Agnostic**: PostgreSQL and SQLite support with optimized queries
- **Scalable Architecture**: Modular resolvers with clean separation of concerns
- **Performance**: Efficient queries with pagination and caching opportunities
- **Monitoring Ready**: Structured for metrics, logging, and observability 