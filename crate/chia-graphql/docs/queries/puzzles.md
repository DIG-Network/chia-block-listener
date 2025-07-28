# Puzzle Queries

The `puzzles` namespace provides analysis of CLVM (ChiaLisp Virtual Machine) puzzles used in the Chia blockchain. Puzzles define the spending conditions for coins and represent smart contracts in Chia.

## Available Queries

### 1. `byHash`

Get detailed information about a specific puzzle by its hash.

**Arguments:**
- `puzzleHash` (String!) - The puzzle hash as hex string

**Returns:** `PuzzleInfo` (nullable)

**Example:**
```graphql
query {
  puzzles {
    byHash(puzzleHash: "0xabc123...") {
      puzzleHash
      serializedPuzzle
      puzzleSize
      complexityScore
      firstUsedBlock
      lastUsedBlock
      totalCoins
      totalValue
      isStandardPuzzle
      puzzleType
      estimatedGas
    }
  }
}
```

### 2. `stats`

Get overall statistics about puzzle usage on the blockchain.

**Arguments:** None

**Returns:** `PuzzleStats!`

**Example:**
```graphql
query {
  puzzles {
    stats {
      totalPuzzles
      uniquePuzzles
      puzzlesWithReveals
      avgPuzzleSize
      avgRevealSize
      avgComplexityScore
      standardPuzzleRatio
      customPuzzleRatio
      mostUsedPuzzleType
    }
  }
}
```

### 3. `mostUsed`

Get the most frequently used puzzles on the blockchain.

**Arguments:**
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)
- `sortBy` (String) - Sort by: "USAGE", "VALUE", "RECENT" (default: "USAGE")

**Returns:** `[CommonPuzzle!]!`

**Example:**
```graphql
query {
  puzzles {
    mostUsed(page: 1, pageSize: 50, sortBy: "USAGE") {
      puzzleHash
      puzzleType
      usageCount
      totalValue
      firstUsed
      lastUsed
      avgCoinSize
      isStandard
      complexityScore
      description
    }
  }
}
```

### 4. `complexityAnalysis`

Analyze puzzle complexity distribution and gas usage patterns.

**Arguments:** None

**Returns:** `PuzzleComplexityAnalysis!`

**Example:**
```graphql
query {
  puzzles {
    complexityAnalysis {
      avgComplexity
      medianComplexity
      maxComplexity
      complexityDistribution {
        complexityRange
        puzzleCount
        percentageOfTotal
        avgGasUsage
      }
      gasEfficiencyStats {
        avgGasPerOperation
        mostEfficientPuzzles
        leastEfficientPuzzles
      }
    }
  }
}
```

## Type Definitions

### PuzzleInfo

Detailed information about a specific puzzle.

```graphql
type PuzzleInfo {
  # Puzzle hash identifier
  puzzleHash: String!
  
  # Serialized puzzle code (hex)
  serializedPuzzle: String
  
  # Size of puzzle in bytes
  puzzleSize: Int!
  
  # Complexity score (0-100)
  complexityScore: Float!
  
  # First block where puzzle was used
  firstUsedBlock: Int!
  
  # Most recent block where puzzle was used
  lastUsedBlock: Int!
  
  # Total number of coins using this puzzle
  totalCoins: Int!
  
  # Total value locked in coins with this puzzle
  totalValue: BigInt!
  
  # Whether this is a standard Chia puzzle type
  isStandardPuzzle: Boolean!
  
  # Classification of puzzle type
  puzzleType: String!
  
  # Estimated gas cost for execution
  estimatedGas: Int!
}
```

### PuzzleStats

Overall puzzle usage statistics.

```graphql
type PuzzleStats {
  # Total number of puzzle instances
  totalPuzzles: Int!
  
  # Number of unique puzzle hashes
  uniquePuzzles: Int!
  
  # Puzzles with revealed source code
  puzzlesWithReveals: Int!
  
  # Average puzzle size in bytes
  avgPuzzleSize: Float!
  
  # Average reveal size in bytes
  avgRevealSize: Float!
  
  # Average complexity score
  avgComplexityScore: Float!
  
  # Percentage using standard puzzles
  standardPuzzleRatio: Float!
  
  # Percentage using custom puzzles
  customPuzzleRatio: Float!
  
  # Most popular puzzle type
  mostUsedPuzzleType: String!
}
```

### CommonPuzzle

Information about frequently used puzzles.

```graphql
type CommonPuzzle {
  # Puzzle hash
  puzzleHash: String!
  
  # Classified puzzle type
  puzzleType: String!
  
  # Number of times used
  usageCount: Int!
  
  # Total value secured by this puzzle
  totalValue: BigInt!
  
  # First usage block
  firstUsed: Int!
  
  # Most recent usage block
  lastUsed: Int!
  
  # Average coin size using this puzzle
  avgCoinSize: Float!
  
  # Whether it's a standard puzzle type
  isStandard: Boolean!
  
  # Complexity score
  complexityScore: Float!
  
  # Human-readable description
  description: String!
}
```

### PuzzleComplexityAnalysis

Analysis of puzzle complexity patterns.

```graphql
type PuzzleComplexityAnalysis {
  # Average complexity across all puzzles
  avgComplexity: Float!
  
  # Median complexity score
  medianComplexity: Float!
  
  # Maximum complexity observed
  maxComplexity: Float!
  
  # Distribution of complexity ranges
  complexityDistribution: [ComplexityRange!]!
  
  # Gas efficiency statistics
  gasEfficiencyStats: GasEfficiencyStats!
}

type ComplexityRange {
  # Complexity range (e.g., "0-10", "11-20")
  complexityRange: String!
  
  # Number of puzzles in this range
  puzzleCount: Int!
  
  # Percentage of total puzzles
  percentageOfTotal: Float!
  
  # Average gas usage for this complexity
  avgGasUsage: Float!
}

type GasEfficiencyStats {
  # Average gas per operation
  avgGasPerOperation: Float!
  
  # Most gas-efficient puzzles
  mostEfficientPuzzles: [String!]!
  
  # Least gas-efficient puzzles
  leastEfficientPuzzles: [String!]!
}
```

## Common Use Cases

### 1. Smart Contract Analysis

Analyze smart contract usage patterns:

```graphql
query SmartContractAnalysis {
  puzzles {
    stats {
      totalPuzzles
      standardPuzzleRatio
      customPuzzleRatio
      avgComplexityScore
    }
    
    mostUsed(sortBy: "VALUE") {
      puzzleHash
      puzzleType
      totalValue
      usageCount
      complexityScore
    }
  }
}
```

### 2. Gas Optimization Research

Research gas usage and efficiency:

```graphql
query GasOptimization {
  puzzles {
    complexityAnalysis {
      avgComplexity
      gasEfficiencyStats {
        avgGasPerOperation
        mostEfficientPuzzles
        leastEfficientPuzzles
      }
      complexityDistribution {
        complexityRange
        avgGasUsage
        puzzleCount
      }
    }
  }
}
```

### 3. Security Auditing

Identify potentially risky or unusual puzzles:

```graphql
query SecurityAudit {
  puzzles {
    mostUsed(sortBy: "RECENT") {
      puzzleHash
      puzzleType
      isStandard
      complexityScore
      totalValue
      description
    }
  }
}
```

### 4. Puzzle Development Insights

Understand popular puzzle patterns:

```graphql
query DevelopmentInsights {
  puzzles {
    stats {
      mostUsedPuzzleType
      avgPuzzleSize
      puzzlesWithReveals
    }
    
    complexityAnalysis {
      complexityDistribution {
        complexityRange
        percentageOfTotal
      }
    }
  }
}
```

## Puzzle Types

### Standard Puzzle Types
- **P2_DELEGATED_PUZZLE**: Standard wallet puzzle
- **P2_SINGLETON**: Singleton (NFT) puzzle  
- **P2_CONDITIONS**: Direct conditions puzzle
- **CAT**: Chia Asset Token puzzle
- **DID**: Decentralized Identity puzzle

### Custom Puzzle Categories
- **SIMPLE**: Basic custom logic
- **COMPLEX**: Advanced smart contracts
- **EXPERIMENTAL**: Unusual or testing puzzles
- **UNKNOWN**: Unclassified custom puzzles

### Complexity Scoring
- **0-20**: Simple puzzles (basic conditions)
- **21-40**: Moderate complexity (standard features)
- **41-60**: Complex puzzles (advanced logic)
- **61-80**: Very complex (sophisticated contracts)
- **81-100**: Extremely complex (cutting-edge features)

## Gas Analysis

### Gas Efficiency Categories
- **HIGH**: < 50 gas per operation
- **MEDIUM**: 50-200 gas per operation
- **LOW**: 200-500 gas per operation
- **POOR**: > 500 gas per operation

### Optimization Metrics
- **Operations per Gas**: Higher is better
- **Size Efficiency**: Gas per byte of puzzle
- **Execution Efficiency**: Gas per logical operation

## Security Considerations

### Risk Indicators
- **Unusual Complexity**: Puzzles with very high complexity scores
- **Large Value**: High-value puzzles with custom logic
- **Recent Introduction**: New puzzle types with significant usage
- **No Reveals**: Puzzles without public source code

### Best Practices
- Prefer standard puzzle types when possible
- Review high-complexity custom puzzles carefully
- Monitor new puzzle introductions
- Validate puzzle logic before large transfers

## Performance Considerations

1. **Puzzle Reveals**: Large puzzles may take time to analyze
2. **Complexity Calculations**: CPU-intensive for complex puzzles
3. **Usage Statistics**: Cached and updated periodically
4. **Gas Estimates**: Approximate, actual usage may vary

## Error Handling

Common errors:
- `PUZZLE_NOT_FOUND`: Puzzle hash doesn't exist
- `INVALID_HASH`: Invalid puzzle hash format
- `REVEAL_UNAVAILABLE`: Puzzle code not revealed
- `ANALYSIS_FAILED`: Complexity analysis failed

## Development Tools

### Puzzle Analysis
- **Complexity Scoring**: Automated complexity assessment
- **Gas Estimation**: Predicted execution costs
- **Pattern Recognition**: Common puzzle pattern identification
- **Security Scanning**: Potential risk identification

### Debugging Support
- **Execution Tracing**: Step-by-step puzzle execution
- **Error Analysis**: Common failure pattern identification
- **Optimization Suggestions**: Performance improvement recommendations

## Next Steps

- Explore [Solution Queries](solutions.md) for puzzle execution analysis
- Check [Core Queries](core.md) for coin and puzzle hash relationships
- See [Examples](../examples/index.md) for puzzle analysis patterns 