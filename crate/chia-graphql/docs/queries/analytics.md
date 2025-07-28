# Analytics Queries

The `analytics` namespace provides a unified interface to all specialized analytics resolvers, offering high-level insights and cross-domain analysis of the Chia blockchain.

## Available Queries

### 1. `overview`

Get a comprehensive blockchain overview with key metrics.

**Arguments:** None

**Returns:** `BlockchainOverview!`

**Example:**
```graphql
query {
  analytics {
    overview {
      totalBlocks
      totalCoins
      totalAddresses
      currentHeight
      dailyBlocks
      dailyCoins
      avgBlocksToSpend
      networkActivityScore
    }
  }
}
```

### 2. `healthMetrics`

Get blockchain health metrics and scoring.

**Arguments:** None

**Returns:** `HealthMetrics!`

**Example:**
```graphql
query {
  analytics {
    healthMetrics {
      overallHealthScore
      blockTimeHealth
      throughputHealth
      feeHealth
      coinsetHealth
      tps24h
      avgFee24h
    }
  }
}
```

### 3. `network`

Access network-specific analytics (delegates to NetworkQueries).

**Arguments:** None

**Returns:** `NetworkQueries!`

**Example:**
```graphql
query {
  analytics {
    network {
      networkStats {
        totalBlocks
        avgBlockTimeSeconds
        currentHeight
      }
      
      unspentCoinAnalysis {
        totalUnspentCoins
        avgCoinValue
        dustCoins
      }
    }
  }
}
```

### 4. `temporal`

Access time-based analytics (delegates to TemporalQueries).

**Arguments:** None

**Returns:** `TemporalQueries!`

**Example:**
```graphql
query {
  analytics {
    temporal {
      growthTrends(daysBack: 30) {
        period
        blockCount
        coinCount
        growthRate
      }
      
      activityHeatmap(daysBack: 7) {
        hourOfDay
        dayOfWeek
        transactionCount
        peakActivityScore
      }
    }
  }
}
```

### 5. `transactions`

Access transaction analytics (delegates to TransactionQueries).

**Arguments:** None

**Returns:** `TransactionQueries!`

**Example:**
```graphql
query {
  analytics {
    transactions {
      spendVelocityAnalysis {
        avgBlocksToSpend
        immediateSpends
        hodlRatio
      }
      
      feeAnalysis(hoursBack: 24) {
        avgFeePerTransaction
        networkCongestion
      }
    }
  }
}
```

### 6. `addresses`

Access address analytics (delegates to AddressQueries).

**Arguments:** None

**Returns:** `AddressQueries!`

**Example:**
```graphql
query {
  analytics {
    addresses {
      addressBehaviorPatterns {
        totalAddresses
        whaleAddresses
        hodlerAddresses
      }
      
      mostActiveAddresses(page: 1, pageSize: 10) {
        puzzleHashHex
        totalTransactions
        currentBalance
      }
    }
  }
}
```

### 7. `balances`

Access balance analytics (delegates to BalanceQueries).

**Arguments:** None

**Returns:** `BalanceQueries!`

**Example:**
```graphql
query {
  analytics {
    balances {
      distributionStats {
        giniCoefficient
        wealthConcentrationTop1Percent
        addressesWith1000XchPlus
      }
      
      percentiles {
        median
        p95
        p99
      }
    }
  }
}
```

### 8. `cats`

Access CAT token analytics (delegates to CatQueries).

**Arguments:** None

**Returns:** `CatQueries!`

**Example:**
```graphql
query {
  analytics {
    cats {
      assetSummary(sortBy: "HOLDERS") {
        symbol
        uniqueHolders
        totalSupply
        isActive
      }
    }
  }
}
```

### 9. `nfts`

Access NFT analytics (delegates to NftQueries).

**Arguments:** None

**Returns:** `NftQueries!`

**Example:**
```graphql
query {
  analytics {
    nfts {
      collectionAnalytics(collectionId: "col1...") {
        totalNfts
        uniqueOwners
        floorPrice
        liquidityScore
      }
    }
  }
}
```

## Type Definitions

### BlockchainOverview

Comprehensive blockchain metrics overview.

```graphql
type BlockchainOverview {
  # Total number of blocks
  totalBlocks: Int!
  
  # Total number of coins
  totalCoins: Int!
  
  # Total unique addresses
  totalAddresses: Int!
  
  # Current blockchain height
  currentHeight: Int!
  
  # Blocks created in last 24 hours
  dailyBlocks: Int!
  
  # Coins created in last 24 hours
  dailyCoins: Int!
  
  # Average blocks until coin is spent
  avgBlocksToSpend: Float!
  
  # Network activity score (0-100)
  networkActivityScore: Float!
}
```

### HealthMetrics

Blockchain health scoring and metrics.

```graphql
type HealthMetrics {
  # Overall health score (0-100)
  overallHealthScore: Float!
  
  # Block production health (0-100)
  blockTimeHealth: Float!
  
  # Network throughput health (0-100)
  throughputHealth: Float!
  
  # Fee market health (0-100)
  feeHealth: Float!
  
  # Coinset health (0-100)
  coinsetHealth: Float!
  
  # Transactions per second (24h average)
  tps24h: Float!
  
  # Average fee (24h)
  avgFee24h: Float!
}
```

## Common Use Cases

### 1. Complete Blockchain Dashboard

Get all key metrics for a comprehensive dashboard:

```graphql
query BlockchainDashboard {
  analytics {
    overview {
      totalBlocks
      totalCoins
      totalAddresses
      currentHeight
      networkActivityScore
    }
    
    healthMetrics {
      overallHealthScore
      blockTimeHealth
      throughputHealth
      feeHealth
    }
    
    network {
      networkStats {
        avgBlockTimeSeconds
        totalValue
      }
      
      throughputAnalysis(hoursBack: 24) {
        tps
        peakTps
      }
    }
  }
}
```

### 2. Multi-Domain Analysis

Analyze multiple aspects of the blockchain simultaneously:

```graphql
query MultiDomainAnalysis {
  analytics {
    addresses {
      addressBehaviorPatterns {
        whaleAddresses
        hodlerAddresses
        traderAddresses
      }
    }
    
    balances {
      distributionStats {
        giniCoefficient
        wealthConcentrationTop1Percent
      }
    }
    
    transactions {
      spendVelocityAnalysis {
        avgBlocksToSpend
        hodlRatio
      }
    }
    
    temporal {
      growthTrends(daysBack: 30) {
        period
        growthRate
        uniqueAddresses
      }
    }
  }
}
```

### 3. Economic Health Assessment

Comprehensive economic health analysis:

```graphql
query EconomicHealth {
  analytics {
    healthMetrics {
      overallHealthScore
      feeHealth
      coinsetHealth
      tps24h
    }
    
    balances {
      distributionStats {
        giniCoefficient
        avgBalance
        addressesWith1XchPlus
      }
      
      percentiles {
        median
        p95
        p99
      }
    }
    
    network {
      unspentCoinAnalysis {
        totalUnspentCoins
        avgCoinValue
        avgCoinAgeBlocks
      }
    }
  }
}
```

### 4. Token Ecosystem Overview

Analyze the token ecosystem:

```graphql
query TokenEcosystem {
  analytics {
    cats {
      assetSummary(pageSize: 10, sortBy: "HOLDERS") {
        symbol
        name
        uniqueHolders
        totalSupply
        isActive
      }
    }
    
    nfts {
      # Note: Would need specific collection IDs
      # This is a conceptual example
    }
  }
}
```

### 5. Performance Monitoring

Monitor blockchain performance across domains:

```graphql
query PerformanceMonitoring {
  analytics {
    healthMetrics {
      overallHealthScore
      blockTimeHealth
      throughputHealth
    }
    
    network {
      blockProductionStats {
        avgBlockTimeSeconds
        blocksToday
        blockTimeStddev
      }
    }
    
    transactions {
      feeAnalysis(hoursBack: 24) {
        networkCongestion
        feeEfficiency
        avgFeePerTransaction
      }
    }
  }
}
```

## Analytics Architecture

The Analytics namespace serves as a unified entry point to all specialized resolvers:

```
analytics
├── overview() → BlockchainOverview
├── healthMetrics() → HealthMetrics  
├── network() → NetworkQueries
├── temporal() → TemporalQueries
├── transactions() → TransactionQueries
├── addresses() → AddressQueries
├── balances() → BalanceQueries
├── cats() → CatQueries
└── nfts() → NftQueries
```

## Health Scoring System

### Overall Health Score Calculation
The overall health score is calculated as a weighted average:
- **Block Time Health**: 25% weight
- **Throughput Health**: 25% weight  
- **Fee Health**: 25% weight
- **Coinset Health**: 25% weight

### Individual Health Components

#### Block Time Health (0-100)
- **100**: 18.75s ± 2s (ideal)
- **80**: 18.75s ± 5s (good)
- **60**: 18.75s ± 10s (acceptable)
- **< 60**: Outside acceptable range

#### Throughput Health (0-100)
- **100**: > 10 TPS
- **80**: 5-10 TPS
- **60**: 1-5 TPS
- **< 60**: < 1 TPS

#### Fee Health (0-100)
- **100**: < 1% high-fee transactions
- **80**: 1-5% high-fee transactions
- **60**: 5-15% high-fee transactions
- **< 60**: > 15% high-fee transactions

#### Coinset Health (0-100)
- **100**: > 1M unspent coins, low dust ratio
- **80**: 500K-1M unspent coins
- **60**: 100K-500K unspent coins
- **< 60**: < 100K unspent coins

## Cross-Domain Insights

### Network Activity Score
Combines multiple metrics to assess overall network activity:
- Daily transaction volume
- Number of active addresses
- Block production consistency
- Coin velocity

### Economic Indicators
- **Wealth Distribution**: Gini coefficient and concentration metrics
- **Liquidity**: Coin velocity and spending patterns
- **Growth**: Address and transaction growth rates
- **Stability**: Price stability and fee predictability

## Performance Considerations

1. **Comprehensive Queries**: Analytics queries can be resource-intensive
2. **Caching Strategy**: Results cached at different intervals:
   - Overview: 5 minutes
   - Health Metrics: 1 minute
   - Delegated queries: Per individual resolver
3. **Pagination**: Use pagination for large datasets
4. **Time Ranges**: Limit time-based queries to reasonable ranges

## Error Handling

Common errors:
- `INSUFFICIENT_DATA`: Not enough data for analysis
- `CALCULATION_ERROR`: Metric calculation failed
- `TIMEOUT`: Query exceeded time limits
- `DELEGATE_ERROR`: Error from delegated resolver

## Best Practices

1. **Use Analytics for Overviews**: Start with analytics for high-level insights
2. **Drill Down with Specialized Resolvers**: Use specific resolvers for detailed analysis
3. **Combine Multiple Domains**: Analytics excels at cross-domain queries
4. **Monitor Health Metrics**: Regular health monitoring for operational insights
5. **Cache Results**: Cache analytics results for dashboard applications

## Integration Patterns

### Dashboard Integration
```javascript
// React example
const useBlockchainDashboard = () => {
  const { data } = useQuery(BLOCKCHAIN_DASHBOARD_QUERY);
  
  return {
    overview: data?.analytics?.overview,
    health: data?.analytics?.healthMetrics,
    isHealthy: data?.analytics?.healthMetrics?.overallHealthScore > 80
  };
};
```

### Monitoring Integration
```javascript
// Monitoring alert example
const checkBlockchainHealth = async () => {
  const { healthMetrics } = await client.query({
    query: HEALTH_METRICS_QUERY
  });
  
  if (healthMetrics.overallHealthScore < 70) {
    sendAlert('Blockchain health degraded', healthMetrics);
  }
};
```

## Next Steps

- Explore individual resolver documentation for detailed queries
- Check [Examples](../examples/index.md) for comprehensive analytics patterns
- See [Performance Guide](../performance.md) for optimization tips 