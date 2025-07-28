# Chia Full Indexer

A production-ready, high-performance blockchain indexer for the Chia network. This indexer provides comprehensive blockchain data ingestion, processing, and storage capabilities, following battle-tested patterns from real-world blockchain indexers.

## Features

### 🚀 **Dual-Mode Synchronization**
- **Historical Sync**: Automatically detects and fills gaps in blockchain data
- **Real-time Sync**: Listens for new blocks and processes them immediately
- **Intelligent Gap Detection**: Uses recursive CTEs to efficiently identify missing blocks
- **Concurrent Processing**: Historical and real-time sync run simultaneously

### 🔍 **Advanced Asset Processing**
- **CAT Token Support**: Automatically detects and indexes Chia Asset Tokens
- **NFT Collection Tracking**: Processes NFT collections with metadata fetching
- **Solution Analysis**: Analyzes CLVM solutions to extract asset information
- **Metadata Caching**: Intelligent caching of NFT/CAT metadata with HTTP fetching

### 🗄️ **Database Flexibility**
- **PostgreSQL**: Full-featured support with advanced SQL capabilities
- **SQLite**: Lightweight option for development and testing
- **Automatic Migrations**: Database schema versioning and migration system
- **Connection Pooling**: Efficient database connection management

### 🌐 **Peer Management**
- **DNS Discovery**: Automatic peer discovery using Chia DNS introducers
- **Connection Health**: Tracks peer reliability and performance
- **Failover Support**: Automatic peer replacement on connection failures
- **Geographic Distribution**: Supports IPv4 and IPv6 peers globally

### 📊 **Monitoring & Health**
- **Sync Watchdog**: Monitors sync health and detects stuck conditions
- **Performance Metrics**: Real-time performance and throughput monitoring
- **Progress Tracking**: Detailed sync progress with ETA calculations
- **Error Recovery**: Automatic retry logic with exponential backoff

### ⚙️ **Production Ready**
- **Graceful Shutdown**: Clean shutdown with resource cleanup
- **Pause/Resume**: Runtime control over sync operations
- **Configuration Management**: Environment variables and config file support
- **Structured Logging**: Comprehensive logging with configurable levels

## Architecture

The indexer follows a modular architecture inspired by production blockchain indexers:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Peer Manager  │    │  Block Listener │    │   Peer Pool     │
│                 │    │                 │    │                 │
│ • DNS Discovery │    │ • Real-time     │    │ • Historical    │
│ • Peer Health   │    │ • Event-driven  │    │ • Batch Download│
│ • Failover      │    │ • Block Events  │    │ • Load Balancing│
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   Sync Worker   │
                    │                 │
                    │ • Gap Detection │
                    │ • Block Queue   │
                    │ • Asset Processing
                    │ • Error Handling│
                    └─────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Solution Indexer│    │    Database     │    │    Watchdog     │
│                 │    │                 │    │                 │
│ • CAT Detection │    │ • CRUD Ops      │    │ • Health Check  │
│ • NFT Processing│    │ • Migrations    │    │ • Stuck Detection
│ • Metadata Cache│    │ • Connection Pool│    │ • Auto Recovery │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.70+ with Cargo
- Database (PostgreSQL 12+ or SQLite 3.35+)
- Network access to Chia introducers

### Installation

1. **Clone the repository:**
```bash
git clone https://github.com/your-org/chia-block-listener
cd chia-block-listener/crate/chia-full-indexer
```

2. **Set up environment:**
```bash
# For SQLite (development)
export INDEXER_DATABASE_CONNECTION_STRING="sqlite:./chia_blockchain.db"

# For PostgreSQL (production)
export INDEXER_DATABASE_CONNECTION_STRING="postgresql://user:password@localhost:5432/chia_blockchain"

# Set network and starting point
export INDEXER_BLOCKCHAIN_NETWORK_ID="mainnet"
export INDEXER_BLOCKCHAIN_START_HEIGHT="5000000"
```

3. **Run the indexer:**
```bash
cargo run --release
```

## Configuration

### Environment Variables

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `INDEXER_DATABASE_CONNECTION_STRING` | Database URL | `sqlite:./chia_blockchain.db` | `postgresql://user:pass@host/db` |
| `INDEXER_BLOCKCHAIN_NETWORK_ID` | Network to sync | `mainnet` | `mainnet`, `testnet11` |
| `INDEXER_BLOCKCHAIN_START_HEIGHT` | Starting block height | `1` | `5000000` |
| `INDEXER_BLOCKCHAIN_MAX_PEERS_HISTORICAL` | Historical sync peers | `3` | `5` |
| `INDEXER_BLOCKCHAIN_MAX_PEERS_REALTIME` | Real-time sync peers | `2` | `3` |
| `INDEXER_SYNC_GAP_SCAN_INTERVAL_SECS` | Gap scan frequency | `30` | `60` |
| `INDEXER_SYNC_BATCH_SIZE` | Blocks per batch | `100` | `500` |
| `INDEXER_WATCHDOG_TIMEOUT_MINUTES` | Watchdog timeout | `10` | `15` |
| `INDEXER_LOG_LEVEL` | Logging level | `info` | `debug`, `warn` |

### Configuration File

Create `indexer.toml`:

```toml
[database]
connection_string = "postgresql://chia:secure_password@localhost:5432/chia_blockchain"
max_connections = 20
connection_timeout_secs = 30

[blockchain]
start_height = 5000000
network_id = "mainnet"
max_peers_historical = 5
max_peers_realtime = 3

[network]
introducers = [
    "introducer.chia.net",
    "introducer-or.chia.net", 
    "introducer-eu.chia.net"
]
default_port = 8444
connection_timeout_secs = 30

[sync]
gap_scan_interval_secs = 30
status_update_interval_secs = 10
batch_size = 200
max_concurrent_downloads = 10
enable_balance_refresh = true

[watchdog]
timeout_minutes = 15
check_interval_secs = 60
max_stuck_time_secs = 900
max_no_progress_checks = 5
```

## Database Setup

### PostgreSQL (Recommended for Production)

```sql
-- Create database and user
CREATE DATABASE chia_blockchain;
CREATE USER chia_indexer WITH PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE chia_blockchain TO chia_indexer;

-- Connect to the database
\c chia_blockchain

-- Grant schema permissions
GRANT ALL ON SCHEMA public TO chia_indexer;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO chia_indexer;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO chia_indexer;
```

### SQLite (Development)

SQLite databases are created automatically:

```bash
export INDEXER_DATABASE_CONNECTION_STRING="sqlite:./chia_blockchain.db"
```

## Usage Examples

### Basic Usage

```bash
# Start with default settings
cargo run --release

# Start from specific height
INDEXER_BLOCKCHAIN_START_HEIGHT=5000000 cargo run --release

# Debug mode with verbose logging
INDEXER_LOG_LEVEL=debug cargo run --release
```

### Production Deployment

```bash
# Set production configuration
export INDEXER_DATABASE_CONNECTION_STRING="postgresql://chia:$DB_PASSWORD@db.example.com:5432/chia_blockchain"
export INDEXER_BLOCKCHAIN_NETWORK_ID="mainnet"
export INDEXER_BLOCKCHAIN_START_HEIGHT="5000000"
export INDEXER_LOG_LEVEL="info"

# Run with systemd or process manager
cargo build --release
./target/release/chia-full-indexer
```

### Docker Deployment

```dockerfile
FROM rust:1.70-alpine AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin chia-full-indexer

FROM alpine:latest
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/chia-full-indexer /usr/local/bin/
ENTRYPOINT ["chia-full-indexer"]
```

## Monitoring

### Sync Status

The indexer provides comprehensive status information:

```bash
# Status is logged every 30 seconds
INFO Sync Status Report:
INFO    Height: 5,125,432 / 5,125,500 (99.86%)
INFO    ETA: 2 minutes
INFO    Speed: 125.34 blocks/sec
INFO    Session: 125,432 blocks processed
```

### Performance Metrics

Monitor key performance indicators:

- **Blocks per second**: Processing throughput
- **Gap detection efficiency**: Time to identify missing blocks
- **Peer connection health**: Active vs. failed connections
- **Database performance**: Query execution times
- **Memory usage**: Resource consumption

### Health Checks

The watchdog monitors:

- **Sync progress**: Detects when sync gets stuck
- **Peer connectivity**: Monitors peer connection health
- **Database responsiveness**: Ensures database connectivity
- **Memory usage**: Prevents memory leaks
- **Error rates**: Tracks error frequency

## API Integration

### Database Schema

The indexer creates a comprehensive database schema:

```sql
-- Core blockchain data
blocks (height, weight, header_hash, timestamp)
coins (coin_id, parent_coin_info, puzzle_hash, amount, created_block)
spends (coin_id, puzzle_id, solution_id, spent_block)

-- Asset tracking
cat_assets (asset_id, symbol, name, total_supply, decimals)
cats (coin_id, asset_id, owner_puzzle_hash, amount)
nft_collections (collection_id, name, creator_puzzle_hash)
nfts (coin_id, launcher_id, collection_id, owner_puzzle_hash)

-- Materialized views for performance
xch_balances (puzzle_hash_hex, total_mojos, coin_count)
cat_balances (asset_id, owner_puzzle_hash_hex, total_amount)

-- System management
sync_status (sync_type, last_synced_height, current_peak_height)
system_preferences (meta_key, meta_value, updated_at)
```

### GraphQL Integration

Use with the included GraphQL server:

```rust
use chia_graphql::ChiaGraphQLServer;
use chia_block_database::ChiaBlockDatabase;

// Initialize database connection
let database = ChiaBlockDatabase::new(&database_url).await?;

// Start GraphQL server
let server = ChiaGraphQLServer::new(database);
server.start("0.0.0.0:4000").await?;
```

## Troubleshooting

### Common Issues

**1. Database Connection Errors**
```bash
# Check connection string format
postgresql://user:password@host:port/database
sqlite:./path/to/database.db

# Verify database permissions
GRANT ALL PRIVILEGES ON DATABASE chia_blockchain TO indexer_user;
```

**2. Peer Connection Issues**
```bash
# Check network connectivity
ping introducer.chia.net

# Verify firewall rules
sudo ufw allow 8444/tcp

# Test DNS resolution
nslookup introducer.chia.net
```

**3. Sync Getting Stuck**
```bash
# Check watchdog logs for specific issues
# Common causes:
# - Network connectivity issues
# - Database performance problems
# - Insufficient disk space
# - Memory constraints
```

**4. High Memory Usage**
```bash
# Reduce batch size
export INDEXER_SYNC_BATCH_SIZE=50

# Reduce concurrent downloads
export INDEXER_SYNC_MAX_CONCURRENT_DOWNLOADS=3

# Clear metadata cache periodically
# (automatic in production)
```

### Performance Tuning

**Database Optimization:**
```sql
-- PostgreSQL performance tuning
shared_buffers = 256MB
effective_cache_size = 1GB
work_mem = 16MB
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.9
```

**System Resources:**
- **RAM**: Minimum 4GB, recommended 8GB+
- **Storage**: SSD recommended, 100GB+ free space
- **CPU**: 4+ cores for optimal performance
- **Network**: Stable internet connection

## Development

### Building from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/your-org/chia-block-listener
cd chia-block-listener/crate/chia-full-indexer
cargo build --release
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests with database
DATABASE_URL=sqlite::memory: cargo test

# Performance benchmarks
cargo bench
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) for details.

## Support

- **Documentation**: [docs.rs/chia-full-indexer](https://docs.rs/chia-full-indexer)
- **Issues**: [GitHub Issues](https://github.com/your-org/chia-block-listener/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/chia-block-listener/discussions)

## Acknowledgments

This indexer is built on top of:
- [chia-protocol](https://crates.io/crates/chia-protocol) - Chia protocol implementation
- [sqlx](https://crates.io/crates/sqlx) - Async SQL toolkit
- [tokio](https://crates.io/crates/tokio) - Async runtime
- [tracing](https://crates.io/crates/tracing) - Structured logging

Inspired by production blockchain indexers and the [insight](cloud2/terraform/application/insight/) reference implementation.

