# Transaction Queries

The `transactions` namespace provides comprehensive transaction analysis, spending patterns, fee analytics, and complex transaction pattern detection for the Chia blockchain.

## Available Queries

### 1. `spendVelocityAnalysis`

Analyze how quickly coins are spent after being created.

**Arguments:** None

**Returns:** `SpendVelocityAnalysis!`

**Example:**
```graphql
query {
  transactions {
    spendVelocityAnalysis {
      avgBlocksToSpend
      medianBlocksToSpend
      immediateSpends
      quickSpends
      mediumSpends
      longTermHolds
      totalCoinsAnalyzed
      spendVelocityScore
      hodlRatio
    }
  }
}
```

### 2. `complexTransactionPatterns`

Identify complex transaction patterns involving multiple inputs/outputs.

**Arguments:**
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)
- `minComplexity` (Int) - Minimum complexity score (default: 5)

**Returns:** `[ComplexTransactionPattern!]!`

**Example:**
```graphql
query {
  transactions {
    complexTransactionPatterns(
      page: 1
      pageSize: 50
      minComplexity: 10
    ) {
      spentBlock
      blockTimestamp
      coinsSpent
      coinsCreated
      totalValueIn
      totalValueOut
      feeAmount
      uniquePuzzlesUsed
      complexityScore
      patternType
      suspiciousActivity
    }
  }
}
```

### 3. `spendingFrequencyByAddress`

Analyze spending frequency patterns by address.

**Arguments:**
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)
- `minTransactions` (Int) - Minimum transaction count (default: 10)

**Returns:** `[SpendingFrequency!]!`

**Example:**
```graphql
query {
  transactions {
    spendingFrequencyByAddress(
      page: 1
      pageSize: 100
      minTransactions: 50
    ) {
      puzzleHashHex
      totalCoinsSpent
      totalCoinsReceived
      avgSpendsPerBlock
      avgReceivesPerBlock
      spendingPattern
      activityLevel
      lastActivityBlock
      riskScore
    }
  }
}
```

### 4. `feeAnalysis`

Analyze transaction fee patterns and trends.

**Arguments:**
- `hoursBack` (Int) - Hours to analyze (default: 24)

**Returns:** `TransactionFeeAnalysis!`

**Example:**
```graphql
query {
  transactions {
    feeAnalysis(hoursBack: 168) {  # 1 week
      timePeriod
      totalTransactions
      totalFees
      avgFeePerTransaction
      medianFee
      minFee
      maxFee
      zeroFeeTransactions
      highFeeTransactions
      feeEfficiency
      networkCongestion
    }
  }
}
```

### 5. `patternsByType`

Categorize and analyze different types of transaction patterns.

**Arguments:** None

**Returns:** `[TransactionPatternByType!]!`

**Example:**
```graphql
query {
  transactions {
    patternsByType {
      transactionType
      count
      percentage
      avgValueIn
      avgValueOut
      avgFee
      avgComplexity
      typicalUseCase
      riskLevel
    }
  }
}
```

## Type Definitions

### SpendVelocityAnalysis

Analysis of how quickly coins are spent.

```graphql
type SpendVelocityAnalysis {
  # Average blocks between creation and spending
  avgBlocksToSpend: Float!
  
  # Median blocks between creation and spending
  medianBlocksToSpend: Float
  
  # Coins spent within 1 block
  immediateSpends: Int!
  
  # Coins spent within 10 blocks
  quickSpends: Int!
  
  # Coins spent within 100 blocks
  mediumSpends: Int!
  
  # Coins held for 1000+ blocks
  longTermHolds: Int!
  
  # Total coins analyzed
  totalCoinsAnalyzed: Int!
  
  # Velocity score (0-100, higher = faster spending)
  spendVelocityScore: Float!
  
  # Percentage of coins held long-term
  hodlRatio: Float!
}
```

### ComplexTransactionPattern

Complex transaction involving multiple inputs/outputs.

```graphql
type ComplexTransactionPattern {
  # Block where transaction occurred
  spentBlock: Int!
  
  # Block timestamp
  blockTimestamp: String!
  
  # Number of coins spent
  coinsSpent: Int!
  
  # Number of coins created
  coinsCreated: Int!
  
  # Total value of inputs
  totalValueIn: BigInt!
  
  # Total value of outputs
  totalValueOut: BigInt!
  
  # Fee amount paid
  feeAmount: BigInt!
  
  # Number of unique puzzle hashes involved
  uniquePuzzlesUsed: Int!
  
  # Complexity score (higher = more complex)
  complexityScore: Float!
  
  # Pattern classification
  patternType: String!
  
  # Whether pattern appears suspicious
  suspiciousActivity: Boolean!
}
```

### SpendingFrequency

Address spending frequency analysis.

```graphql
type SpendingFrequency {
  # Address puzzle hash
  puzzleHashHex: String!
  
  # Total coins spent by address
  totalCoinsSpent: Int!
  
  # Total coins received by address
  totalCoinsReceived: Int!
  
  # Average spends per block
  avgSpendsPerBlock: Float!
  
  # Average receives per block
  avgReceivesPerBlock: Float!
  
  # Spending pattern classification
  spendingPattern: String!
  
  # Activity level (LOW, MEDIUM, HIGH)
  activityLevel: String!
  
  # Most recent activity block
  lastActivityBlock: Int!
  
  # Risk score based on patterns
  riskScore: Float!
}
```

### TransactionFeeAnalysis

Fee market analysis.

```graphql
type TransactionFeeAnalysis {
  # Time period analyzed
  timePeriod: String!
  
  # Total number of transactions
  totalTransactions: Int!
  
  # Total fees paid
  totalFees: BigInt!
  
  # Average fee per transaction
  avgFeePerTransaction: Float!
  
  # Median fee paid
  medianFee: BigInt
  
  # Minimum fee observed
  minFee: BigInt!
  
  # Maximum fee observed
  maxFee: BigInt!
  
  # Number of zero-fee transactions
  zeroFeeTransactions: Int!
  
  # Number of high-fee transactions
  highFeeTransactions: Int!
  
  # Fee efficiency score
  feeEfficiency: Float!
  
  # Network congestion indicator
  networkCongestion: Float!
}
```

### TransactionPatternByType

Transaction patterns categorized by type.

```graphql
type TransactionPatternByType {
  # Type of transaction pattern
  transactionType: String!
  
  # Number of transactions of this type
  count: Int!
  
  # Percentage of total transactions
  percentage: Float!
  
  # Average input value
  avgValueIn: Float!
  
  # Average output value
  avgValueOut: Float!
  
  # Average fee for this type
  avgFee: Float!
  
  # Average complexity score
  avgComplexity: Float!
  
  # Typical use case description
  typicalUseCase: String!
  
  # Risk level (LOW, MEDIUM, HIGH)
  riskLevel: String!
}
```

## Common Use Cases

### 1. Network Health Monitoring

Monitor transaction processing efficiency:

```graphql
query NetworkHealth {
  transactions {
    spendVelocityAnalysis {
      avgBlocksToSpend
      spendVelocityScore
      hodlRatio
    }
    
    feeAnalysis(hoursBack: 24) {
      avgFeePerTransaction
      networkCongestion
      feeEfficiency
    }
  }
}
```

### 2. Suspicious Activity Detection

Identify potentially suspicious transaction patterns:

```graphql
query SuspiciousActivity {
  transactions {
    complexTransactionPatterns(minComplexity: 15) {
      spentBlock
      complexityScore
      patternType
      suspiciousActivity
      uniquePuzzlesUsed
    }
    
    spendingFrequencyByAddress(minTransactions: 100) {
      puzzleHashHex
      spendingPattern
      riskScore
      activityLevel
    }
  }
}
```

### 3. Fee Market Analysis

Understand fee dynamics:

```graphql
query FeeMarketAnalysis {
  transactions {
    feeAnalysis(hoursBack: 168) {
      totalTransactions
      avgFeePerTransaction
      medianFee
      networkCongestion
      zeroFeeTransactions
    }
    
    patternsByType {
      transactionType
      avgFee
      count
      percentage
    }
  }
}
```

### 4. User Behavior Analysis

Analyze user transaction patterns:

```graphql
query UserBehavior {
  transactions {
    spendingFrequencyByAddress(page: 1, pageSize: 50) {
      puzzleHashHex
      spendingPattern
      activityLevel
      avgSpendsPerBlock
    }
    
    spendVelocityAnalysis {
      immediateSpends
      longTermHolds
      avgBlocksToSpend
    }
  }
}
```

## Transaction Pattern Types

### Spending Patterns
- **IMMEDIATE**: Spends coins within 1-2 blocks
- **QUICK**: Spends coins within 10 blocks
- **NORMAL**: Spends coins within 100 blocks
- **PATIENT**: Holds coins for 100+ blocks
- **HODLER**: Holds coins for 1000+ blocks

### Pattern Classifications
- **SIMPLE**: Basic 1-to-1 or 1-to-2 transactions
- **BATCH**: Multiple inputs to multiple outputs
- **MIXING**: Complex multi-hop transactions
- **CONSOLIDATION**: Many inputs to few outputs
- **DISTRIBUTION**: Few inputs to many outputs

### Activity Levels
- **LOW**: < 1 transaction per 1000 blocks
- **MEDIUM**: 1-10 transactions per 1000 blocks
- **HIGH**: > 10 transactions per 1000 blocks

## Risk Assessment

### Risk Factors
- **High Frequency**: Unusually high transaction rates
- **Complex Patterns**: Unusual input/output combinations
- **Rapid Spending**: Immediate re-spending of received coins
- **Large Values**: Transactions significantly above average
- **Pattern Anomalies**: Deviations from normal behavior

### Risk Scores
- **0-25**: Low risk, normal patterns
- **26-50**: Moderate risk, some unusual activity
- **51-75**: High risk, suspicious patterns
- **76-100**: Critical risk, likely malicious

## Performance Metrics

### Velocity Metrics
- **Immediate Spends**: < 1 block (very fast)
- **Quick Spends**: 1-10 blocks (fast)
- **Medium Spends**: 11-100 blocks (normal)
- **Long Holds**: 100+ blocks (patient)

### Complexity Scoring
- **1-3**: Simple transactions
- **4-6**: Moderate complexity
- **7-10**: Complex transactions
- **11+**: Highly complex, potentially suspicious

## Fee Analysis

### Fee Categories
- **Zero Fee**: Exactly 0 mojos
- **Low Fee**: 1-1,000 mojos
- **Normal Fee**: 1,001-10,000 mojos
- **High Fee**: 10,001+ mojos

### Congestion Indicators
- **Low**: < 1% high-fee transactions
- **Moderate**: 1-5% high-fee transactions
- **High**: 5-15% high-fee transactions
- **Critical**: > 15% high-fee transactions

## Error Handling

Common errors:
- `INSUFFICIENT_DATA`: Not enough transaction history
- `INVALID_TIMEFRAME`: Invalid time range specified
- `COMPLEXITY_LIMIT`: Complexity threshold too high
- `RATE_LIMITED`: Too many requests

## Performance Tips

1. **Pagination**: Always use pagination for large result sets
2. **Time Limits**: Limit analysis periods to reasonable ranges
3. **Complexity Filters**: Use minimum complexity filters appropriately
4. **Caching**: Complex pattern analysis results are cached

## Next Steps

- Explore [Address Queries](addresses.md) for address-specific transaction analysis
- Check [Temporal Queries](temporal.md) for time-based transaction trends
- See [Examples](../examples/index.md) for advanced transaction analysis 