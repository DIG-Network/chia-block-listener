# Temporal Queries

The `temporal` namespace provides time-based analytics and trend analysis for the Chia blockchain. It helps understand how blockchain activity, growth, and patterns evolve over time.

## Available Queries

### 1. `growthTrends`

Analyze blockchain growth trends over time.

**Arguments:**
- `daysBack` (Int) - Number of days to analyze (default: 30)
- `periodType` (String) - Aggregation period: "hour", "day", "week", "month" (default: "day")

**Returns:** `[GrowthTrend!]!`

**Example:**
```graphql
query {
  temporal {
    growthTrends(daysBack: 90, periodType: "day") {
      period
      periodStart
      periodEnd
      blockCount
      coinCount
      transactionCount
      uniqueAddresses
      totalValue
      avgBlockTime
      growthRate
      cumulativeBlocks
      cumulativeCoins
    }
  }
}
```

### 2. `activityHeatmap`

Generate activity heatmap showing patterns by time of day and day of week.

**Arguments:**
- `daysBack` (Int) - Number of days to analyze (default: 30)

**Returns:** `[ActivityHeatmap!]!`

**Example:**
```graphql
query {
  temporal {
    activityHeatmap(daysBack: 60) {
      hourOfDay
      dayOfWeek
      blockCount
      transactionCount
      totalValue
      avgValuePerTransaction
      uniqueAddresses
      peakActivityScore
      relativeActivity
    }
  }
}
```

### 3. `coinAggregation`

Aggregate coin creation and spending patterns over time.

**Arguments:**
- `daysBack` (Int) - Number of days to analyze (default: 30)
- `aggregationPeriod` (String) - Period: "hour", "day", "week" (default: "day")

**Returns:** `[CoinAggregation!]!`

**Example:**
```graphql
query {
  temporal {
    coinAggregation(daysBack: 30, aggregationPeriod: "day") {
      period
      timestamp
      coinsCreated
      coinsSpent
      netCoinChange
      valueCreated
      valueSpent
      netValueChange
      avgCoinSize
      largestCoin
      smallestCoin
      coinVelocity
    }
  }
}
```

### 4. `seasonalPatterns`

Identify seasonal patterns and cyclical trends.

**Arguments:**
- `monthsBack` (Int) - Number of months to analyze (default: 12)

**Returns:** `[SeasonalPattern!]!`

**Example:**
```graphql
query {
  temporal {
    seasonalPatterns(monthsBack: 24) {
      period
      periodType
      avgDailyTransactions
      avgDailyVolume
      avgActiveAddresses
      seasonalIndex
      trend
      volatility
      peakDay
      lowDay
      cycleStrength
    }
  }
}
```

## Type Definitions

### GrowthTrend

Represents blockchain growth over time periods.

```graphql
type GrowthTrend {
  # Time period identifier
  period: String!
  
  # Period start timestamp
  periodStart: String!
  
  # Period end timestamp
  periodEnd: String!
  
  # Number of blocks in period
  blockCount: Int!
  
  # Number of coins created in period
  coinCount: Int!
  
  # Number of transactions in period
  transactionCount: Int!
  
  # Unique addresses active in period
  uniqueAddresses: Int!
  
  # Total value moved in period
  totalValue: BigInt!
  
  # Average block time in period
  avgBlockTime: Float!
  
  # Growth rate compared to previous period
  growthRate: Float!
  
  # Cumulative blocks up to this period
  cumulativeBlocks: Int!
  
  # Cumulative coins up to this period
  cumulativeCoins: Int!
}
```

### ActivityHeatmap

Activity patterns by time of day and day of week.

```graphql
type ActivityHeatmap {
  # Hour of day (0-23)
  hourOfDay: Int!
  
  # Day of week (0-6, 0=Sunday)
  dayOfWeek: Int!
  
  # Number of blocks produced
  blockCount: Int!
  
  # Number of transactions
  transactionCount: Int!
  
  # Total value transferred
  totalValue: BigInt!
  
  # Average value per transaction
  avgValuePerTransaction: Float!
  
  # Unique addresses active
  uniqueAddresses: Int!
  
  # Peak activity score (0-100)
  peakActivityScore: Float!
  
  # Relative activity compared to average
  relativeActivity: Float!
}
```

### CoinAggregation

Coin creation and spending aggregated by time.

```graphql
type CoinAggregation {
  # Time period
  period: String!
  
  # Period timestamp
  timestamp: String!
  
  # Coins created in period
  coinsCreated: Int!
  
  # Coins spent in period
  coinsSpent: Int!
  
  # Net change in coin count
  netCoinChange: Int!
  
  # Value created in period
  valueCreated: BigInt!
  
  # Value spent in period
  valueSpent: BigInt!
  
  # Net change in value
  netValueChange: BigInt!
  
  # Average coin size
  avgCoinSize: Float!
  
  # Largest coin created
  largestCoin: BigInt!
  
  # Smallest coin created
  smallestCoin: BigInt!
  
  # Coin velocity (spent/created ratio)
  coinVelocity: Float!
}
```

### SeasonalPattern

Seasonal trends and cyclical patterns.

```graphql
type SeasonalPattern {
  # Time period (month, quarter, etc.)
  period: String!
  
  # Type of period aggregation
  periodType: String!
  
  # Average daily transactions
  avgDailyTransactions: Float!
  
  # Average daily volume
  avgDailyVolume: Float!
  
  # Average daily active addresses
  avgActiveAddresses: Float!
  
  # Seasonal index (1.0 = average)
  seasonalIndex: Float!
  
  # Trend direction and strength
  trend: Float!
  
  # Volatility measure
  volatility: Float!
  
  # Peak activity day in period
  peakDay: String!
  
  # Lowest activity day in period
  lowDay: String!
  
  # Strength of cyclical pattern
  cycleStrength: Float!
}
```

## Common Use Cases

### 1. Growth Analysis

Track blockchain adoption and growth:

```graphql
query GrowthAnalysis {
  temporal {
    growthTrends(daysBack: 365, periodType: "week") {
      period
      blockCount
      uniqueAddresses
      totalValue
      growthRate
      cumulativeBlocks
    }
  }
}
```

### 2. Activity Pattern Discovery

Find peak usage times:

```graphql
query ActivityPatterns {
  temporal {
    activityHeatmap(daysBack: 90) {
      hourOfDay
      dayOfWeek
      transactionCount
      peakActivityScore
      relativeActivity
    }
  }
}
```

### 3. Economic Cycle Analysis

Understand economic patterns:

```graphql
query EconomicCycles {
  temporal {
    coinAggregation(daysBack: 180, aggregationPeriod: "week") {
      period
      coinsCreated
      coinsSpent
      coinVelocity
      avgCoinSize
      netValueChange
    }
  }
}
```

### 4. Long-term Trend Analysis

Identify seasonal patterns:

```graphql
query SeasonalTrends {
  temporal {
    seasonalPatterns(monthsBack: 24) {
      period
      avgDailyTransactions
      seasonalIndex
      trend
      volatility
      cycleStrength
    }
  }
}
```

## Time Period Types

### Aggregation Periods
- **hour**: Hourly aggregation (max 30 days)
- **day**: Daily aggregation (max 2 years)
- **week**: Weekly aggregation (max 5 years)
- **month**: Monthly aggregation (unlimited)

### Seasonal Analysis
- **Monthly**: 12 data points per year
- **Quarterly**: 4 data points per year
- **Weekly**: 52 data points per year
- **Daily**: 365 data points per year

## Performance Insights

### Peak Activity Times
Based on historical data, typical patterns show:
- **Peak Hours**: 14:00-16:00 UTC (highest activity)
- **Peak Days**: Tuesday-Thursday (business days)
- **Seasonal**: Q4 often shows increased activity

### Growth Indicators
- **Healthy Growth**: 5-15% monthly increase in unique addresses
- **Block Time Stability**: Consistent ~18.75 second average
- **Transaction Growth**: Should correlate with address growth

## Analytics Features

### Trend Detection
- **Linear Trends**: Simple growth/decline patterns
- **Exponential Trends**: Accelerating growth patterns
- **Cyclical Trends**: Repeating seasonal patterns
- **Volatility Measures**: Stability vs. fluctuation analysis

### Comparative Analysis
- **Period-over-Period**: Compare current vs. previous periods
- **Year-over-Year**: Compare same periods across years
- **Moving Averages**: Smooth out short-term fluctuations
- **Growth Rates**: Calculate percentage changes

## Data Quality

### Accuracy Notes
- **Block Times**: Precise to the second
- **Coin Counts**: Exact counts from blockchain
- **Value Calculations**: Precise to the mojo
- **Address Counts**: Unique puzzle hashes only

### Data Limitations
- **Historical Depth**: Limited by blockchain start date
- **Real-time Lag**: Some metrics may lag by 1-2 blocks
- **Timezone**: All times in UTC
- **Aggregation**: Some precision lost in aggregation

## Performance Considerations

1. **Time Range Limits**: Larger ranges require more processing
2. **Aggregation Level**: Higher detail = more compute time
3. **Caching**: Results cached for common time ranges
4. **Database Load**: Complex temporal queries are resource-intensive

## Error Handling

Common errors:
- `INVALID_PERIOD`: Invalid aggregation period specified
- `RANGE_TOO_LARGE`: Time range exceeds limits
- `INSUFFICIENT_DATA`: Not enough historical data
- `FUTURE_DATE`: Cannot analyze future periods

## Next Steps

- Explore [Network Queries](network.md) for current network status
- Check [Transaction Queries](transactions.md) for transaction patterns
- See [Examples](../examples/index.md) for temporal analysis patterns 