# Core Types

Core types represent fundamental blockchain data structures used throughout the Chia GraphQL API.

## CoinInfo

Represents a coin in the Chia blockchain using the coinset model.

```graphql
type CoinInfo {
  # Unique coin identifier as hex string
  coinId: String!
  
  # Parent coin ID as hex string
  parentCoinId: String!
  
  # Puzzle hash (address) as hex string
  puzzleHash: String!
  
  # Amount in mojos (1 XCH = 1e12 mojos)
  amount: BigInt!
  
  # Block height where this coin was created
  createdBlock: Int!
  
  # Block height where this coin was spent (null if unspent)
  spentBlock: Int
}
```

### Usage Example

```graphql
query {
  core {
    coinById(coinId: "0x123...") {
      coinId
      amount
      puzzleHash
      spentBlock  # Check if null to determine if unspent
    }
  }
}
```

### Key Points
- Coins are immutable once created
- A coin is either spent or unspent (no partial spends)
- The `coinId` is derived from parent ID, puzzle hash, and amount

## BlockInfo

Represents a block in the Chia blockchain.

```graphql
type BlockInfo {
  # Block height (sequential number)
  height: Int!
  
  # Block hash as hex string
  blockHash: String!
  
  # Previous block hash as hex string
  prevHash: String!
  
  # Block timestamp (ISO 8601 format)
  timestamp: String!
  
  # Number of coins created in this block
  coinCount: Int!
  
  # Number of coins spent in this block
  spendCount: Int!
}
```

### Usage Example

```graphql
query {
  core {
    blockByHeight(height: 1000000) {
      height
      blockHash
      timestamp
      coinCount
      spendCount
    }
  }
}
```

### Key Points
- Block height is sequential and unique
- Timestamp represents when the block was created
- `coinCount` includes all new coins (rewards + outputs)
- `spendCount` represents coins consumed in the block

## BalanceInfo

Aggregated balance information for a puzzle hash (address).

```graphql
type BalanceInfo {
  # Puzzle hash as hex string
  puzzleHash: String!
  
  # Confirmed balance in mojos
  confirmedBalance: BigInt!
  
  # Unconfirmed balance in mojos (currently always 0)
  unconfirmedBalance: BigInt!
  
  # Number of unspent coins for this address
  unspentCoinCount: Int!
  
  # First block where this address was active
  firstActivityBlock: Int
  
  # Most recent block with activity
  lastActivityBlock: Int
}
```

### Usage Example

```graphql
query {
  core {
    balanceByPuzzleHash(puzzleHashHex: "0xabc...") {
      confirmedBalance
      unspentCoinCount
      firstActivityBlock
      lastActivityBlock
    }
  }
}
```

### Key Points
- Balance is the sum of all unspent coins
- `unconfirmedBalance` reserved for future mempool support
- Activity blocks help track address lifespan

## Coin (Direct Query Type)

Used for direct coin queries at the root level.

```graphql
type Coin {
  # Unique coin ID
  coinId: String!
  
  # Parent coin info as hex
  parentCoinInfo: String!
  
  # Puzzle hash as hex
  puzzleHash: String!
  
  # Amount in mojos
  amount: BigInt!
  
  # Block where created (nullable for compatibility)
  createdBlock: Int
}
```

### Usage Example

```graphql
query {
  coin(coinId: "0x456...") {
    coinId
    amount
    puzzleHash
    createdBlock
  }
}
```

## Block (Direct Query Type)

Used for direct block queries at the root level.

```graphql
type Block {
  # Block height
  height: Int!
  
  # Block weight (cumulative difficulty)
  weight: Int!
  
  # Header hash as hex
  headerHash: String!
  
  # Block timestamp
  timestamp: String!
}
```

### Usage Example

```graphql
query {
  block(height: 2000000) {
    height
    weight
    headerHash
    timestamp
  }
}
```

## Value Conversions

### Mojos to XCH

```javascript
// Convert mojos to XCH
const xch = mojos / 1_000_000_000_000;

// Convert XCH to mojos
const mojos = xch * 1_000_000_000_000;
```

### Common Values

- 1 XCH = 1,000,000,000,000 mojos
- Dust threshold: < 1 mojo
- Small coin: < 10,000,000,000 mojos (0.01 XCH)
- Medium coin: 10,000,000,000 - 1,000,000,000,000 mojos (0.01 - 1 XCH)
- Large coin: > 1,000,000,000,000 mojos (> 1 XCH)

## Hex String Handling

All binary data is represented as hex strings:

```javascript
// With 0x prefix
const coinId = "0x1234567890abcdef...";

// Without 0x prefix (also accepted)
const coinId = "1234567890abcdef...";
```

## Null Handling

Fields that may be null:

- `CoinInfo.spentBlock` - null if coin is unspent
- `BalanceInfo.firstActivityBlock` - null if no activity
- `BalanceInfo.lastActivityBlock` - null if no activity
- Query results - null if not found

Always check for null values:

```javascript
if (coin.spentBlock === null) {
  console.log("Coin is unspent");
}
```

## Performance Tips

1. **Batch queries** - Combine multiple queries in one request
2. **Specific fields** - Only request fields you need
3. **Use aggregates** - Use `BalanceInfo` instead of summing coins
4. **Cache coin data** - Coins are immutable once created

## Related Types

- [Address Types](addresses.md) - Extended address analytics
- [Balance Types](balances.md) - Balance distribution analysis
- [Network Types](network.md) - Network-wide statistics

## Next Steps

- See [Core Queries](../queries/core.md) for usage examples
- Explore [Address Types](addresses.md) for address analytics
- Check [Examples](../examples/index.md) for real-world usage 