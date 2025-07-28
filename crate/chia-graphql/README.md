# Chia GraphQL API Documentation

## Overview

The Chia GraphQL API provides a comprehensive, read-only interface for querying blockchain data from Chia. This API offers powerful analytics, detailed blockchain exploration, and advanced querying capabilities across all aspects of the Chia blockchain including coins, blocks, CAT tokens, NFTs, and more.

### Key Features

- **Comprehensive Coverage**: 65+ query methods across 11 specialized resolvers
- **Read-Only Design**: All queries are read-only, ensuring data integrity
- **Database Agnostic**: Supports both PostgreSQL and SQLite backends
- **Chia-Specific**: Uses proper Chia coinset model terminology (not UTXO)
- **Type-Safe**: Fully typed GraphQL schema with detailed documentation
- **Performance Optimized**: Efficient queries with pagination support
- **Analytics Rich**: Advanced analytics for network health, trends, and patterns

## Table of Contents

### Core Documentation

1. **[Getting Started](docs/getting-started.md)** - Installation, setup, and basic usage
2. **[Schema Overview](docs/schema-overview.md)** - High-level view of the GraphQL schema structure
3. **[Authentication & Security](docs/authentication.md)** - API security considerations

### Query Documentation

4. **[Core Queries](docs/queries/core.md)** - Basic blockchain data (coins, blocks, balances)
5. **[Address Queries](docs/queries/addresses.md)** - Address analytics and activity tracking
6. **[Balance Queries](docs/queries/balances.md)** - Balance distribution and wealth analytics
7. **[CAT Queries](docs/queries/cats.md)** - Chia Asset Token analytics and trading metrics
8. **[NFT Queries](docs/queries/nfts.md)** - NFT collections, ownership, and marketplace data
9. **[Network Queries](docs/queries/network.md)** - Network statistics and blockchain health
10. **[Puzzle Queries](docs/queries/puzzles.md)** - CLVM puzzle analysis and complexity metrics
11. **[Solution Queries](docs/queries/solutions.md)** - Solution patterns and usage analysis
12. **[Temporal Queries](docs/queries/temporal.md)** - Time-based trends and seasonal patterns
13. **[Transaction Queries](docs/queries/transactions.md)** - Transaction analysis and fee metrics
14. **[Analytics Queries](docs/queries/analytics.md)** - Combined analytics and blockchain overview

### Type Reference

15. **[Type Definitions](docs/types/index.md)** - Complete reference of all GraphQL types
    - [Core Types](docs/types/core.md)
    - [Address Types](docs/types/addresses.md)
    - [Balance Types](docs/types/balances.md)
    - [CAT Types](docs/types/cats.md)
    - [NFT Types](docs/types/nfts.md)
    - [Network Types](docs/types/network.md)
    - [Puzzle Types](docs/types/puzzles.md)
    - [Solution Types](docs/types/solutions.md)
    - [Temporal Types](docs/types/temporal.md)
    - [Transaction Types](docs/types/transactions.md)
    - [Analytics Types](docs/types/analytics.md)

### Advanced Topics

16. **[Query Examples](docs/examples/index.md)** - Real-world query examples
17. **[Performance Guide](docs/performance.md)** - Query optimization tips
18. **[Migration Guide](docs/migration.md)** - Migrating from direct database queries

## Quick Start

```graphql
# Get current blockchain height
query {
  currentHeight
}

# Get coin details
query {
  coin(coinId: "0x...") {
    coinId
    amount
    puzzleHash
    createdBlock
  }
}

# Get network statistics
query {
  network {
    networkStats {
      totalBlocks
      totalCoins
      totalAddresses
      currentHeight
    }
  }
}
```

## Architecture

The GraphQL API is organized into specialized resolvers, each focusing on a specific domain:

```
┌─────────────────────────────────────────────┐
│              GraphQL Client                 │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│            Query Root                       │
├─────────────────────────────────────────────┤
│  • core      • addresses   • balances      │
│  • cats      • nfts        • network       │
│  • puzzles   • solutions   • temporal      │
│  • transactions            • analytics     │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│          Database Layer                     │
│    (PostgreSQL or SQLite)                   │
└─────────────────────────────────────────────┘
```

## Database Support

The API supports both PostgreSQL and SQLite databases with automatic query adaptation:

- **PostgreSQL**: Recommended for production use with full feature support
- **SQLite**: Suitable for development and smaller deployments

## Design Principles

1. **Read-Only**: No mutations or write operations - all data modification happens through the `chia-block-database` crate
2. **Coinset Model**: Uses Chia's coinset model terminology, not UTXO
3. **Comprehensive**: Covers all aspects of blockchain data and analytics
4. **Type-Safe**: Strong typing throughout with detailed documentation
5. **Performant**: Optimized queries with proper indexing and pagination

## Contributing

For contributing guidelines and development setup, see [CONTRIBUTING.md](../CONTRIBUTING.md).

## License

This project is licensed under the [MIT License](../LICENSE).
