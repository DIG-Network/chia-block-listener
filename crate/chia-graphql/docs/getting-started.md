# Getting Started with Chia GraphQL API

## Installation

### Prerequisites

- Rust 1.70 or later
- Either PostgreSQL 12+ or SQLite 3.35+
- A synced Chia blockchain database (via `chia-block-database`)

### Adding to Your Project

Add the following to your `Cargo.toml`:

```toml
[dependencies]
chia-graphql = "0.1.0"
async-graphql = "7.0"
async-graphql-axum = "7.0"
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "sqlite"] }
tokio = { version = "1", features = ["full"] }
```

## Basic Setup

### 1. Database Connection

First, establish a connection to your blockchain database:

```rust
use chia_graphql::{ChiaGraphql, DatabaseConfig};
use sqlx::AnyPool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // For PostgreSQL
    let database_url = "postgresql://user:password@localhost/chia_blockchain";
    
    // For SQLite
    // let database_url = "sqlite:///path/to/chia_blockchain.db";
    
    let pool = AnyPool::connect(&database_url).await?;
    let graphql = ChiaGraphql::new(pool).await?;
    
    Ok(())
}
```

### 2. Running the GraphQL Server

Here's a complete example of setting up a GraphQL server with Axum:

```rust
use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQL};
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use chia_graphql::{QueryRoot, DatabaseType};
use sqlx::AnyPool;
use std::sync::Arc;

async fn graphql_playground() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/chia".to_string());
    
    let pool = Arc::new(AnyPool::connect(&database_url).await?);
    
    // Detect database type
    let db_type = if database_url.starts_with("postgresql://") {
        DatabaseType::Postgres
    } else {
        DatabaseType::Sqlite
    };
    
    // Create GraphQL schema
    let schema = Schema::build(
        QueryRoot::new(pool.clone(), db_type),
        EmptyMutation,
        EmptySubscription,
    )
    .finish();
    
    // Create router
    let app = Router::new()
        .route("/graphql", get(graphql_playground).post(GraphQL::new(schema)));
    
    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    println!("GraphQL playground available at http://localhost:8080/graphql");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

### 3. Running with Environment Variables

Create a `.env` file:

```bash
# For PostgreSQL
DATABASE_URL=postgresql://user:password@localhost/chia_blockchain

# For SQLite
# DATABASE_URL=sqlite:///path/to/chia_blockchain.db
```

Run the server:

```bash
cargo run --example server
```

## Your First Queries

Once the server is running, navigate to `http://localhost:8080/graphql` to access the GraphQL playground.

### Simple Queries

1. **Get current blockchain height:**

```graphql
query {
  currentHeight
}
```

2. **Get a specific coin:**

```graphql
query {
  coin(coinId: "0x1234567890abcdef...") {
    coinId
    amount
    puzzleHash
    parentCoinInfo
    createdBlock
  }
}
```

3. **Get a block by height:**

```graphql
query {
  block(height: 1000000) {
    height
    weight
    headerHash
    timestamp
  }
}
```

### Namespaced Queries

4. **Get network statistics:**

```graphql
query {
  network {
    networkStats {
      totalBlocks
      totalCoins
      totalAddresses
      totalValue
      currentHeight
      avgBlockTimeSeconds
    }
  }
}
```

5. **Get address balance:**

```graphql
query {
  core {
    balanceByPuzzleHash(puzzleHashHex: "0xabcd...") {
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

## Query Structure

The GraphQL API is organized into namespaces:

```graphql
query {
  # Direct queries
  currentHeight
  coin(coinId: "...")
  block(height: 123)
  
  # Namespaced queries
  core { ... }        # Basic blockchain data
  addresses { ... }   # Address analytics
  balances { ... }    # Balance analytics
  cats { ... }        # CAT token queries
  nfts { ... }        # NFT queries
  network { ... }     # Network statistics
  puzzles { ... }     # Puzzle analysis
  solutions { ... }   # Solution analysis
  temporal { ... }    # Time-based analytics
  transactions { ... } # Transaction analytics
  analytics { ... }   # Combined analytics
}
```

## Configuration Options

### Database Configuration

The API automatically detects the database type from the connection string:

- PostgreSQL: `postgresql://` or `postgres://`
- SQLite: `sqlite://` or file path

### Server Configuration

You can configure the server with environment variables:

```bash
# Server port (default: 8080)
PORT=8080

# Database URL (required)
DATABASE_URL=postgresql://localhost/chia

# Log level (default: info)
RUST_LOG=debug

# Maximum connections (default: 10)
DATABASE_MAX_CONNECTIONS=20
```

## Error Handling

The API returns structured errors:

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

Common error codes:
- `INVALID_INPUT` - Invalid parameters
- `NOT_FOUND` - Resource not found
- `DATABASE_ERROR` - Database operation failed
- `INTERNAL_ERROR` - Unexpected server error

## Performance Tips

1. **Use specific field selection** - Only request the fields you need
2. **Implement pagination** - Use `page` and `pageSize` for large datasets
3. **Batch queries** - Combine multiple queries in a single request
4. **Use fragments** - Reuse common field selections

Example of efficient querying:

```graphql
query EfficientQuery($page: Int!) {
  addresses {
    mostActiveAddresses(page: $page, pageSize: 20) {
      puzzleHashHex
      totalTransactions
      # Only request needed fields
    }
  }
}
```

## Next Steps

- Explore the [Schema Overview](schema-overview.md) to understand the API structure
- Check [Query Documentation](queries/core.md) for detailed query examples
- Review [Type Definitions](types/index.md) for complete type reference
- See [Query Examples](examples/index.md) for real-world use cases 