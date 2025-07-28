# Network Queries

The `network` namespace provides blockchain-wide statistics, health metrics, and performance analysis. It uses Chia's coinset model (not UTXO) for all coin-related analytics.

## Available Queries

### 1. `networkStats`

Get comprehensive network statistics.

**Arguments:** None

**Returns:** `NetworkStats!`

**Example:**
```graphql
query {
  network {
    networkStats {
      totalBlocks
      totalCoins
      totalAddresses
      totalValue
      avgBlockTimeSeconds
      avgCoinsPerBlock
      avgValuePerBlock
      uniquePuzzles
      uniqueSolutions
      currentHeight
    }
  }
}
```

### 2. `blockProductionStats`

Analyze block production patterns and timing.

**Arguments:** None

**Returns:** `BlockProductionStats!`

**Example:**
```graphql
query {
  network {
    blockProductionStats {
      avgBlockTimeSeconds
      minBlockTimeSeconds
      maxBlockTimeSeconds
      medianBlockTime
      blockTimeStddev
      blocksToday
      blocksThisWeek
      blocksThisMonth
    }
  }
}
```

### 3. `growthMetrics`

Track network growth over time.

**Arguments:**
- `daysBack` (Int) - Number of days to analyze (default: 30)

**Returns:** `[NetworkGrowthMetrics!]!`

**Example:**
```graphql
query {
  network {
    growthMetrics(daysBack: 90) {
      metricDate
      totalBlocks
      blocksCreated
      uniqueAddresses
      newAddresses
      totalCoins
      newCoins
      totalValue
      valueCreated
    }
  }
}
```

### 4. `unspentCoinAnalysis`

Analyze the unspent coinset (Chia's model, not UTXO).

**Arguments:** None

**Returns:** `UnspentCoinAnalysis!`

**Example:**
```graphql
query {
  network {
    unspentCoinAnalysis {
      totalUnspentCoins
      totalUnspentValue
      avgCoinValue
      medianCoinValue
      dustCoins          # < 1 mojo
      smallCoins         # < 0.01 XCH
      mediumCoins        # 0.01 - 1 XCH
      largeCoins         # > 1 XCH
      avgCoinAgeBlocks
    }
  }
}
```

### 5. `throughputAnalysis`

Analyze network throughput and performance.

**Arguments:**
- `hoursBack` (Int) - Hours to analyze (default: 24)

**Returns:** `ThroughputAnalysis!`

**Example:**
```graphql
query {
  network {
    throughputAnalysis(hoursBack: 48) {
      timePeriod
      tps                    # Transactions per second
      coinsPerSecond
      valuePerSecond
      blocksPerHour
      peakTps
      avgTps
    }
  }
}
```

## Type Definitions

### NetworkStats

Overall network statistics snapshot.

```graphql
type NetworkStats {
  # Total number of blocks
  totalBlocks: Int!
  
  # Total number of coins created
  totalCoins: Int!
  
  # Total unique addresses
  totalAddresses: Int!
  
  # Total value in circulation (mojos)
  totalValue: BigInt!
  
  # Average block time in seconds
  avgBlockTimeSeconds: Float!
  
  # Average coins per block
  avgCoinsPerBlock: Float!
  
  # Average value per block
  avgValuePerBlock: Float!
  
  # Number of unique puzzles
  uniquePuzzles: Int!
  
  # Number of unique solutions
  uniqueSolutions: Int!
  
  # Current blockchain height
  currentHeight: Int!
}
```

### BlockProductionStats

Detailed block production metrics.

```graphql
type BlockProductionStats {
  # Average block time in seconds
  avgBlockTimeSeconds: Float!
  
  # Minimum block time
  minBlockTimeSeconds: Int!
  
  # Maximum block time
  maxBlockTimeSeconds: Int!
  
  # Median block time (PostgreSQL only)
  medianBlockTime: Float
  
  # Block time standard deviation (PostgreSQL only)
  blockTimeStddev: Float
  
  # Blocks produced today
  blocksToday: Int!
  
  # Blocks produced this week
  blocksThisWeek: Int!
  
  # Blocks produced this month
  blocksThisMonth: Int!
}
```

### UnspentCoinAnalysis

Analysis of the unspent coinset (Chia's model).

```graphql
type UnspentCoinAnalysis {
  # Total unspent coins
  totalUnspentCoins: Int!
  
  # Total value in unspent coins
  totalUnspentValue: BigInt!
  
  # Average coin value
  avgCoinValue: Float!
  
  # Median coin value (PostgreSQL only)
  medianCoinValue: Float
  
  # Dust coins (< 1 mojo)
  dustCoins: Int!
  
  # Small coins (< 0.01 XCH)
  smallCoins: Int!
  
  # Medium coins (0.01 - 1 XCH)
  mediumCoins: Int!
  
  # Large coins (> 1 XCH)
  largeCoins: Int!
  
  # Average age of unspent coins in blocks
  avgCoinAgeBlocks: Float!
}
```

## Common Use Cases

### 1. Network Health Check

Monitor overall blockchain health:

```graphql
query NetworkHealth {
  network {
    networkStats {
      currentHeight
      avgBlockTimeSeconds
      totalCoins
      totalAddresses
    }
    blockProductionStats {
      avgBlockTimeSeconds
      blocksToday
      blockTimeStddev
    }
  }
}
```

### 2. Growth Tracking

Track network growth trends:

```graphql
query GrowthAnalysis {
  network {
    growthMetrics(daysBack: 30) {
      metricDate
      totalBlocks
      newAddresses
      newCoins
      valueCreated
    }
  }
}
```

### 3. Performance Monitoring

Monitor network performance:

```graphql
query PerformanceMetrics {
  network {
    throughputAnalysis(hoursBack: 24) {
      timePeriod
      tps
      coinsPerSecond
      valuePerSecond
      peakTps
    }
  }
}
```

### 4. Coinset Analysis

Analyze the unspent coinset distribution:

```graphql
query CoinsetHealth {
  network {
    unspentCoinAnalysis {
      totalUnspentCoins
      totalUnspentValue
      avgCoinValue
      dustCoins
      largeCoins
      avgCoinAgeBlocks
    }
  }
}
```

## Chia-Specific Considerations

1. **Coinset Model**: All coin analysis uses Chia's coinset model, not UTXO
2. **Block Time**: Target block time is 18.75 seconds
3. **Coin Values**: All values are in mojos (1 XCH = 1,000,000,000,000 mojos)
4. **Dust Threshold**: Coins < 1 mojo are considered dust

## Performance Tips

1. **Growth Metrics**: Limit `daysBack` to reasonable ranges (≤ 365)
2. **Caching**: Network stats change slowly, good candidates for caching
3. **Database Differences**: Some statistical functions only available on PostgreSQL

## Database Compatibility

- **PostgreSQL**: Full support for all statistical functions
- **SQLite**: Limited support for median and standard deviation calculations

## Error Handling

Common errors:
- `DATABASE_ERROR`: Database connection issues
- `INVALID_PARAMETER`: Invalid time range parameters

## Next Steps

- Explore [Temporal Queries](temporal.md) for time-based analysis
- Check [Transaction Queries](transactions.md) for transaction metrics
- See [Analytics Queries](analytics.md) for combined insights 