# Core Queries

The `core` namespace provides fundamental blockchain data queries for coins, blocks, and balances. These are the building blocks for exploring the Chia blockchain.

## Available Queries

### 1. `coinById`

Get detailed information about a specific coin.

**Arguments:**
- `coinId` (String!) - The coin ID as a hex string

**Returns:** `CoinInfo` (nullable)

**Example:**
```graphql
query {
  core {
    coinById(coinId: "0x1234567890abcdef...") {
      coinId
      parentCoinId
      puzzleHash
      amount
      createdBlock
      spentBlock
    }
  }
}
```

### 2. `coinsByPuzzleHash`

Get all coins associated with a specific puzzle hash (address).

**Arguments:**
- `puzzleHashHex` (String!) - The puzzle hash as a hex string
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)

**Returns:** `[CoinInfo!]!`

**Example:**
```graphql
query {
  core {
    coinsByPuzzleHash(
      puzzleHashHex: "0xabcd..."
      page: 1
      pageSize: 50
    ) {
      coinId
      amount
      createdBlock
      spentBlock
    }
  }
}
```

### 3. `blockByHeight`

Get information about a specific block.

**Arguments:**
- `height` (Int!) - The block height

**Returns:** `BlockInfo` (nullable)

**Example:**
```graphql
query {
  core {
    blockByHeight(height: 1000000) {
      height
      blockHash
      prevHash
      timestamp
      coinCount
      spendCount
    }
  }
}
```

### 4. `recentBlocks`

Get the most recent blocks.

**Arguments:**
- `limit` (Int) - Number of blocks to return (default: 10)

**Returns:** `[BlockInfo!]!`

**Example:**
```graphql
query {
  core {
    recentBlocks(limit: 20) {
      height
      blockHash
      timestamp
      coinCount
      spendCount
    }
  }
}
```

### 5. `balanceByPuzzleHash`

Get the balance information for a specific puzzle hash (address).

**Arguments:**
- `puzzleHashHex` (String!) - The puzzle hash as a hex string

**Returns:** `BalanceInfo` (nullable)

**Example:**
```graphql
query {
  core {
    balanceByPuzzleHash(puzzleHashHex: "0xdef...") {
      puzzleHash
      confirmedBalance
      unconfirmedBalance
      unspentCoinCount
      firstActivityBlock
      lastActivityBlock
    }
  }
}
```

## Type Definitions

### CoinInfo

Represents a coin in the Chia blockchain.

```graphql
type CoinInfo {
  # Unique coin ID as hex
  coinId: String!
  
  # Parent coin ID as hex
  parentCoinId: String!
  
  # Puzzle hash as hex
  puzzleHash: String!
  
  # Amount in mojos
  amount: BigInt!
  
  # Block height where created
  createdBlock: Int!
  
  # Block height where spent (null if unspent)
  spentBlock: Int
}
```

### BlockInfo

Represents a block in the blockchain.

```graphql
type BlockInfo {
  # Block height
  height: Int!
  
  # Block hash as hex
  blockHash: String!
  
  # Previous block hash as hex
  prevHash: String!
  
  # Block timestamp
  timestamp: String!
  
  # Number of coins created in block
  coinCount: Int!
  
  # Number of coins spent in block
  spendCount: Int!
}
```

### BalanceInfo

Balance information for an address.

```graphql
type BalanceInfo {
  # Puzzle hash as hex
  puzzleHash: String!
  
  # Confirmed balance in mojos
  confirmedBalance: BigInt!
  
  # Unconfirmed balance in mojos
  unconfirmedBalance: BigInt!
  
  # Number of unspent coins
  unspentCoinCount: Int!
  
  # First block with activity
  firstActivityBlock: Int
  
  # Last block with activity
  lastActivityBlock: Int
}
```

## Common Use Cases

### 1. Check if a coin is spent

```graphql
query CheckCoinStatus($coinId: String!) {
  core {
    coinById(coinId: $coinId) {
      coinId
      spentBlock
    }
  }
}
```

If `spentBlock` is null, the coin is unspent.

### 2. Get all unspent coins for an address

```graphql
query UnspentCoins($puzzleHash: String!) {
  core {
    coinsByPuzzleHash(puzzleHashHex: $puzzleHash) {
      coinId
      amount
      createdBlock
      spentBlock
    }
  }
}
```

Filter results where `spentBlock` is null.

### 3. Calculate total balance

```graphql
query AddressBalance($puzzleHash: String!) {
  core {
    balanceByPuzzleHash(puzzleHashHex: $puzzleHash) {
      confirmedBalance
      unspentCoinCount
    }
  }
}
```

### 4. Monitor recent blockchain activity

```graphql
query RecentActivity {
  core {
    recentBlocks(limit: 5) {
      height
      timestamp
      coinCount
      spendCount
    }
  }
}
```

## Best Practices

1. **Hex String Format**: All puzzle hashes and coin IDs must be provided as hex strings with or without the `0x` prefix.

2. **Pagination**: Use pagination for `coinsByPuzzleHash` when dealing with addresses that have many coins.

3. **Null Checks**: Always check for null returns, especially for `coinById` and `blockByHeight`.

4. **Balance Calculation**: Use `balanceByPuzzleHash` for efficient balance queries instead of summing coins manually.

## Error Handling

Common errors:

- `INVALID_INPUT`: Invalid hex string format
- `NOT_FOUND`: Coin or block doesn't exist
- `DATABASE_ERROR`: Database connection issues

Example error response:
```json
{
  "errors": [{
    "message": "Invalid input: Invalid hex string",
    "path": ["core", "coinById"],
    "extensions": {
      "code": "INVALID_INPUT"
    }
  }]
}
```

## Performance Tips

1. Use specific queries instead of fetching all data
2. Leverage the `balanceByPuzzleHash` for balance checks
3. Implement client-side caching for frequently accessed coins
4. Use batch queries when checking multiple coins

## Next Steps

- Explore [Address Queries](addresses.md) for advanced address analytics
- Check [Balance Queries](balances.md) for distribution analysis
- See [Examples](../examples/core-examples.md) for more use cases 