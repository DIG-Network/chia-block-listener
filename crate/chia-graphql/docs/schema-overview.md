# Chia GraphQL Schema Overview

## Architecture

The Chia GraphQL API is structured as a hierarchical schema with a root query object that provides access to specialized resolvers. Each resolver focuses on a specific domain of blockchain data.

```
QueryRoot
├── Direct Queries
│   ├── currentHeight
│   ├── coin(coinId)
│   └── block(height)
│
└── Namespaced Resolvers
    ├── core         → Basic blockchain operations
    ├── addresses    → Address analytics and tracking
    ├── balances     → Balance distribution analysis
    ├── cats         → Chia Asset Token operations
    ├── nfts         → Non-Fungible Token queries
    ├── network      → Network statistics
    ├── puzzles      → CLVM puzzle analysis
    ├── solutions    → Solution pattern analysis
    ├── temporal     → Time-based analytics
    ├── transactions → Transaction analysis
    └── analytics    → Combined analytics
```

## Query Organization

### 1. Direct Queries

Direct queries are available at the root level for common operations:

```graphql
type Query {
  # Get current blockchain height
  currentHeight: Int!
  
  # Get a specific coin by ID
  coin(coinId: String!): Coin
  
  # Get a specific block by height
  block(height: Int!): Block
}
```

### 2. Namespaced Resolvers

Each namespace groups related functionality:

#### Core Namespace (`core`)
Basic blockchain data queries for coins, blocks, and balances.

```graphql
query {
  core {
    coinById(coinId: String!): CoinInfo
    coinsByPuzzleHash(puzzleHashHex: String!, page: Int, pageSize: Int): [CoinInfo!]!
    blockByHeight(height: Int!): BlockInfo
    recentBlocks(limit: Int): [BlockInfo!]!
    balanceByPuzzleHash(puzzleHashHex: String!): BalanceInfo
  }
}
```

#### Addresses Namespace (`addresses`)
Comprehensive address analysis including activity, patterns, and risk assessment.

```graphql
query {
  addresses {
    addressActivity(puzzleHashHex: String!, daysBack: Int): AddressActivity
    mostActiveAddresses(page: Int, pageSize: Int): [ActiveAddress!]!
    addressProfile(puzzleHashHex: String!): AddressProfile
    addressRiskAssessment(puzzleHashHex: String!): AddressRiskAssessment
    # ... and 9 more methods
  }
}
```

#### Network Namespace (`network`)
Network-wide statistics and blockchain health metrics.

```graphql
query {
  network {
    networkStats: NetworkStats!
    blockProductionStats: BlockProductionStats!
    growthMetrics(daysBack: Int): [NetworkGrowthMetrics!]!
    unspentCoinAnalysis: UnspentCoinAnalysis!
    throughputAnalysis(hoursBack: Int): ThroughputAnalysis!
  }
}
```

## Type System

The GraphQL schema uses strongly typed objects for all responses. Types are organized by domain:

### Core Types
- `CoinInfo` - Basic coin data
- `BlockInfo` - Block information
- `BalanceInfo` - Balance details

### Analytics Types
- `NetworkStats` - Overall network statistics
- `AddressProfile` - Comprehensive address analysis
- `TransactionPattern` - Transaction behavior patterns

### Token Types
- `CatToken` - CAT token information
- `NftCollection` - NFT collection data
- `TokenAnalytics` - Token market metrics

## Pagination

Most list queries support pagination with consistent parameters:

```graphql
query {
  addresses {
    mostActiveAddresses(
      page: 1        # Page number (default: 1)
      pageSize: 20   # Items per page (default: 20)
    ) {
      puzzleHashHex
      totalTransactions
    }
  }
}
```

## Time-Based Queries

Many analytics queries accept time parameters:

```graphql
query {
  network {
    growthMetrics(
      daysBack: 30  # Number of days to analyze
    ) {
      metricDate
      totalBlocks
      newAddresses
    }
  }
}
```

## Null Handling

The schema uses GraphQL's type system for null handling:

- `!` indicates non-nullable fields
- Fields without `!` may return null
- Lists are non-null but may be empty: `[Type!]!`

## Error Handling

All queries follow consistent error handling:

```json
{
  "data": null,
  "errors": [{
    "message": "Error description",
    "path": ["namespace", "method"],
    "extensions": {
      "code": "ERROR_CODE",
      "details": "Additional context"
    }
  }]
}
```

## Query Complexity

The API doesn't currently enforce query complexity limits, but best practices include:

1. Limit nested queries to 3-4 levels
2. Use pagination for large datasets
3. Request only needed fields
4. Batch related queries together

## Database Compatibility

All queries automatically adapt to the underlying database:

- **PostgreSQL**: Full feature support with advanced analytics
- **SQLite**: Compatible with some limitations on statistical functions

The API handles dialect differences transparently.

## Security Considerations

- All queries are **read-only** - no mutations allowed
- Input validation on all parameters
- Hex string validation for addresses and IDs
- SQL injection protection via parameterized queries

## Performance Features

1. **Efficient Joins**: Queries use optimized JOIN strategies
2. **Index Usage**: All queries utilize database indexes
3. **Pagination**: Prevents large result sets
4. **Query Planning**: Database-specific optimizations

## Schema Evolution

The schema follows semantic versioning:

- **Backward Compatible**: New fields added as nullable
- **Deprecation**: Fields marked with `@deprecated` directive
- **Breaking Changes**: Major version bumps only

## Introspection

The GraphQL schema supports full introspection:

```graphql
query {
  __schema {
    types {
      name
      description
      fields {
        name
        type {
          name
        }
      }
    }
  }
}
```

Use GraphQL playground's documentation explorer for interactive browsing.

## Next Steps

- Review [Core Queries](queries/core.md) for basic blockchain operations
- Explore [Type Definitions](types/index.md) for complete type reference
- See [Query Examples](examples/index.md) for practical usage patterns 