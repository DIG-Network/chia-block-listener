# GraphQL Query Examples

This section provides real-world examples of using the Chia GraphQL API for common blockchain exploration tasks.

## Example Categories

1. **[Basic Blockchain Queries](#basic-blockchain-queries)** - Simple queries to get started
2. **[Address Analysis](#address-analysis)** - Investigating address behavior
3. **[Token Analytics](#token-analytics)** - CAT and NFT analysis
4. **[Network Monitoring](#network-monitoring)** - Blockchain health and statistics
5. **[Advanced Analytics](#advanced-analytics)** - Complex multi-query analysis

## Basic Blockchain Queries

### Get Current Status

```graphql
query BlockchainStatus {
  currentHeight
  
  core {
    recentBlocks(limit: 5) {
      height
      timestamp
      coinCount
      spendCount
    }
  }
  
  network {
    networkStats {
      totalBlocks
      totalCoins
      totalAddresses
      avgBlockTimeSeconds
    }
  }
}
```

### Check Coin Status

```graphql
query CoinStatus($coinId: String!) {
  coin(coinId: $coinId) {
    coinId
    amount
    puzzleHash
    createdBlock
  }
  
  core {
    coinById(coinId: $coinId) {
      spentBlock
      parentCoinId
    }
  }
}
```

### Get Address Balance

```graphql
query AddressBalance($address: String!) {
  core {
    balanceByPuzzleHash(puzzleHashHex: $address) {
      confirmedBalance
      unspentCoinCount
      firstActivityBlock
      lastActivityBlock
    }
    
    coinsByPuzzleHash(puzzleHashHex: $address, pageSize: 10) {
      coinId
      amount
      createdBlock
      spentBlock
    }
  }
}
```

## Address Analysis

### Complete Address Profile

```graphql
query AddressInvestigation($address: String!) {
  addresses {
    addressProfile(puzzleHashHex: $address) {
      currentBalance
      totalCoinsReceived
      totalCoinsSpent
      uniqueInteractions
      lifespanBlocks
      addressType
      activityScore
    }
    
    addressRiskAssessment(puzzleHashHex: $address) {
      riskScore
      riskCategory
      transactionFrequency
      largeTransactionCount
      riskFactors
    }
    
    addressActivity(puzzleHashHex: $address, daysBack: 30) {
      totalValueReceived
      totalValueSpent
      uniqueSenders
      uniqueRecipients
      peakActivityDate
    }
  }
}
```

### Transaction History with Counterparties

```graphql
query TransactionHistory($address: String!, $page: Int!) {
  addresses {
    addressTransactionHistory(
      puzzleHashHex: $address
      page: $page
      pageSize: 50
    ) {
      coinId
      amount
      transactionType
      blockHeight
      blockTimestamp
      counterpartyPuzzleHash
    }
    
    addressInteractionNetwork(
      puzzleHashHex: $address
      depth: 2
      limit: 20
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

### Whale Tracking

```graphql
query WhaleWatch {
  addresses {
    addressesByBalance(page: 1, pageSize: 100) {
      puzzleHashHex
      currentBalance
      coinCount
      lastSeen
    }
  }
  
  balances {
    richestAddresses(page: 1, pageSize: 20) {
      puzzleHashHex
      totalXch
      percentOfSupply
      rankPosition
    }
  }
}
```

## Token Analytics

### CAT Token Overview

```graphql
query CATAnalysis($assetId: String!) {
  cats {
    assetById(assetId: $assetId) {
      assetId
      symbol
      name
      totalSupply
      decimals
    }
    
    tokenAnalytics(assetId: $assetId) {
      uniqueHolders
      currentSupply
      supplyUtilizationPercent
      marketCapXch
    }
    
    topHolders(assetId: $assetId, page: 1, pageSize: 10) {
      holderAddress
      balance
      balancePercentage
    }
    
    tradingVelocity(assetId: $assetId, daysBack: 30) {
      periodDays
      coinsSpent
      valueTransferred
      uniqueAddressesActive
      velocityPercentage
    }
  }
}
```

### NFT Collection Analysis

```graphql
query NFTCollection($collectionId: String!) {
  nfts {
    collectionById(collectionId: $collectionId) {
      collectionId
      name
      totalSupply
      creatorPuzzleHash
    }
    
    collectionAnalytics(collectionId: $collectionId) {
      totalNfts
      uniqueOwners
      floorPrice
      totalVolume
      saleRatioPercent
    }
    
    ownershipHistory(
      collectionId: $collectionId
      page: 1
      pageSize: 20
    ) {
      nftId
      fromAddress
      toAddress
      transferBlock
      transferTimestamp
    }
  }
}
```

## Network Monitoring

### Health Dashboard

```graphql
query NetworkHealth {
  network {
    networkStats {
      currentHeight
      totalCoins
      totalAddresses
      avgBlockTimeSeconds
    }
    
    blockProductionStats {
      avgBlockTimeSeconds
      blockTimeStddev
      blocksToday
      blocksThisWeek
    }
    
    throughputAnalysis(hoursBack: 24) {
      tps
      coinsPerSecond
      valuePerSecond
      peakTps
    }
    
    unspentCoinAnalysis {
      totalUnspentCoins
      totalUnspentValue
      avgCoinValue
      dustCoins
      avgCoinAgeBlocks
    }
  }
}
```

### Growth Tracking

```graphql
query NetworkGrowth($days: Int!) {
  network {
    growthMetrics(daysBack: $days) {
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
  
  temporal {
    growthTrends(daysBack: $days, periodType: "day") {
      period
      blockCount
      coinCount
      transactionCount
      uniqueAddresses
      growthRate
    }
  }
}
```

## Advanced Analytics

### Multi-Dimensional Analysis

```graphql
query ComprehensiveAnalysis {
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
  
  transactions {
    spendVelocityAnalysis {
      avgBlocksToSpend
      immediateSpends
      quickSpends
      totalCoinsAnalyzed
    }
    
    feeAnalysis(hoursBack: 24) {
      totalFees
      avgFeePerTransaction
      zeroFeeTransactions
      highFeeTransactions
    }
  }
}
```

### Time-Based Pattern Analysis

```graphql
query TemporalPatterns {
  temporal {
    activityHeatmap(daysBack: 7) {
      hourOfDay
      dayOfWeek
      totalTransactions
      avgValuePerTransaction
      peakActivityScore
    }
    
    seasonalPatterns(monthsBack: 12) {
      period
      avgDailyTransactions
      avgDailyVolume
      seasonalIndex
      trend
    }
  }
  
  addresses {
    addressTimeAnalysis(puzzleHashHex: "0x...") {
      mostActiveHour
      mostActiveDayOfWeek
      avgTimeBetweenTransactions
      weeklyActivityPattern {
        dayOfWeek
        transactionCount
      }
    }
  }
}
```

### Complex Transaction Patterns

```graphql
query TransactionPatterns {
  transactions {
    complexTransactionPatterns(page: 1, pageSize: 20) {
      spentBlock
      blockTimestamp
      coinsSpent
      coinsCreated
      totalValueIn
      totalValueOut
      feeAmount
      uniquePuzzlesUsed
      complexityScore
    }
    
    patternsByType {
      transactionType
      count
      avgValueIn
      avgValueOut
      avgFee
    }
  }
}
```

## Pagination Examples

### Iterating Through Large Result Sets

```graphql
query PaginatedResults($page: Int!) {
  addresses {
    mostActiveAddresses(page: $page, pageSize: 100) {
      puzzleHashHex
      totalTransactions
      currentBalance
    }
  }
}

# Variables:
{
  "page": 1
}
```

### Cursor-Based Pattern (Client-Side)

```javascript
// Client-side cursor pagination
async function getAllActiveAddresses() {
  const addresses = [];
  let page = 1;
  let hasMore = true;
  
  while (hasMore) {
    const result = await graphql({
      query: PAGINATED_QUERY,
      variables: { page, pageSize: 100 }
    });
    
    addresses.push(...result.addresses.mostActiveAddresses);
    hasMore = result.addresses.mostActiveAddresses.length === 100;
    page++;
  }
  
  return addresses;
}
```

## Performance Optimization

### Efficient Field Selection

```graphql
# Good: Only request needed fields
query EfficientQuery {
  network {
    networkStats {
      currentHeight
      totalCoins
    }
  }
}

# Avoid: Requesting all fields
query InefficientQuery {
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

### Batch Queries

```graphql
query BatchAnalysis($addresses: [String!]!) {
  address1: core {
    balanceByPuzzleHash(puzzleHashHex: $addresses[0]) {
      confirmedBalance
    }
  }
  
  address2: core {
    balanceByPuzzleHash(puzzleHashHex: $addresses[1]) {
      confirmedBalance
    }
  }
  
  address3: core {
    balanceByPuzzleHash(puzzleHashHex: $addresses[2]) {
      confirmedBalance
    }
  }
}
```

## Error Handling

### Graceful Error Handling

```javascript
const query = `
  query SafeQuery($address: String!) {
    core {
      balanceByPuzzleHash(puzzleHashHex: $address) {
        confirmedBalance
      }
    }
  }
`;

try {
  const result = await graphqlClient.request(query, {
    address: "0xinvalid"
  });
} catch (error) {
  if (error.response?.errors?.[0]?.extensions?.code === 'INVALID_INPUT') {
    console.log('Invalid address format');
  }
}
```

## Next Steps

- Review specific [Query Documentation](../queries/core.md) for detailed field descriptions
- Check [Type Definitions](../types/index.md) for complete type reference
- See [Performance Guide](../performance.md) for optimization tips 