# Address Queries

The `addresses` namespace provides comprehensive address analytics, activity tracking, and behavioral analysis. It offers deep insights into how addresses interact with the Chia blockchain.

## Available Queries

### 1. `mostActiveAddresses`

Get the most active addresses by transaction count.

**Arguments:**
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)

**Returns:** `[ActiveAddress!]!`

**Example:**
```graphql
query {
  addresses {
    mostActiveAddresses(page: 1, pageSize: 10) {
      puzzleHashHex
      totalTransactions
      coinsReceived
      coinsSpent
      currentBalance
      firstActivityBlock
      lastActivityBlock
    }
  }
}
```

### 2. `addressesByBalance`

Get addresses sorted by their current balance.

**Arguments:**
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)

**Returns:** `[AddressByBalance!]!`

**Example:**
```graphql
query {
  addresses {
    addressesByBalance(page: 1) {
      puzzleHashHex
      currentBalance
      coinCount
      firstSeen
      lastSeen
    }
  }
}
```

### 3. `addressActivity`

Get detailed activity statistics for a specific address.

**Arguments:**
- `puzzleHashHex` (String!) - The puzzle hash as hex
- `daysBack` (Int) - Number of days to analyze (default: 30)

**Returns:** `AddressActivity` (nullable)

**Example:**
```graphql
query {
  addresses {
    addressActivity(
      puzzleHashHex: "0xabc..."
      daysBack: 90
    ) {
      totalCoinsReceived
      totalCoinsSpent
      totalValueReceived
      totalValueSpent
      uniqueSenders
      uniqueRecipients
      avgTransactionValue
      peakActivityDate
    }
  }
}
```

### 4. `addressTransactionHistory`

Get transaction history for an address.

**Arguments:**
- `puzzleHashHex` (String!) - The puzzle hash as hex
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)
- `includeSpends` (Boolean) - Include spend transactions (default: true)
- `includeReceives` (Boolean) - Include receive transactions (default: true)

**Returns:** `[AddressTransaction!]!`

**Example:**
```graphql
query {
  addresses {
    addressTransactionHistory(
      puzzleHashHex: "0xdef..."
      page: 1
      includeSpends: true
      includeReceives: true
    ) {
      coinId
      amount
      transactionType
      blockHeight
      blockTimestamp
      counterpartyPuzzleHash
    }
  }
}
```

### 5. `addressProfile`

Get comprehensive profile analysis for an address.

**Arguments:**
- `puzzleHashHex` (String!) - The puzzle hash as hex

**Returns:** `AddressProfile` (nullable)

**Example:**
```graphql
query {
  addresses {
    addressProfile(puzzleHashHex: "0x123...") {
      puzzleHashHex
      currentBalance
      totalCoinsReceived
      totalCoinsSpent
      totalValueReceived
      totalValueSpent
      uniqueInteractions
      lifespanBlocks
      avgTransactionValue
      addressType
      activityScore
      isActive
    }
  }
}
```

### 6. `addressBehaviorPatterns`

Analyze behavioral patterns across all addresses.

**Arguments:** None

**Returns:** `AddressBehaviorPatterns!`

**Example:**
```graphql
query {
  addresses {
    addressBehaviorPatterns {
      totalAddresses
      activeAddresses
      dormantAddresses
      hodlerAddresses
      traderAddresses
      whaleAddresses
      avgAddressLifespan
      avgTransactionsPerAddress
    }
  }
}
```

### 7. `addressRiskAssessment`

Get risk assessment for an address based on behavior patterns.

**Arguments:**
- `puzzleHashHex` (String!) - The puzzle hash as hex

**Returns:** `AddressRiskAssessment` (nullable)

**Example:**
```graphql
query {
  addresses {
    addressRiskAssessment(puzzleHashHex: "0x789...") {
      puzzleHashHex
      riskScore
      riskCategory
      avgTransactionSize
      transactionFrequency
      dormancyPeriods
      largeTransactionCount
      behaviorConsistency
      riskFactors
    }
  }
}
```

### 8. `addressInteractionNetwork`

Get the interaction network for an address.

**Arguments:**
- `puzzleHashHex` (String!) - The puzzle hash as hex
- `depth` (Int) - Network depth (default: 1, max: 3)
- `limit` (Int) - Max interactions per level (default: 20)

**Returns:** `[AddressInteraction!]!`

**Example:**
```graphql
query {
  addresses {
    addressInteractionNetwork(
      puzzleHashHex: "0xabc..."
      depth: 2
      limit: 10
    ) {
      fromAddress
      toAddress
      interactionCount
      totalValue
      firstInteraction
      lastInteraction
    }
  }
}
```

### 9. `addressBalanceEvolution`

Track balance changes over time for an address.

**Arguments:**
- `puzzleHashHex` (String!) - The puzzle hash as hex
- `blocksBack` (Int) - Number of blocks to analyze (default: 10000)

**Returns:** `[BalanceChange!]!`

**Example:**
```graphql
query {
  addresses {
    addressBalanceEvolution(
      puzzleHashHex: "0xdef..."
      blocksBack: 50000
    ) {
      blockHeight
      timestamp
      changeAmount
      newBalance
      transactionType
      description
    }
  }
}
```

### 10. `addressTimeAnalysis`

Analyze address activity patterns over time.

**Arguments:**
- `puzzleHashHex` (String!) - The puzzle hash as hex

**Returns:** `AddressTimeAnalysis` (nullable)

**Example:**
```graphql
query {
  addresses {
    addressTimeAnalysis(puzzleHashHex: "0x456...") {
      mostActiveHour
      mostActiveDayOfWeek
      avgTimeBetweenTransactions
      longestInactivePeriod
      weeklyActivityPattern {
        dayOfWeek
        transactionCount
      }
      hourlyActivityPattern {
        hour
        transactionCount
      }
    }
  }
}
```

### 11. `similarAddresses`

Find addresses with similar behavior patterns.

**Arguments:**
- `puzzleHashHex` (String!) - The puzzle hash as hex
- `limit` (Int) - Maximum results (default: 10)

**Returns:** `[SimilarAddress!]!`

**Example:**
```graphql
query {
  addresses {
    similarAddresses(
      puzzleHashHex: "0x789..."
      limit: 5
    ) {
      puzzleHashHex
      similarityScore
      currentBalance
      transactionCount
      commonPatterns
    }
  }
}
```

### 12. `addressAnomalyDetection`

Detect anomalous behavior for an address.

**Arguments:**
- `puzzleHashHex` (String!) - The puzzle hash as hex
- `sensitivity` (Float) - Anomaly detection sensitivity (default: 2.0)

**Returns:** `[AddressAnomaly!]!`

**Example:**
```graphql
query {
  addresses {
    addressAnomalyDetection(
      puzzleHashHex: "0xabc..."
      sensitivity: 1.5
    ) {
      anomalyType
      detectedAt
      severity
      description
      metrics {
        name
        value
        expectedValue
        deviation
      }
    }
  }
}
```

### 13. `addressCohortAnalysis`

Analyze addresses created in the same time period.

**Arguments:**
- `puzzleHashHex` (String!) - The puzzle hash as hex

**Returns:** `AddressCohortAnalysis` (nullable)

**Example:**
```graphql
query {
  addresses {
    addressCohortAnalysis(puzzleHashHex: "0xdef...") {
      cohortPeriod
      cohortSize
      avgCohortBalance
      cohortActivityLevel
      addressRankInCohort
      cohortRetentionRate
    }
  }
}
```

## Common Use Cases

### 1. Whale Watching

Monitor large balance holders:

```graphql
query WhaleWatch {
  addresses {
    addressesByBalance(page: 1, pageSize: 100) {
      puzzleHashHex
      currentBalance
      lastActivityBlock
    }
  }
}
```

### 2. Activity Monitoring

Track the most active addresses:

```graphql
query ActivityMonitor {
  addresses {
    mostActiveAddresses(page: 1) {
      puzzleHashHex
      totalTransactions
      currentBalance
      lastActivityBlock
    }
  }
}
```

### 3. Risk Assessment

Evaluate address risk for compliance:

```graphql
query RiskCheck($address: String!) {
  addresses {
    addressRiskAssessment(puzzleHashHex: $address) {
      riskScore
      riskCategory
      riskFactors
    }
    addressProfile(puzzleHashHex: $address) {
      addressType
      lifespanBlocks
      uniqueInteractions
    }
  }
}
```

### 4. Transaction Investigation

Deep dive into address transactions:

```graphql
query InvestigateAddress($address: String!) {
  addresses {
    addressTransactionHistory(puzzleHashHex: $address) {
      coinId
      amount
      transactionType
      blockTimestamp
      counterpartyPuzzleHash
    }
    addressInteractionNetwork(puzzleHashHex: $address, depth: 2) {
      fromAddress
      toAddress
      interactionCount
      totalValue
    }
  }
}
```

## Performance Considerations

1. **Use Pagination**: Always paginate results for queries returning lists
2. **Limit Network Depth**: Keep `depth` ≤ 2 for `addressInteractionNetwork`
3. **Time Ranges**: Use reasonable time ranges for historical analysis
4. **Caching**: Consider caching frequently accessed address profiles

## Error Handling

Common errors:
- `INVALID_INPUT`: Invalid hex format for puzzle hash
- `NOT_FOUND`: Address has no activity
- `LIMIT_EXCEEDED`: Query parameters exceed allowed limits

## Next Steps

- Explore [Balance Queries](balances.md) for wealth distribution
- Check [Transaction Queries](transactions.md) for transaction patterns
- See [Examples](../examples/address-examples.md) for advanced use cases 