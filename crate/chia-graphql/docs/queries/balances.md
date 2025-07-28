# Balance Queries

The `balances` namespace provides comprehensive balance distribution analysis and wealth analytics for the Chia blockchain. It helps understand economic patterns and wealth distribution across addresses.

## Available Queries

### 1. `distributionStats`

Get statistical overview of balance distribution across all addresses.

**Arguments:** None

**Returns:** `BalanceDistributionStats!`

**Example:**
```graphql
query {
  balances {
    distributionStats {
      totalAddresses
      addressesWithBalance
      avgBalance
      medianBalance
      addressesWith1XchPlus
      addressesWith10XchPlus
      addressesWith100XchPlus
      addressesWith1000XchPlus
      giniCoefficient
      wealthConcentrationTop1Percent
      wealthConcentrationTop10Percent
    }
  }
}
```

### 2. `richestAddresses`

Get the wealthiest addresses on the blockchain.

**Arguments:**
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)

**Returns:** `[RichestAddress!]!`

**Example:**
```graphql
query {
  balances {
    richestAddresses(page: 1, pageSize: 50) {
      puzzleHashHex
      totalXch
      totalMojos
      percentOfSupply
      rankPosition
      coinCount
      firstSeen
      lastActivity
    }
  }
}
```

### 3. `balanceChanges`

Track balance changes for a specific address over time.

**Arguments:**
- `puzzleHashHex` (String!) - The puzzle hash as hex
- `startBlock` (Int!) - Starting block height
- `endBlock` (Int!) - Ending block height

**Returns:** `[BalanceChange!]!`

**Example:**
```graphql
query {
  balances {
    balanceChanges(
      puzzleHashHex: "0xabc..."
      startBlock: 1000000
      endBlock: 2000000
    ) {
      blockHeight
      timestamp
      oldBalance
      newBalance
      netChange
      changeType
      triggeringTransactionId
    }
  }
}
```

### 4. `addressesByBalanceRange`

Get addresses within a specific balance range.

**Arguments:**
- `minBalance` (String!) - Minimum balance in mojos
- `maxBalance` (String!) - Maximum balance in mojos  
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)

**Returns:** `[AddressBalanceRange!]!`

**Example:**
```graphql
query {
  balances {
    addressesByBalanceRange(
      minBalance: "1000000000000"    # 1 XCH
      maxBalance: "10000000000000"   # 10 XCH
      page: 1
    ) {
      puzzleHashHex
      currentBalance
      balanceXch
      coinCount
      category
      lastActivity
    }
  }
}
```

### 5. `coinSizeDistribution`

Analyze the distribution of coin sizes across the network.

**Arguments:** None

**Returns:** `[CoinSizeDistribution!]!`

**Example:**
```graphql
query {
  balances {
    coinSizeDistribution {
      sizeCategory
      minValue
      maxValue
      coinCount
      totalValue
      percentageOfCoins
      percentageOfValue
      avgCoinSize
    }
  }
}
```

### 6. `percentiles`

Get balance percentile statistics.

**Arguments:** None

**Returns:** `BalancePercentiles!`

**Example:**
```graphql
query {
  balances {
    percentiles {
      p50
      p75
      p90
      p95
      p99
      p999
      median
      q1
      q3
      iqr
    }
  }
}
```

## Type Definitions

### BalanceDistributionStats

Overall balance distribution statistics.

```graphql
type BalanceDistributionStats {
  # Total number of addresses
  totalAddresses: Int!
  
  # Addresses with non-zero balance
  addressesWithBalance: Int!
  
  # Average balance across all addresses
  avgBalance: Float!
  
  # Median balance (PostgreSQL only)
  medianBalance: Float
  
  # Addresses with 1+ XCH
  addressesWith1XchPlus: Int!
  
  # Addresses with 10+ XCH
  addressesWith10XchPlus: Int!
  
  # Addresses with 100+ XCH
  addressesWith100XchPlus: Int!
  
  # Addresses with 1000+ XCH
  addressesWith1000XchPlus: Int!
  
  # Gini coefficient (wealth inequality measure)
  giniCoefficient: Float
  
  # Wealth held by top 1% of addresses
  wealthConcentrationTop1Percent: Float!
  
  # Wealth held by top 10% of addresses
  wealthConcentrationTop10Percent: Float!
}
```

### RichestAddress

Information about wealthy addresses.

```graphql
type RichestAddress {
  # Address puzzle hash
  puzzleHashHex: String!
  
  # Balance in XCH
  totalXch: Float!
  
  # Balance in mojos
  totalMojos: BigInt!
  
  # Percentage of total supply
  percentOfSupply: Float!
  
  # Rank position (1 = richest)
  rankPosition: Int!
  
  # Number of coins held
  coinCount: Int!
  
  # First seen block
  firstSeen: Int
  
  # Last activity block
  lastActivity: Int
}
```

### BalanceChange

Track balance evolution over time.

```graphql
type BalanceChange {
  # Block height when change occurred
  blockHeight: Int!
  
  # Block timestamp
  timestamp: String!
  
  # Balance before change
  oldBalance: BigInt!
  
  # Balance after change
  newBalance: BigInt!
  
  # Net change (positive or negative)
  netChange: BigInt!
  
  # Type of change (RECEIVE, SPEND, etc.)
  changeType: String!
  
  # Transaction that caused the change
  triggeringTransactionId: String
}
```

### CoinSizeDistribution

Distribution of coin sizes.

```graphql
type CoinSizeDistribution {
  # Category name (e.g., "Dust", "Small", "Medium", "Large")
  sizeCategory: String!
  
  # Minimum value in category
  minValue: BigInt!
  
  # Maximum value in category
  maxValue: BigInt!
  
  # Number of coins in category
  coinCount: Int!
  
  # Total value in category
  totalValue: BigInt!
  
  # Percentage of total coins
  percentageOfCoins: Float!
  
  # Percentage of total value
  percentageOfValue: Float!
  
  # Average coin size in category
  avgCoinSize: Float!
}
```

## Common Use Cases

### 1. Wealth Distribution Analysis

Understand economic inequality:

```graphql
query WealthAnalysis {
  balances {
    distributionStats {
      giniCoefficient
      wealthConcentrationTop1Percent
      wealthConcentrationTop10Percent
      addressesWith1000XchPlus
    }
    
    percentiles {
      median
      p95
      p99
    }
  }
}
```

### 2. Whale Watching

Monitor large holders:

```graphql
query WhaleWatch {
  balances {
    richestAddresses(page: 1, pageSize: 100) {
      puzzleHashHex
      totalXch
      percentOfSupply
      lastActivity
    }
    
    addressesByBalanceRange(
      minBalance: "100000000000000"  # 100 XCH+
      maxBalance: "999999999999999999"
    ) {
      puzzleHashHex
      currentBalance
      lastActivity
    }
  }
}
```

### 3. Balance Evolution Tracking

Track how balances change over time:

```graphql
query BalanceEvolution($address: String!, $startBlock: Int!) {
  balances {
    balanceChanges(
      puzzleHashHex: $address
      startBlock: $startBlock
      endBlock: 999999999
    ) {
      blockHeight
      timestamp
      newBalance
      netChange
      changeType
    }
  }
}
```

### 4. Economic Health Monitoring

Monitor network economic health:

```graphql
query EconomicHealth {
  balances {
    distributionStats {
      totalAddresses
      addressesWithBalance
      avgBalance
      addressesWith1XchPlus
    }
    
    coinSizeDistribution {
      sizeCategory
      coinCount
      percentageOfValue
    }
  }
}
```

## Balance Categories

The system categorizes balances into several tiers:

- **Dust**: < 0.001 XCH (1,000,000,000 mojos)
- **Small**: 0.001 - 0.1 XCH
- **Medium**: 0.1 - 10 XCH  
- **Large**: 10 - 1,000 XCH
- **Whale**: > 1,000 XCH

## Performance Considerations

1. **Large Date Ranges**: Limit `balanceChanges` queries to reasonable block ranges
2. **Pagination**: Always use pagination for large result sets
3. **Caching**: Distribution stats change slowly, good candidates for caching
4. **Database Type**: Some statistical functions work better on PostgreSQL

## Economic Insights

### Gini Coefficient
- **0.0**: Perfect equality (everyone has same balance)
- **1.0**: Perfect inequality (one address has everything)
- **Typical range**: 0.6-0.9 for cryptocurrencies

### Wealth Concentration
- **Top 1%**: Typically holds 20-40% of total supply
- **Top 10%**: Typically holds 60-80% of total supply

## Error Handling

Common errors:
- `INVALID_INPUT`: Invalid hex string or balance range
- `INVALID_RANGE`: Start block > end block
- `LIMIT_EXCEEDED`: Query parameters exceed limits

## Next Steps

- Explore [Address Queries](addresses.md) for address-specific analytics
- Check [Network Queries](network.md) for network-wide statistics  
- See [Examples](../examples/index.md) for more complex use cases 