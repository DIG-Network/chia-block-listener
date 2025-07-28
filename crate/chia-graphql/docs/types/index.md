# GraphQL Type Definitions

This section provides a complete reference for all GraphQL types used in the Chia GraphQL API. Types are organized by domain for easy navigation.

## Type Categories

### [Core Types](core.md)
Basic blockchain data types used throughout the API.
- `CoinInfo` - Coin representation
- `BlockInfo` - Block data
- `BalanceInfo` - Balance information

### [Address Types](addresses.md)
Types for address analytics and activity tracking.
- `ActiveAddress` - Active address metrics
- `AddressProfile` - Comprehensive address analysis
- `AddressTransaction` - Transaction history
- `AddressRiskAssessment` - Risk evaluation
- And 10+ more specialized types

### [Balance Types](balances.md)
Types for balance distribution and wealth analysis.
- `BalanceDistribution` - Statistical distribution
- `BalancePercentiles` - Wealth percentiles
- `RichestAddress` - Top balance holders
- `BalanceChange` - Balance evolution

### [CAT Types](cats.md)
Chia Asset Token types.
- `CatToken` - Token information
- `CatBalance` - Token balances
- `TokenAnalytics` - Market metrics
- `TradingVelocity` - Trading patterns
- And 8+ more CAT-specific types

### [NFT Types](nfts.md)
Non-Fungible Token types.
- `NftCollection` - Collection data
- `NftOwnership` - Ownership records
- `CollectionAnalytics` - Collection metrics
- `NftRarity` - Rarity analysis
- And 5+ more NFT types

### [Network Types](network.md)
Network-wide statistics and metrics.
- `NetworkStats` - Overall statistics
- `BlockProductionStats` - Block timing
- `UnspentCoinAnalysis` - Coinset analysis
- `ThroughputAnalysis` - Performance metrics

### [Puzzle Types](puzzles.md)
CLVM puzzle analysis types.
- `PuzzleInfo` - Puzzle data
- `PuzzleStats` - Puzzle statistics
- `CommonPuzzle` - Usage patterns
- `PuzzleComplexityAnalysis` - Complexity metrics

### [Solution Types](solutions.md)
Solution pattern types.
- `SolutionInfo` - Solution data
- `SolutionStats` - Solution statistics
- `CommonSolution` - Usage patterns
- `SolutionComplexityAnalysis` - Complexity analysis

### [Temporal Types](temporal.md)
Time-based analysis types.
- `GrowthTrend` - Growth patterns
- `ActivityHeatmap` - Activity distribution
- `SeasonalPattern` - Seasonal trends
- `CoinAggregation` - Time-based aggregation

### [Transaction Types](transactions.md)
Transaction analysis types.
- `SpendVelocityAnalysis` - Spend patterns
- `ComplexTransactionPattern` - Complex transactions
- `TransactionFeeAnalysis` - Fee metrics
- `TransactionPatternByType` - Pattern classification

### [Analytics Types](analytics.md)
Combined analytics types.
- `BlockchainOverview` - Comprehensive overview
- `HealthMetrics` - Blockchain health
- `PerformanceMetrics` - Performance tracking
- `EconomicMetrics` - Economic indicators

## Common Type Patterns

### Nullable vs Non-Nullable

- Types ending with `!` are non-nullable
- Types without `!` may return null
- Lists are typically `[Type!]!` (non-null list of non-null items)

### Pagination

Most list types support pagination:

```graphql
type Query {
  addresses {
    mostActiveAddresses(
      page: Int = 1
      pageSize: Int = 20
    ): [ActiveAddress!]!
  }
}
```

### Time-Based Types

Many types include time-related fields:
- Block heights for blockchain time
- ISO 8601 timestamps for calendar time
- Relative time periods (e.g., "Last 30 days")

### Value Representation

All monetary values are in mojos (smallest unit):
- 1 XCH = 1,000,000,000,000 mojos
- Use `BigInt` type for large values
- Percentages as `Float` (0-100)

## Type Naming Conventions

1. **Info suffix**: Basic data types (`CoinInfo`, `BlockInfo`)
2. **Stats suffix**: Statistical aggregations (`NetworkStats`, `PuzzleStats`)
3. **Analysis suffix**: Complex analytics (`SpendVelocityAnalysis`)
4. **Profile suffix**: Comprehensive views (`AddressProfile`)
5. **Metrics suffix**: Measurement types (`HealthMetrics`)

## GraphQL Scalars

Custom scalar types used:

- `BigInt` - Large integers (coin amounts)
- `String` - Text, including hex strings
- `Int` - 32-bit integers
- `Float` - Floating point numbers
- `Boolean` - True/false values

## Enum Types

Common enumerations:

```graphql
enum TransactionType {
  RECEIVE
  SPEND
  FEE
}

enum RiskCategory {
  LOW
  MEDIUM
  HIGH
  CRITICAL
}

enum AddressType {
  REGULAR
  SMART_CONTRACT
  MULTISIG
  NFT_OWNER
  CAT_HOLDER
}
```

## Best Practices

1. **Always check nullable fields** before using values
2. **Use fragments** for common field selections
3. **Request only needed fields** to optimize performance
4. **Understand value units** (mojos vs XCH)
5. **Handle large numbers** appropriately in your client

## Next Steps

- Explore specific type categories for detailed field descriptions
- See [Query Documentation](../queries/core.md) for usage examples
- Check [Schema Overview](../schema-overview.md) for architecture 