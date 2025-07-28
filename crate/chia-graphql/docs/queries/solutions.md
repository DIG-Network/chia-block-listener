# Solution Queries

The `solutions` namespace provides analysis of CLVM puzzle solutions - the arguments and execution data that satisfy puzzle conditions when spending coins on the Chia blockchain.

## Available Queries

### 1. `byHash`

Get detailed information about a specific solution by its hash.

**Arguments:**
- `solutionHash` (String!) - The solution hash as hex string

**Returns:** `SolutionInfo` (nullable)

**Example:**
```graphql
query {
  solutions {
    byHash(solutionHash: "0xdef456...") {
      solutionHash
      serializedSolution
      solutionSize
      complexityScore
      executionCost
      firstUsedBlock
      lastUsedBlock
      usageCount
      associatedPuzzleHash
      solutionType
      isSuccessful
    }
  }
}
```

### 2. `stats`

Get overall statistics about solution usage and patterns.

**Arguments:** None

**Returns:** `SolutionStats!`

**Example:**
```graphql
query {
  solutions {
    stats {
      totalSolutions
      uniqueSolutions
      avgSolutionSize
      avgExecutionCost
      avgComplexityScore
      successRate
      mostCommonSolutionType
      totalExecutionTime
    }
  }
}
```

### 3. `mostUsed`

Get the most frequently used solutions.

**Arguments:**
- `page` (Int) - Page number (default: 1)
- `pageSize` (Int) - Items per page (default: 20)
- `sortBy` (String) - Sort by: "USAGE", "COST", "RECENT" (default: "USAGE")

**Returns:** `[CommonSolution!]!`

**Example:**
```graphql
query {
  solutions {
    mostUsed(page: 1, pageSize: 25, sortBy: "COST") {
      solutionHash
      solutionType
      usageCount
      avgExecutionCost
      firstUsed
      lastUsed
      successRate
      description
      associatedPuzzleTypes
    }
  }
}
```

### 4. `complexityAnalysis`

Analyze solution complexity patterns and execution efficiency.

**Arguments:** None

**Returns:** `SolutionComplexityAnalysis!`

**Example:**
```graphql
query {
  solutions {
    complexityAnalysis {
      avgComplexity
      medianComplexity
      maxComplexity
      complexityDistribution {
        complexityRange
        solutionCount
        percentageOfTotal
        avgExecutionCost
      }
      executionEfficiency {
        avgCostPerOperation
        mostEfficientSolutions
        leastEfficientSolutions
      }
    }
  }
}
```

### 5. `recent`

Get recently executed solutions.

**Arguments:**
- `limit` (Int) - Number of solutions to return (default: 20)
- `hoursBack` (Int) - Hours to look back (default: 24)

**Returns:** `[RecentSolution!]!`

**Example:**
```graphql
query {
  solutions {
    recent(limit: 50, hoursBack: 48) {
      solutionHash
      executedAt
      blockHeight
      executionCost
      wasSuccessful
      solutionType
      associatedPuzzleHash
      coinId
      errorMessage
    }
  }
}
```

## Type Definitions

### SolutionInfo

Detailed information about a specific solution.

```graphql
type SolutionInfo {
  # Solution hash identifier
  solutionHash: String!
  
  # Serialized solution data (hex)
  serializedSolution: String
  
  # Size of solution in bytes
  solutionSize: Int!
  
  # Complexity score (0-100)
  complexityScore: Float!
  
  # Execution cost in gas units
  executionCost: Int!
  
  # First block where solution was used
  firstUsedBlock: Int!
  
  # Most recent block where solution was used
  lastUsedBlock: Int!
  
  # Number of times this solution was used
  usageCount: Int!
  
  # Associated puzzle hash
  associatedPuzzleHash: String!
  
  # Classification of solution type
  solutionType: String!
  
  # Whether solution executed successfully
  isSuccessful: Boolean!
}
```

### SolutionStats

Overall solution usage statistics.

```graphql
type SolutionStats {
  # Total number of solution executions
  totalSolutions: Int!
  
  # Number of unique solution hashes
  uniqueSolutions: Int!
  
  # Average solution size in bytes
  avgSolutionSize: Float!
  
  # Average execution cost
  avgExecutionCost: Float!
  
  # Average complexity score
  avgComplexityScore: Float!
  
  # Percentage of successful executions
  successRate: Float!
  
  # Most common solution type
  mostCommonSolutionType: String!
  
  # Total cumulative execution time
  totalExecutionTime: Float!
}
```

### CommonSolution

Information about frequently used solutions.

```graphql
type CommonSolution {
  # Solution hash
  solutionHash: String!
  
  # Classified solution type
  solutionType: String!
  
  # Number of times used
  usageCount: Int!
  
  # Average execution cost
  avgExecutionCost: Float!
  
  # First usage block
  firstUsed: Int!
  
  # Most recent usage block
  lastUsed: Int!
  
  # Success rate percentage
  successRate: Float!
  
  # Human-readable description
  description: String!
  
  # Associated puzzle types
  associatedPuzzleTypes: [String!]!
}
```

### SolutionComplexityAnalysis

Analysis of solution complexity and efficiency.

```graphql
type SolutionComplexityAnalysis {
  # Average complexity across all solutions
  avgComplexity: Float!
  
  # Median complexity score
  medianComplexity: Float!
  
  # Maximum complexity observed
  maxComplexity: Float!
  
  # Distribution of complexity ranges
  complexityDistribution: [SolutionComplexityRange!]!
  
  # Execution efficiency statistics
  executionEfficiency: ExecutionEfficiencyStats!
}

type SolutionComplexityRange {
  # Complexity range (e.g., "0-10", "11-20")
  complexityRange: String!
  
  # Number of solutions in this range
  solutionCount: Int!
  
  # Percentage of total solutions
  percentageOfTotal: Float!
  
  # Average execution cost for this complexity
  avgExecutionCost: Float!
}

type ExecutionEfficiencyStats {
  # Average cost per operation
  avgCostPerOperation: Float!
  
  # Most efficient solutions
  mostEfficientSolutions: [String!]!
  
  # Least efficient solutions
  leastEfficientSolutions: [String!]!
}
```

### RecentSolution

Recently executed solution information.

```graphql
type RecentSolution {
  # Solution hash
  solutionHash: String!
  
  # Execution timestamp
  executedAt: String!
  
  # Block height of execution
  blockHeight: Int!
  
  # Execution cost incurred
  executionCost: Int!
  
  # Whether execution was successful
  wasSuccessful: Boolean!
  
  # Solution type classification
  solutionType: String!
  
  # Associated puzzle hash
  associatedPuzzleHash: String!
  
  # Coin ID that was spent
  coinId: String!
  
  # Error message if execution failed
  errorMessage: String
}
```

## Common Use Cases

### 1. Execution Cost Analysis

Analyze solution execution efficiency:

```graphql
query ExecutionAnalysis {
  solutions {
    stats {
      avgExecutionCost
      successRate
      totalExecutionTime
    }
    
    complexityAnalysis {
      executionEfficiency {
        avgCostPerOperation
        mostEfficientSolutions
        leastEfficientSolutions
      }
    }
  }
}
```

### 2. Solution Pattern Research

Understand common solution patterns:

```graphql
query SolutionPatterns {
  solutions {
    mostUsed(sortBy: "USAGE") {
      solutionType
      usageCount
      successRate
      description
      associatedPuzzleTypes
    }
    
    stats {
      mostCommonSolutionType
      uniqueSolutions
    }
  }
}
```

### 3. Debugging Failed Executions

Investigate failed solution executions:

```graphql
query FailureAnalysis {
  solutions {
    recent(limit: 100) {
      solutionHash
      wasSuccessful
      errorMessage
      executionCost
      solutionType
    }
  }
}
```

### 4. Performance Optimization

Identify optimization opportunities:

```graphql
query PerformanceOptimization {
  solutions {
    complexityAnalysis {
      complexityDistribution {
        complexityRange
        avgExecutionCost
        solutionCount
      }
    }
    
    mostUsed(sortBy: "COST") {
      solutionHash
      avgExecutionCost
      usageCount
      solutionType
    }
  }
}
```

## Solution Types

### Standard Solution Types
- **STANDARD_SPEND**: Basic coin spending
- **AGGREGATED_SIGNATURE**: Multi-signature solutions
- **DELEGATED_PUZZLE**: Puzzle delegation solutions
- **SINGLETON_SPEND**: NFT/singleton spending
- **CAT_SPEND**: CAT token transfers

### Custom Solution Categories
- **SIMPLE**: Basic custom solutions
- **COMPLEX**: Advanced smart contract solutions
- **BATCH**: Multi-operation solutions
- **CONDITIONAL**: Conditional execution solutions
- **UNKNOWN**: Unclassified solutions

## Execution Metrics

### Cost Categories
- **LOW**: < 1000 gas units
- **MEDIUM**: 1000-10000 gas units  
- **HIGH**: 10000-100000 gas units
- **EXTREME**: > 100000 gas units

### Complexity Scoring
- **0-20**: Simple solutions (basic operations)
- **21-40**: Moderate complexity
- **41-60**: Complex solutions
- **61-80**: Very complex solutions
- **81-100**: Extremely complex solutions

## Performance Analysis

### Efficiency Metrics
- **Gas per Operation**: Lower is better
- **Size Efficiency**: Gas per byte of solution
- **Success Rate**: Percentage of successful executions
- **Reuse Factor**: How often solutions are reused

### Optimization Opportunities
- **Pattern Recognition**: Identify common inefficient patterns
- **Cost Reduction**: Find cheaper alternatives to expensive solutions
- **Caching**: Reuse successful solutions where possible
- **Batching**: Combine multiple operations efficiently

## Error Analysis

### Common Failure Types
- **INVALID_SOLUTION**: Solution doesn't satisfy puzzle
- **EXECUTION_TIMEOUT**: Solution took too long to execute
- **INSUFFICIENT_GAS**: Not enough gas provided
- **MALFORMED_DATA**: Invalid solution format
- **LOGIC_ERROR**: Puzzle logic rejected solution

### Debugging Tools
- **Execution Traces**: Step-by-step execution analysis
- **Gas Profiling**: Detailed gas usage breakdown
- **Error Categorization**: Common failure pattern classification
- **Success Pattern Analysis**: What makes solutions work

## Security Considerations

### Risk Indicators
- **Unusual Complexity**: Solutions with very high complexity
- **High Failure Rate**: Solutions that frequently fail
- **Excessive Gas Usage**: Solutions consuming unusual amounts of gas
- **Pattern Anomalies**: Solutions deviating from normal patterns

### Best Practices
- Test solutions thoroughly before deployment
- Monitor execution costs and optimize where possible
- Use standard solution patterns when available
- Validate input data before solution execution

## Performance Considerations

1. **Solution Size**: Large solutions take more time to analyze
2. **Execution History**: Historical analysis is resource-intensive
3. **Complexity Calculations**: Complex solutions require more processing
4. **Real-time Data**: Recent solution data updates frequently

## Error Handling

Common errors:
- `SOLUTION_NOT_FOUND`: Solution hash doesn't exist
- `INVALID_HASH`: Invalid solution hash format
- `EXECUTION_DATA_UNAVAILABLE`: Execution details not available
- `ANALYSIS_TIMEOUT`: Analysis took too long

## Next Steps

- Explore [Puzzle Queries](puzzles.md) for related puzzle analysis
- Check [Transaction Queries](transactions.md) for transaction context
- See [Examples](../examples/index.md) for solution analysis patterns 