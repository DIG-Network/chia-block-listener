# NFT (Non-Fungible Token) Queries

The `nfts` namespace provides comprehensive analysis of NFT collections, ownership, trading patterns, and marketplace dynamics on the Chia blockchain.

## Available Queries

### 1. `byOwner`

Get all NFTs owned by a specific address.

**Arguments:**
- `puzzleHashHex` (String!) - The owner's puzzle hash as hex
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)

**Returns:** `[NftOwnership!]!`

**Example:**
```graphql
query {
  nfts {
    byOwner(
      puzzleHashHex: "0xabc123..."
      page: 1
      pageSize: 50
    ) {
      nftId
      collectionId
      collectionName
      editionNumber
      totalEditions
      metadata
      currentOwner
      previousOwner
      ownedSince
      acquisitionBlock
      estimatedValue
    }
  }
}
```

### 2. `byCollection`

Get all NFTs in a specific collection.

**Arguments:**
- `collectionId` (String!) - The collection ID
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)
- `sortBy` (String) - Sort by: "EDITION", "RARITY", "RECENT" (default: "EDITION")

**Returns:** `[NftCollectionItem!]!`

**Example:**
```graphql
query {
  nfts {
    byCollection(
      collectionId: "col1..."
      page: 1
      sortBy: "RARITY"
    ) {
      nftId
      editionNumber
      totalEditions
      rarityScore
      rarityRank
      traits
      metadata
      currentOwner
      isForSale
      lastSalePrice
      lastTransferBlock
    }
  }
}
```

### 3. `collectionById`

Get detailed information about an NFT collection.

**Arguments:**
- `collectionId` (String!) - The collection ID

**Returns:** `NftCollection` (nullable)

**Example:**
```graphql
query {
  nfts {
    collectionById(collectionId: "col1...") {
      collectionId
      name
      description
      totalSupply
      creatorPuzzleHash
      createdBlock
      verified
      floorPrice
      ceilingPrice
      totalVolume
      uniqueOwners
      royaltyPercentage
      website
      imageUrl
      bannerUrl
    }
  }
}
```

### 4. `ownershipHistory`

Track ownership changes for NFTs in a collection.

**Arguments:**
- `collectionId` (String!) - The collection ID
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)
- `nftId` (String) - Specific NFT ID (optional)

**Returns:** `[NftOwnershipHistory!]!`

**Example:**
```graphql
query {
  nfts {
    ownershipHistory(
      collectionId: "col1..."
      page: 1
      nftId: "nft123..."
    ) {
      nftId
      fromAddress
      toAddress
      transferBlock
      transferTimestamp
      transferType
      salePrice
      marketplaceFee
      royaltyPaid
      transactionHash
    }
  }
}
```

### 5. `collectionAnalytics`

Get comprehensive analytics for an NFT collection.

**Arguments:**
- `collectionId` (String!) - The collection ID

**Returns:** `CollectionAnalytics` (nullable)

**Example:**
```graphql
query {
  nfts {
    collectionAnalytics(collectionId: "col1...") {
      collectionId
      totalNfts
      uniqueOwners
      ownershipDistributionPercent
      floorPrice
      averagePrice
      ceilingPrice
      totalVolume
      volume24h
      volume7d
      volume30d
      salesCount
      saleRatioPercent
      avgHoldingPeriod
      ownershipConcentration
      liquidityScore
    }
  }
}
```

## Type Definitions

### NftCollection

Core collection information.

```graphql
type NftCollection {
  # Unique collection identifier
  collectionId: String!
  
  # Collection name
  name: String!
  
  # Collection description
  description: String
  
  # Total number of NFTs in collection
  totalSupply: Int!
  
  # Creator's address
  creatorPuzzleHash: String!
  
  # Block when collection was created
  createdBlock: Int!
  
  # Whether collection is verified
  verified: Boolean!
  
  # Current floor price
  floorPrice: BigInt
  
  # Highest price paid
  ceilingPrice: BigInt
  
  # Total trading volume
  totalVolume: BigInt!
  
  # Number of unique owners
  uniqueOwners: Int!
  
  # Royalty percentage (0-100)
  royaltyPercentage: Float
  
  # Collection website
  website: String
  
  # Collection image URL
  imageUrl: String
  
  # Collection banner URL
  bannerUrl: String
}
```

### NftOwnership

NFT ownership details.

```graphql
type NftOwnership {
  # NFT identifier
  nftId: String!
  
  # Parent collection ID
  collectionId: String!
  
  # Collection name
  collectionName: String!
  
  # Edition number within collection
  editionNumber: Int!
  
  # Total editions in collection
  totalEditions: Int!
  
  # NFT metadata (JSON)
  metadata: String
  
  # Current owner address
  currentOwner: String!
  
  # Previous owner address
  previousOwner: String
  
  # Timestamp when acquired
  ownedSince: String!
  
  # Block when acquired
  acquisitionBlock: Int!
  
  # Estimated current value
  estimatedValue: BigInt
}
```

### CollectionAnalytics

Comprehensive collection analytics.

```graphql
type CollectionAnalytics {
  # Collection identifier
  collectionId: String!
  
  # Total NFTs in collection
  totalNfts: Int!
  
  # Number of unique owners
  uniqueOwners: Int!
  
  # Percentage of NFTs with unique owners
  ownershipDistributionPercent: Float!
  
  # Lowest listed/sold price
  floorPrice: BigInt
  
  # Average sale price
  averagePrice: BigInt
  
  # Highest price paid
  ceilingPrice: BigInt
  
  # Total trading volume
  totalVolume: BigInt!
  
  # 24-hour volume
  volume24h: BigInt!
  
  # 7-day volume
  volume7d: BigInt!
  
  # 30-day volume
  volume30d: BigInt!
  
  # Total number of sales
  salesCount: Int!
  
  # Percentage of NFTs that have been sold
  saleRatioPercent: Float!
  
  # Average holding period in blocks
  avgHoldingPeriod: Float!
  
  # How concentrated ownership is (0-100)
  ownershipConcentration: Float!
  
  # Liquidity score (0-100)
  liquidityScore: Float!
}
```

## Common Use Cases

### 1. Collection Overview

Get complete collection statistics:

```graphql
query CollectionOverview($collectionId: String!) {
  nfts {
    collectionById(collectionId: $collectionId) {
      name
      totalSupply
      uniqueOwners
      floorPrice
      totalVolume
      verified
    }
    
    collectionAnalytics(collectionId: $collectionId) {
      ownershipDistributionPercent
      saleRatioPercent
      liquidityScore
      volume30d
    }
  }
}
```

### 2. Portfolio Analysis

Analyze NFT portfolio for an address:

```graphql
query NFTPortfolio($ownerAddress: String!) {
  nfts {
    byOwner(puzzleHashHex: $ownerAddress) {
      nftId
      collectionName
      editionNumber
      ownedSince
      estimatedValue
    }
  }
}
```

### 3. Market Activity Tracking

Monitor recent collection activity:

```graphql
query MarketActivity($collectionId: String!) {
  nfts {
    ownershipHistory(
      collectionId: $collectionId
      page: 1
      pageSize: 50
    ) {
      nftId
      fromAddress
      toAddress
      transferTimestamp
      transferType
      salePrice
    }
  }
}
```

### 4. Rarity Analysis

Find rare NFTs in a collection:

```graphql
query RarityAnalysis($collectionId: String!) {
  nfts {
    byCollection(
      collectionId: $collectionId
      sortBy: "RARITY"
      pageSize: 20
    ) {
      nftId
      editionNumber
      rarityScore
      rarityRank
      traits
      currentOwner
      lastSalePrice
    }
  }
}
```

## NFT Categories

### Transfer Types
- **MINT**: Initial creation/minting
- **SALE**: Marketplace sale transaction
- **TRANSFER**: Direct transfer between addresses
- **GIFT**: Zero-value transfer
- **UNKNOWN**: Unclassified transfer

### Rarity Tiers
- **LEGENDARY**: Top 1% rarest
- **EPIC**: Top 5% rarest
- **RARE**: Top 10% rarest
- **UNCOMMON**: Top 25% rarest
- **COMMON**: Bottom 75%

### Ownership Concentration
- **DISTRIBUTED**: < 50% owned by top 10 holders
- **MODERATE**: 50-70% owned by top 10 holders
- **CONCENTRATED**: 70-90% owned by top 10 holders
- **WHALE-DOMINATED**: > 90% owned by top 10 holders

## Analytics Metrics

### Liquidity Scoring
- **0-25**: Illiquid (very rare sales)
- **26-50**: Low liquidity (occasional sales)
- **51-75**: Good liquidity (regular sales)
- **76-100**: High liquidity (frequent sales)

### Floor Price Calculation
- Lowest current listing price
- If no listings, lowest recent sale price
- Excludes obvious outliers (< 10% of average)

### Volume Metrics
- **24h Volume**: Trading volume in last 24 hours
- **7d Volume**: Trading volume in last 7 days
- **30d Volume**: Trading volume in last 30 days
- **All-time Volume**: Total trading volume since launch

## Performance Considerations

1. **Collection Size**: Large collections may require pagination
2. **History Depth**: Limit ownership history queries for performance
3. **Metadata Loading**: NFT metadata can be large, request selectively
4. **Real-time Data**: Some metrics update with delay

## Marketplace Integration

The NFT queries support integration with various marketplaces:

- **MintGarden**: Primary Chia NFT marketplace
- **SpaceScan**: NFT explorer and marketplace
- **Dexie**: Decentralized exchange with NFT support
- **Custom Marketplaces**: Any platform using standard Chia NFTs

## Error Handling

Common errors:
- `COLLECTION_NOT_FOUND`: Collection ID doesn't exist
- `NFT_NOT_FOUND`: NFT ID doesn't exist
- `INVALID_INPUT`: Invalid collection/NFT ID format
- `METADATA_UNAVAILABLE`: NFT metadata couldn't be loaded

## Next Steps

- Explore [CAT Queries](cats.md) for fungible token analytics
- Check [Address Queries](addresses.md) for holder behavior analysis
- See [Examples](../examples/index.md) for advanced NFT analysis patterns 