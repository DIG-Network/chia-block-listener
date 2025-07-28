# CAT (Chia Asset Token) Queries

The `cats` namespace provides comprehensive analysis and tracking of Chia Asset Tokens (CATs). CATs are fungible tokens built on the Chia blockchain, similar to ERC-20 tokens on Ethereum.

## Available Queries

### 1. `assetById`

Get detailed information about a specific CAT token.

**Arguments:**
- `assetId` (String!) - The asset ID as hex string

**Returns:** `CatAsset` (nullable)

**Example:**
```graphql
query {
  cats {
    assetById(assetId: "0x123abc...") {
      assetId
      symbol
      name
      description
      totalSupply
      decimals
      creatorPuzzleHash
      createdBlock
      isVerified
      iconUrl
      websiteUrl
    }
  }
}
```

### 2. `balancesByOwner`

Get all CAT token balances for a specific address.

**Arguments:**
- `puzzleHashHex` (String!) - The address puzzle hash as hex
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)

**Returns:** `[CatBalance!]!`

**Example:**
```graphql
query {
  cats {
    balancesByOwner(
      puzzleHashHex: "0xdef456..."
      page: 1
      pageSize: 50
    ) {
      assetId
      symbol
      name
      balance
      balanceFormatted
      coinCount
      firstReceived
      lastActivity
      percentOfSupply
    }
  }
}
```

### 3. `holdersByAsset`

Get all holders of a specific CAT token.

**Arguments:**
- `assetId` (String!) - The asset ID as hex string
- `page` (Int) - Page number (default: 1)  
- `pageSize` (Int) - Items per page (default: 20)

**Returns:** `[CatHolder!]!`

**Example:**
```graphql
query {
  cats {
    holdersByAsset(
      assetId: "0x789def..."
      page: 1
      pageSize: 100
    ) {
      holderAddress
      balance
      balanceFormatted
      percentOfSupply
      coinCount
      firstReceived
      lastActivity
      holderCategory
    }
  }
}
```

### 4. `assetSummary`

Get summary statistics for all CAT tokens or filtered list.

**Arguments:**
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)
- `sortBy` (String) - Sort criteria: "HOLDERS", "SUPPLY", "ACTIVITY" (default: "HOLDERS")

**Returns:** `[CatAssetSummary!]!`

**Example:**
```graphql
query {
  cats {
    assetSummary(
      page: 1
      pageSize: 25
      sortBy: "HOLDERS"
    ) {
      assetId
      symbol
      name
      totalSupply
      uniqueHolders
      totalTransactions
      avgHolderBalance
      topHolderPercentage
      isActive
      lastActivityBlock
    }
  }
}
```

### 5. `tokenAnalytics`

Get comprehensive analytics for a specific CAT token.

**Arguments:**
- `assetId` (String!) - The asset ID as hex string

**Returns:** `TokenAnalytics` (nullable)

**Example:**
```graphql
query {
  cats {
    tokenAnalytics(assetId: "0xabc123...") {
      assetId
      uniqueHolders
      totalTransactions
      currentSupply
      circulatingSupply
      supplyUtilizationPercent
      avgTransactionSize
      avgHolderBalance
      medianHolderBalance
      holderGrowthRate
      transactionVelocity
      marketCapXch
      liquidityScore
    }
  }
}
```

### 6. `distributionAnalysis`

Analyze the distribution of token ownership.

**Arguments:**
- `assetId` (String!) - The asset ID as hex string
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)

**Returns:** `[HolderDistribution!]!`

**Example:**
```graphql
query {
  cats {
    distributionAnalysis(
      assetId: "0xdef789..."
      page: 1
    ) {
      holderAddress
      balance
      balancePercentage
      holderCategory
      acquisitionPattern
      holdingPeriod
      riskProfile
    }
  }
}
```

### 7. `tradingVelocity`

Analyze trading velocity and coin movement patterns.

**Arguments:**
- `assetId` (String!) - The asset ID as hex string
- `daysBack` (Int) - Number of days to analyze (default: 30)

**Returns:** `TokenVelocity` (nullable)

**Example:**
```graphql
query {
  cats {
    tradingVelocity(
      assetId: "0x456abc..."
      daysBack: 90
    ) {
      assetId
      periodDays
      coinsSpent
      coinsCreated
      valueTransferred
      uniqueAddressesActive
      velocityPercentage
      avgHoldTimeBlocks
      turnoverRate
      liquidityIndex
    }
  }
}
```

### 8. `holderBehavior`

Analyze holder behavior patterns for a token.

**Arguments:**
- `assetId` (String!) - The asset ID as hex string
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)

**Returns:** `[HolderBehavior!]!`

**Example:**
```graphql
query {
  cats {
    holderBehavior(
      assetId: "0x987fed..."
      page: 1
    ) {
      holderAddress
      behaviorType
      holdingPattern
      transactionFrequency
      avgTransactionSize
      totalTransactions
      holdingPeriod
      profitLossEstimate
      riskCategory
    }
  }
}
```

### 9. `tokenComparison`

Compare multiple CAT tokens side by side.

**Arguments:**
- `assetIds` ([String!]!) - List of asset IDs to compare

**Returns:** `[TokenComparison!]!`

**Example:**
```graphql
query {
  cats {
    tokenComparison(assetIds: [
      "0xabc123..."
      "0xdef456..."
      "0x789abc..."
    ]) {
      assetId
      symbol
      name
      uniqueHolders
      totalSupply
      marketCapXch
      velocityPercentage
      holderGrowthRate
      relativePerformance
      popularityRank
    }
  }
}
```

### 10. `liquidityAnalysis`

Analyze liquidity metrics for a CAT token.

**Arguments:**
- `assetId` (String!) - The asset ID as hex string

**Returns:** `LiquidityAnalysis` (nullable)

**Example:**
```graphql
query {
  cats {
    liquidityAnalysis(assetId: "0xfed987...") {
      assetId
      liquidityScore
      avgCoinSize
      medianCoinSize
      coinSizeStddev
      uniqueAddresses
      activeAddresses
      coinsSpent7d
      activeBlocks7d
      amountMoved7d
      largeCoinRatio
      dustRatio
      weeklyVelocityPercent
    }
  }
}
```

### 11. `topHolders`

Get the top holders of a specific CAT token.

**Arguments:**
- `assetId` (String!) - The asset ID as hex string
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)

**Returns:** `[TopHolder!]!`

**Example:**
```graphql
query {
  cats {
    topHolders(
      assetId: "0x123def..."
      page: 1
      pageSize: 50
    ) {
      holderAddress
      balance
      balancePercentage
    }
  }
}
```

## Common Use Cases

### 1. Token Portfolio Analysis

Get complete portfolio for an address:

```graphql
query TokenPortfolio($address: String!) {
  cats {
    balancesByOwner(puzzleHashHex: $address) {
      assetId
      symbol
      name
      balance
      balanceFormatted
      percentOfSupply
      lastActivity
    }
  }
}
```

### 2. Market Cap Ranking

Find most valuable tokens:

```graphql
query MarketCapRanking {
  cats {
    assetSummary(sortBy: "SUPPLY", pageSize: 50) {
      symbol
      name
      totalSupply
      uniqueHolders
      topHolderPercentage
    }
  }
}
```

### 3. Token Health Analysis

Comprehensive token analysis:

```graphql
query TokenHealth($assetId: String!) {
  cats {
    tokenAnalytics(assetId: $assetId) {
      uniqueHolders
      supplyUtilizationPercent
      transactionVelocity
      liquidityScore
    }
    
    tradingVelocity(assetId: $assetId, daysBack: 30) {
      velocityPercentage
      turnoverRate
      avgHoldTimeBlocks
    }
    
    liquidityAnalysis(assetId: $assetId) {
      liquidityScore
      weeklyVelocityPercent
      largeCoinRatio
    }
  }
}
```

### 4. Whale Detection

Find large holders and their behavior:

```graphql
query WhaleDetection($assetId: String!) {
  cats {
    distributionAnalysis(assetId: $assetId) {
      holderAddress
      balancePercentage
      holderCategory
      riskProfile
    }
    
    holderBehavior(assetId: $assetId) {
      holderAddress
      behaviorType
      holdingPattern
      riskCategory
    }
  }
}
```

## CAT Token Categories

Tokens are automatically categorized based on holder patterns:

### Holder Categories
- **Whale**: > 10% of supply
- **Large Holder**: 1-10% of supply  
- **Medium Holder**: 0.1-1% of supply
- **Small Holder**: < 0.1% of supply

### Behavior Types
- **HODLER**: Long-term holder, low transaction frequency
- **TRADER**: Active trading, high transaction frequency
- **ACCUMULATOR**: Steadily increasing balance
- **DISTRIBUTOR**: Steadily decreasing balance
- **INACTIVE**: No recent activity

### Risk Profiles
- **LOW**: Stable, long-term holding patterns
- **MEDIUM**: Moderate activity and volatility
- **HIGH**: High activity, large movements
- **CRITICAL**: Suspicious patterns or concentration

## Performance Metrics

### Velocity Calculations
- **Velocity %** = (Coins Spent in Period / Total Supply) × 100
- **Turnover Rate** = Coins Spent / Average Holdings
- **Hold Time** = Average blocks between receive and spend

### Liquidity Scoring
- **0-25**: Low liquidity (inactive token)
- **26-50**: Fair liquidity (moderate activity)
- **51-75**: Good liquidity (active trading)
- **76-100**: High liquidity (very active)

## Error Handling

Common errors:
- `INVALID_INPUT`: Invalid asset ID format
- `TOKEN_NOT_FOUND`: Asset ID doesn't exist
- `INSUFFICIENT_DATA`: Not enough transaction history
- `RATE_LIMITED`: Too many requests

## Performance Tips

1. **Use Asset Summaries**: For overviews, use `assetSummary` instead of individual lookups
2. **Limit Time Ranges**: Keep `daysBack` parameters reasonable (≤ 365)
3. **Paginate Results**: Always use pagination for holder lists
4. **Cache Token Data**: Token metadata changes infrequently

## Next Steps

- Explore [NFT Queries](nfts.md) for non-fungible token analytics
- Check [Address Queries](addresses.md) for holder behavior analysis
- See [Examples](../examples/index.md) for advanced CAT analysis patterns 