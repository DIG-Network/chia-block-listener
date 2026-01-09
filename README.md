# Chia Block Listener

A high-performance Chia blockchain listener for Node.js, built with Rust and NAPI bindings. This library provides real-time monitoring of the Chia blockchain with efficient peer connections and block parsing capabilities.

## Features

- **Real-time Block Monitoring**: Listen for new blocks as they're produced on the Chia network
- **Peer Management**: Connect to multiple Chia full nodes simultaneously
- **Automatic Failover**: Intelligent peer failover with automatic retry across multiple peers
- **Enhanced Error Handling**: Automatic disconnection of peers that refuse blocks or have protocol errors
- **Efficient Parsing**: Fast extraction of coin spends, additions, and removals from blocks
- **Event-Driven Architecture**: TypeScript-friendly event system with full type safety
- **Transaction Analysis**: Parse CLVM puzzles and solutions from coin spends
- **Historical Block Access**: Retrieve blocks by height or ranges with automatic load balancing
- **Connection Pool**: ChiaPeerPool provides automatic load balancing and rate limiting for historical queries
- **Peak Height Tracking**: Monitor blockchain sync progress across all connected peers
- **DNS Peer Discovery**: Automatic peer discovery using Chia network DNS introducers with IPv4/IPv6 support
- **Cross-platform Support**: Works on Windows, macOS, and Linux (x64 and ARM64)
- **TypeScript Support**: Complete TypeScript definitions with IntelliSense

## Installation

```bash
npm install @dignetwork/chia-block-listener
```

## Quick Start

```javascript
const { ChiaBlockListener, initTracing } = require('@dignetwork/chia-block-listener')

// Initialize tracing for debugging (optional)
initTracing()

// Create a new listener instance
const listener = new ChiaBlockListener()

// Listen for block events
listener.on('blockReceived', (block) => {
  console.log(`New block received: ${block.height}`)
  console.log(`Header hash: ${block.headerHash}`)
  console.log(`Timestamp: ${new Date(block.timestamp * 1000)}`)
  console.log(`Coin additions: ${block.coinAdditions.length}`)
  console.log(`Coin removals: ${block.coinRemovals.length}`)
  console.log(`Coin spends: ${block.coinSpends.length}`)
})

// Listen for peer connection events
listener.on('peerConnected', (peer) => {
  console.log(`Connected to peer: ${peer.peerId} (${peer.host}:${peer.port})`)
})

listener.on('peerDisconnected', (peer) => {
  console.log(`Disconnected from peer: ${peer.peerId}`)
  if (peer.message) {
    console.log(`Reason: ${peer.message}`)
  }
})

// Connect to a Chia full node
const peerId = listener.addPeer('localhost', 8444, 'mainnet')
console.log(`Added peer: ${peerId}`)

// Keep the process running
process.on('SIGINT', () => {
  console.log('Shutting down...')
  listener.disconnectAllPeers()
  process.exit(0)
})
```

## Rust usage (canonical, Rust-first interface)

This repository now provides a pure Rust API suitable for use in Tokio-based applications, while the N-API adapter remains a thin layer for Node.js. The Rust-facing entry point is `BlockListener` (previously named `Listener`).

Key points of the Rust API:
- Your application owns the policy/event loop. The library only manages networking, peers, and event emission.
- Subscribe to events via a bounded broadcast channel. Delivery is best-effort; slow consumers may miss messages, which is appropriate for catch-up/skip logic.
- Structured shutdown: signal cancellation quickly and optionally wait for all internal tasks to complete.

Example:

```rust
use chia_block_listener::{init_tracing, BlockListener, ListenerConfig};
use chia_block_listener::types::Event;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Optional: enable logging
    init_tracing();

    // Configure and create the block listener (buffer defaults to 1024)
    let block_listener = BlockListener::new(ListenerConfig::default())?;

    // Subscribe to events (bounded, best-effort delivery)
    let rx = block_listener.subscribe();
    let mut events = BroadcastStream::new(rx);

    // Connect a peer
    let _peer_id = block_listener.add_peer("localhost".into(), 8444, "mainnet".into()).await?;

    // Application-owned policy loop: read events and act
    let policy = tokio::spawn(async move {
        while let Some(item) = events.next().await {
            match item {
                Ok(Event::PeerConnected(e)) => {
                    println!("connected: {} ({}:{})", e.peer_id, e.host, e.port);
                }
                Ok(Event::PeerDisconnected(e)) => {
                    println!("disconnected: {} reason={:?}", e.peer_id, e.message);
                }
                Ok(Event::NewPeakHeight(e)) => {
                    println!("new peak: old={:?} new={} from {}", e.old_peak, e.new_peak, e.peer_id);
                }
                Ok(Event::BlockReceived(b)) => {
                    println!("block {} {}", b.height, b.header_hash);
                }
                Err(_lagged) => {
                    // One or more messages were missed (slow consumer). Best-effort model: recompute current state if needed.
                    // For catch-up use cases, you generally only need latest peak or most recent block.
                }
            }
        }
    });

    // Shutdown example (e.g., on SIGINT/SIGTERM):
    // Signal fast
    block_listener.shutdown().await?;
    // Wait for all internal tasks to end deterministically
    block_listener.shutdown_and_wait().await?;

    policy.await?;
    Ok(())
}
```

### Backpressure and delivery guarantees
- The core guarantees that parsed blocks are submitted into the internal event pipeline (no pre-broadcast drop). Blocks are enqueued with backpressure inside the pool and forwarded to the broadcast dispatcher.
- Event broadcast to subscribers uses a bounded buffer (default 1024). If a subscriber is slow, it may receive `Lagged(n)` from the stream wrapper, meaning it missed `n` events. This affects slow subscribers only and does not cause block events to be dropped before entering the broadcast.
- Informational events (peerConnected, peerDisconnected, newPeakHeight) are best-effort before broadcast and may be dropped under overload to avoid stalling networking.
- Recommended approach for “catch-up” logic: maintain your own application state keyed by height/epoch and cancel/work-skip as newer events arrive. If you need every block for historical processing, use explicit queries like `get_block_by_height`/ranges in addition to the live stream.

### Passive WebSocket listening and multi-peer behavior
- **Listener, not poller:** Each `add_peer` establishes a dedicated streaming WebSocket reader (per peer) that passively consumes protocol messages and drives events. `NewPeakWallet` triggers a `RequestBlock` immediately; `RespondBlock` is parsed and emitted as `Event::BlockReceived` via the core event pipeline.
- **Unified event pipeline:** All live events (`PeerConnected`, `PeerDisconnected`, `NewPeakHeight`, `BlockReceived`) flow through a single core mpsc sink and are forwarded to all subscribers via `broadcast`. Blocks use `send().await` into the sink (no pre-broadcast drop); informational events are best-effort `try_send` to avoid stalling I/O.
- **Multiple peers:** You can call `add_peer` multiple times. The pool tracks each peer independently with its own streaming reader and request worker. On-demand requests (`get_block_by_height`) use round-robin with cooldown and remove unhealthy peers based on repeated failures/timeouts/protocol errors.
- **Consumption pattern:** Your app owns the policy loop—subscribe once, then react to events as they arrive. Slow subscribers may receive `Lagged(n)` from the broadcast wrapper; recompute state as needed. For guaranteed historical coverage, pair live listening with explicit `get_block_by_height` / range queries.
- **Shutdown:** `shutdown()` signals cancellation; `shutdown_and_wait()` awaits the dispatcher and all tracked peer/request tasks for deterministic teardown. `Drop` on `Listener` signals cancel without awaiting (non-blocking drop safety).

### Shutdown semantics
- `shutdown()` signals cancellation and returns quickly.
- `shutdown_and_wait()` cancels and then awaits all internal tasks to finish (peer workers, request processor, dispatchers), providing deterministic shutdown.

### Configuration
- `ListenerConfig` currently exposes the event buffer size. Additional knobs (timeouts, rate limits, etc.) are centralized in the core with sensible defaults and named constants.
## API Reference

### ChiaBlockListener Class

#### Constructor

```javascript
const listener = new ChiaBlockListener()
```

Creates a new Chia block listener instance.

#### Methods

##### `addPeer(host, port, networkId): string`

Connects to a Chia full node and starts listening for blocks.

**Parameters:**
- `host` (string): The hostname or IP address of the Chia node
- `port` (number): The port number (typically 8444 for mainnet)
- `networkId` (string): The network identifier ('mainnet', 'testnet', etc.)

**Returns:** A unique peer ID string for this connection

##### `disconnectPeer(peerId): boolean`

Disconnects from a specific peer.

**Parameters:**
- `peerId` (string): The peer ID returned by `addPeer()`

**Returns:** `true` if the peer was successfully disconnected, `false` otherwise

##### `disconnectAllPeers(): void`

Disconnects from all connected peers.

##### `getConnectedPeers(): string[]`

Returns an array of currently connected peer IDs.

##### `getBlockByHeight(peerId, height): BlockReceivedEvent`

Retrieves a specific block by its height from a connected peer.

**Parameters:**
- `peerId` (string): The peer ID to query
- `height` (number): The block height to retrieve

**Returns:** A `BlockReceivedEvent` object containing the block data

##### `getBlocksRange(peerId, startHeight, endHeight): BlockReceivedEvent[]`

Retrieves a range of blocks from a connected peer.

**Parameters:**
- `peerId` (string): The peer ID to query
- `startHeight` (number): The starting block height (inclusive)
- `endHeight` (number): The ending block height (inclusive)

**Returns:** An array of `BlockReceivedEvent` objects

### ChiaPeerPool Class

The `ChiaPeerPool` provides a managed pool of peer connections for retrieving historical blocks with automatic load balancing and intelligent failover across multiple peers. When a peer fails to provide a block or experiences protocol errors, the pool automatically tries alternative peers and removes problematic peers from the pool.

#### Constructor

```javascript
const pool = new ChiaPeerPool()
```

Creates a new peer pool instance with built-in rate limiting (500ms per peer).

#### Methods

##### `addPeer(host, port, networkId): Promise<string>`

Adds a peer to the connection pool.

**Parameters:**
- `host` (string): The hostname or IP address of the Chia node
- `port` (number): The port number (typically 8444 for mainnet)
- `networkId` (string): The network identifier ('mainnet', 'testnet', etc.)

**Returns:** A Promise that resolves to a unique peer ID string

##### `getBlockByHeight(height): Promise<BlockReceivedEvent>`

Retrieves a specific block by height using automatic peer selection and load balancing.

**Parameters:**
- `height` (number): The block height to retrieve

**Returns:** A Promise that resolves to a `BlockReceivedEvent` object

##### `removePeer(peerId): Promise<boolean>`

Removes a peer from the pool.

**Parameters:**
- `peerId` (string): The peer ID to remove

**Returns:** A Promise that resolves to `true` if the peer was removed, `false` otherwise

##### `shutdown(): Promise<void>`

Shuts down the pool and disconnects all peers.

##### `getConnectedPeers(): Promise<string[]>`

Gets the list of currently connected peer IDs.

**Returns:** Array of peer ID strings (format: "host:port")

##### `getPeakHeight(): Promise<number | null>`

Gets the highest blockchain peak height seen across all connected peers.

**Returns:** The highest peak height as a number, or null if no peaks have been received yet

##### `on(event, callback): void`

Registers an event handler for pool events.

**Parameters:**
- `event` (string): The event name ('peerConnected' or 'peerDisconnected')
- `callback` (function): The event handler function

##### `off(event, callback): void`

Removes an event handler.

**Parameters:**
- `event` (string): The event name to stop listening for

### Events

The `ChiaBlockListener` emits the following events:

#### `blockReceived`

Fired when a new block is received from any connected peer.

**Callback:** `(event: BlockReceivedEvent) => void`

#### `peerConnected`

Fired when a connection to a peer is established.

**Callback:** `(event: PeerConnectedEvent) => void`

#### `peerDisconnected`

Fired when a peer connection is lost.

**Callback:** `(event: PeerDisconnectedEvent) => void`

### ChiaPeerPool Events

The `ChiaPeerPool` emits the following events:

#### `peerConnected`

Fired when a peer is successfully added to the pool.

**Callback:** `(event: PeerConnectedEvent) => void`

#### `peerDisconnected`

Fired when a peer is removed from the pool or disconnects.

**Callback:** `(event: PeerDisconnectedEvent) => void`

#### `newPeakHeight`

Fired when a new highest blockchain peak is discovered.

**Callback:** `(event: NewPeakHeightEvent) => void`

### DnsDiscoveryClient Class

The `DnsDiscoveryClient` provides automatic peer discovery using Chia network DNS introducers with full IPv4 and IPv6 support.

#### Constructor

```javascript
const client = new DnsDiscoveryClient()
```

Creates a new DNS discovery client instance.

#### Methods

##### `discoverMainnetPeers(): Promise<DiscoveryResultJS>`

Discovers peers for Chia mainnet using built-in DNS introducers.

**Returns:** Promise resolving to discovery results with separate IPv4 and IPv6 peer lists

##### `discoverTestnet11Peers(): Promise<DiscoveryResultJS>`

Discovers peers for Chia testnet11 using built-in DNS introducers.

**Returns:** Promise resolving to discovery results

##### `discoverPeers(introducers, port): Promise<DiscoveryResultJS>`

Discovers peers using custom DNS introducers.

**Parameters:**
- `introducers` (string[]): Array of DNS introducer hostnames
- `port` (number): Default port for discovered peers

**Returns:** Promise resolving to discovery results

##### `resolveIpv4(hostname): Promise<AddressResult>`

Resolves IPv4 addresses (A records) for a hostname.

**Parameters:**
- `hostname` (string): Hostname to resolve

**Returns:** Promise resolving to IPv4 addresses

##### `resolveIpv6(hostname): Promise<AddressResult>`

Resolves IPv6 addresses (AAAA records) for a hostname.

**Parameters:**
- `hostname` (string): Hostname to resolve

**Returns:** Promise resolving to IPv6 addresses

##### `resolveBoth(hostname, port): Promise<DiscoveryResultJS>`

Resolves both IPv4 and IPv6 addresses for a hostname.

**Parameters:**
- `hostname` (string): Hostname to resolve
- `port` (number): Port for the peer addresses

**Returns:** Promise resolving to discovery results

### Event Data Types

#### `BlockReceivedEvent`

```typescript
interface BlockReceivedEvent {
  peerId: string                    // IP address of the peer that sent this block
  height: number                     // Block height
  weight: string                     // Block weight as string
  headerHash: string               // Block header hash (hex)
  timestamp: number                 // Block timestamp (Unix time)
  coinAdditions: CoinRecord[]      // New coins created in this block
  coinRemovals: CoinRecord[]       // Coins spent in this block
  coinSpends: CoinSpend[]         // Detailed spend information
  coinCreations: CoinRecord[]      // Coins created by puzzles
  hasTransactionsGenerator: boolean // Whether block has a generator
  generatorSize: number            // Size of the generator bytecode
}
```

#### `PeerConnectedEvent`

```typescript
interface PeerConnectedEvent {
  peerId: string  // Peer IP address
  host: string     // Peer hostname/IP
  port: number     // Peer port number
}
```

#### `PeerDisconnectedEvent`

```typescript
interface PeerDisconnectedEvent {
  peerId: string   // Peer IP address
  host: string      // Peer hostname/IP
  port: number      // Peer port number
  message?: string  // Optional disconnection reason
}
```

#### `NewPeakHeightEvent`

```typescript
interface NewPeakHeightEvent {
  oldPeak: number | null  // Previous highest peak (null if first peak)
  newPeak: number        // New highest peak height
  peerId: string         // Peer that discovered this peak
}
```

#### `CoinRecord`

```typescript
interface CoinRecord {
  parentCoinInfo: string  // Parent coin ID (hex)
  puzzleHash: string       // Puzzle hash (hex)
  amount: string            // Coin amount as string
}
```

#### `CoinSpend`

```typescript
interface CoinSpend {
  coin: CoinRecord         // The coin being spent
  puzzleReveal: string    // CLVM puzzle bytecode (hex)
  solution: string         // CLVM solution bytecode (hex)
  offset: number           // Offset in the generator bytecode
}
```

#### `DiscoveryResultJS`

```typescript
interface DiscoveryResultJS {
  ipv4Peers: PeerAddressJS[]    // IPv4 peer addresses
  ipv6Peers: PeerAddressJS[]    // IPv6 peer addresses  
  totalCount: number            // Total peers found
}
```

#### `PeerAddressJS`

```typescript
interface PeerAddressJS {
  host: string           // IP address as string
  port: number           // Port number
  isIpv6: boolean        // Protocol indicator
  displayAddress: string // Formatted for display/URLs
}
```

#### `AddressResult`

```typescript
interface AddressResult {
  addresses: string[]    // List of IP addresses
  count: number          // Number of addresses
}
```

## ChiaPeerPool Usage

The `ChiaPeerPool` is designed for efficiently retrieving historical blocks with automatic load balancing and intelligent failover across multiple peers. When a peer fails to provide a block or experiences protocol errors, the pool automatically tries alternative peers and removes problematic peers from the pool.

### Basic Usage

```javascript
const { ChiaPeerPool, initTracing } = require('@dignetwork/chia-block-listener')

async function main() {
  // Initialize tracing
  initTracing()
  
  // Create a peer pool
  const pool = new ChiaPeerPool()
  
  // Listen for pool events
  pool.on('peerConnected', (event) => {
    console.log(`Peer connected to pool: ${event.peerId}`)
  })
  
  pool.on('peerDisconnected', (event) => {
    console.log(`Peer disconnected from pool: ${event.peerId}`)
  })
  
  pool.on('newPeakHeight', (event) => {
    console.log(`New blockchain peak detected!`)
    console.log(`  Previous: ${event.oldPeak || 'None'}`)
    console.log(`  New: ${event.newPeak}`)
    console.log(`  Discovered by: ${event.peerId}`)
  })
  
  // Add multiple peers
  await pool.addPeer('node1.chia.net', 8444, 'mainnet')
  await pool.addPeer('node2.chia.net', 8444, 'mainnet')
  await pool.addPeer('node3.chia.net', 8444, 'mainnet')
  
  // Fetch blocks with automatic load balancing
  const block1 = await pool.getBlockByHeight(5000000)
  const block2 = await pool.getBlockByHeight(5000001)
  const block3 = await pool.getBlockByHeight(5000002)
  
  console.log(`Block ${block1.height}: ${block1.coinSpends.length} spends`)
  console.log(`Block ${block2.height}: ${block2.coinSpends.length} spends`)
  console.log(`Block ${block3.height}: ${block3.coinSpends.length} spends`)
  
  // Shutdown the pool
  await pool.shutdown()
}

main().catch(console.error)
```

### Advanced Pool Features

#### Rate Limiting

The pool automatically enforces a 500ms rate limit per peer for maximum performance while preventing node overload:

```javascript
// Rapid requests are automatically queued and distributed
const promises = []
for (let i = 5000000; i < 5000100; i++) {
  promises.push(pool.getBlockByHeight(i))
}

// All requests will be processed efficiently across all peers
// with automatic load balancing and rate limiting
const blocks = await Promise.all(promises)
console.log(`Retrieved ${blocks.length} blocks`)
```

#### Automatic Failover and Error Handling

The pool provides robust error handling with automatic failover:

```javascript
// The pool automatically handles various error scenarios:

// 1. Connection failures - automatically tries other peers
try {
  const block = await pool.getBlockByHeight(5000000)
  console.log(`Retrieved block ${block.height}`)
} catch (error) {
  // If all peers fail, you'll get an error after all retry attempts
  console.error('All peers failed:', error.message)
}

// 2. Protocol errors - peers that refuse blocks are automatically disconnected
pool.on('peerDisconnected', (event) => {
  console.log(`Peer ${event.peerId} disconnected: ${event.reason}`)
  // Reasons include: "Block request rejected", "Protocol error", "Connection timeout"
})

// 3. Automatic peer cleanup - problematic peers are removed from the pool
console.log('Active peers before:', await pool.getConnectedPeers())
await pool.getBlockByHeight(5000000) // May trigger peer removal
console.log('Active peers after:', await pool.getConnectedPeers())

// 4. Multiple retry attempts - tries up to 3 different peers per request
// This happens automatically and transparently
const block = await pool.getBlockByHeight(5000000) // Will try multiple peers if needed
```

**Error Types Handled Automatically:**
- **Connection Errors**: Timeouts, network failures, WebSocket errors
- **Protocol Errors**: Block rejections, parsing failures, handshake failures
- **Peer Misbehavior**: Unexpected responses, invalid data formats

#### Dynamic Peer Management

```javascript
// Monitor pool health
const peers = await pool.getConnectedPeers()
console.log(`Active peers in pool: ${peers.length}`)

// Remove underperforming peers
if (slowPeer) {
  await pool.removePeer(slowPeer)
  console.log('Removed slow peer from pool')
}

// Add new peers dynamically
if (peers.length < 3) {
  await pool.addPeer('backup-node.chia.net', 8444, 'mainnet')
}
```

#### Error Handling

```javascript
try {
  const block = await pool.getBlockByHeight(5000000)
  console.log(`Retrieved block ${block.height}`)
} catch (error) {
  console.error('Failed to retrieve block:', error)
  
  // The pool will automatically try other peers
  // You can also add more peers if needed
  const peers = await pool.getConnectedPeers()
  if (peers.length === 0) {
    console.log('No peers available, adding new ones...')
    await pool.addPeer('node1.chia.net', 8444, 'mainnet')
  }
}
```

#### Peak Height Tracking

```javascript
// Monitor blockchain sync progress
const pool = new ChiaPeerPool()

// Track peak changes
let currentPeak = null
pool.on('newPeakHeight', (event) => {
  currentPeak = event.newPeak
  const progress = event.oldPeak 
    ? `+${event.newPeak - event.oldPeak} blocks` 
    : 'Initial peak'
  console.log(`Peak update: ${event.newPeak} (${progress})`)
})

// Add peers
await pool.addPeer('node1.chia.net', 8444, 'mainnet')
await pool.addPeer('node2.chia.net', 8444, 'mainnet')

// Check current peak
const peak = await pool.getPeakHeight()
console.log(`Current highest peak: ${peak || 'None yet'}`)

// Fetch some blocks to trigger peak updates
await pool.getBlockByHeight(5000000)
await pool.getBlockByHeight(5100000)
await pool.getBlockByHeight(5200000)

// Monitor sync status
setInterval(async () => {
  const peak = await pool.getPeakHeight()
  if (peak) {
    const estimatedCurrent = 5200000 + Math.floor((Date.now() / 1000 - 1700000000) / 18.75)
    const syncPercentage = (peak / estimatedCurrent * 100).toFixed(2)
    console.log(`Sync status: ${syncPercentage}% (peak: ${peak})`)
  }
}, 60000) // Check every minute
```

### When to Use ChiaPeerPool vs ChiaBlockListener

- **Use ChiaPeerPool when:**
  - You need to fetch historical blocks
  - You want automatic load balancing across multiple peers
  - You're making many block requests and need rate limiting
  - You don't need real-time block notifications

- **Use ChiaBlockListener when:**
  - You need real-time notifications of new blocks
  - You want to monitor the blockchain as it grows
  - You need to track specific addresses or puzzle hashes in real-time
  - You're building applications that react to blockchain events

Both classes can be used together in the same application for different purposes.

## DNS Discovery Usage

The `DnsDiscoveryClient` enables automatic discovery of Chia network peers using DNS introducers, with full support for both IPv4 and IPv6 addresses.

### Basic DNS Discovery

```javascript
const { DnsDiscoveryClient, initTracing } = require('@dignetwork/chia-block-listener')

async function discoverPeers() {
  // Initialize tracing
  initTracing()
  
  // Create DNS discovery client
  const client = new DnsDiscoveryClient()
  
  // Discover mainnet peers
  const result = await client.discoverMainnetPeers()
  
  console.log(`Found ${result.totalCount} total peers:`)
  console.log(`  IPv4 peers: ${result.ipv4Peers.length}`)
  console.log(`  IPv6 peers: ${result.ipv6Peers.length}`)
  
  // Use with peer connections
  for (const peer of result.ipv4Peers.slice(0, 3)) {
    console.log(`IPv4 peer: ${peer.displayAddress}`)
    // peer.host and peer.port can be used with addPeer()
  }
  
  for (const peer of result.ipv6Peers.slice(0, 3)) {
    console.log(`IPv6 peer: ${peer.displayAddress}`) // [2001:db8::1]:8444
    // IPv6 addresses are properly formatted with brackets
  }
}

discoverPeers().catch(console.error)
```

### Integration with Peer Pool

```javascript
const { ChiaPeerPool, DnsDiscoveryClient } = require('@dignetwork/chia-block-listener')

async function setupPoolWithDnsDiscovery() {
  const pool = new ChiaPeerPool()
  const discovery = new DnsDiscoveryClient()
  
  // Discover peers automatically
  const peers = await discovery.discoverMainnetPeers()
  
  // Add discovered peers to pool (both IPv4 and IPv6)
  const allPeers = [...peers.ipv4Peers, ...peers.ipv6Peers]
  for (const peer of allPeers.slice(0, 5)) {
    await pool.addPeer(peer.host, peer.port, 'mainnet')
    console.log(`Added peer: ${peer.displayAddress}`)
  }
  
  // Now use the pool for block retrieval
  const block = await pool.getBlockByHeight(5000000)
  console.log(`Retrieved block ${block.height}`)
  
  await pool.shutdown()
}

setupPoolWithDnsDiscovery().catch(console.error)
```

### Custom DNS Introducers

```javascript
const client = new DnsDiscoveryClient()

// Use custom introducers
const customIntroducers = [
  'seeder.dexie.space',
  'chia.hoffmang.com'
]

const result = await client.discoverPeers(customIntroducers, 8444)
console.log(`Found ${result.totalCount} peers from custom introducers`)
```

### Individual DNS Resolution

```javascript
const client = new DnsDiscoveryClient()
const hostname = 'dns-introducer.chia.net'

// Resolve specific protocols
try {
  const ipv4 = await client.resolveIpv4(hostname)
  console.log(`IPv4 addresses: ${ipv4.addresses.join(', ')}`)
} catch (error) {
  console.log(`IPv4 resolution failed: ${error.message}`)
}

try {
  const ipv6 = await client.resolveIpv6(hostname)
  console.log(`IPv6 addresses: ${ipv6.addresses.join(', ')}`)
} catch (error) {
  console.log(`IPv6 resolution failed: ${error.message}`)
}

// Or resolve both at once
const both = await client.resolveBoth(hostname, 8444)
console.log(`Combined: ${both.totalCount} addresses`)
```



### Error Handling

```javascript
const client = new DnsDiscoveryClient()

try {
  const result = await client.discoverMainnetPeers()
  console.log(`Discovery successful: ${result.totalCount} peers`)
} catch (error) {
  console.error('Discovery failed:', error.message)
  
  // Handle different error types
  if (error.message.includes('NoPeersFound')) {
    console.log('No peers found from any introducer')
  } else if (error.message.includes('ResolutionFailed')) {
    console.log('DNS resolution failed')
  }
}
```

### Key Features

- **Dual Stack Support**: Separate IPv4 and IPv6 peer lists
- **Proper DNS Lookups**: Uses A records for IPv4, AAAA records for IPv6
- **Built-in Networks**: Ready configurations for mainnet and testnet11
- **Custom Introducers**: Support for any DNS introducers
- **IPv6 URL Formatting**: Automatic bracket formatting for IPv6 addresses
- **Type Safety**: Full TypeScript support with detailed type definitions

## ChiaBlockParser Usage

The `ChiaBlockParser` provides direct access to the Rust-based block parsing engine, enabling efficient parsing of Chia FullBlock data with complete control over the parsing process. This is ideal for applications that need to process block data from external sources or implement custom block analysis.

### Basic Block Parsing

```javascript
const { ChiaBlockParser, initTracing } = require('@dignetwork/chia-block-listener')

async function parseBlockData() {
  // Initialize tracing
  initTracing()
  
  // Create a block parser instance
  const parser = new ChiaBlockParser()
  
  // Parse a block from hex string
  const blockHex = "your_full_block_hex_data_here"
  const parsedBlock = parser.parseFullBlockFromHex(blockHex)
  
  console.log(`Parsed block ${parsedBlock.height}:`)
  console.log(`  Header hash: ${parsedBlock.headerHash}`)
  console.log(`  Weight: ${parsedBlock.weight}`)
  console.log(`  Timestamp: ${new Date(parsedBlock.timestamp * 1000)}`)
  console.log(`  Coin additions: ${parsedBlock.coinAdditions.length}`)
  console.log(`  Coin removals: ${parsedBlock.coinRemovals.length}`)
  console.log(`  Coin spends: ${parsedBlock.coinSpends.length}`)
  console.log(`  Has generator: ${parsedBlock.hasTransactionsGenerator}`)
  
  // Access detailed coin spend information
  parsedBlock.coinSpends.forEach((spend, index) => {
    console.log(`  Spend ${index + 1}:`)
    console.log(`    Coin: ${spend.coin.amount} mojos`)
    console.log(`    Puzzle hash: ${spend.coin.puzzleHash}`)
    console.log(`    Puzzle reveal: ${spend.puzzleReveal.substring(0, 100)}...`)
    console.log(`    Solution: ${spend.solution.substring(0, 100)}...`)
    console.log(`    Created coins: ${spend.createdCoins.length}`)
  })
}

parseBlockData().catch(console.error)
```

### ChiaBlockParser Class

#### Constructor

```javascript
const parser = new ChiaBlockParser()
```

Creates a new block parser instance with access to the full Rust parsing engine.

#### Methods

##### `parseFullBlockFromBytes(blockBytes): ParsedBlockJs`

Parses a FullBlock from raw bytes.

**Parameters:**
- `blockBytes` (Buffer): The serialized FullBlock data

**Returns:** A `ParsedBlockJs` object containing all parsed block information

```javascript
const fs = require('fs')
const blockData = fs.readFileSync('block.bin')
const parsedBlock = parser.parseFullBlockFromBytes(blockData)
```

##### `parseFullBlockFromHex(blockHex): ParsedBlockJs`

Parses a FullBlock from a hex-encoded string.

**Parameters:**
- `blockHex` (string): The hex-encoded FullBlock data

**Returns:** A `ParsedBlockJs` object containing all parsed block information

```javascript
const blockHex = "deadbeef..." // Your hex-encoded block data
const parsedBlock = parser.parseFullBlockFromHex(blockHex)
```

##### `extractGeneratorFromBlockBytes(blockBytes): string | null`

Extracts only the transactions generator from a block without full parsing.

**Parameters:**
- `blockBytes` (Buffer): The serialized FullBlock data

**Returns:** Hex-encoded generator bytecode or `null` if no generator exists

```javascript
const blockData = fs.readFileSync('block.bin')
const generator = parser.extractGeneratorFromBlockBytes(blockData)
if (generator) {
  console.log(`Generator size: ${generator.length / 2} bytes`)
}
```

##### `getHeightAndTxStatusFromBlockBytes(blockBytes): BlockHeightInfoJs`

Quickly extracts basic block information without full parsing.

**Parameters:**
- `blockBytes` (Buffer): The serialized FullBlock data

**Returns:** A `BlockHeightInfoJs` object with height and transaction status

```javascript
const blockData = fs.readFileSync('block.bin')
const info = parser.getHeightAndTxStatusFromBlockBytes(blockData)
console.log(`Block ${info.height}, has transactions: ${info.isTransactionBlock}`)
```

##### `parseBlockInfoFromBytes(blockBytes): GeneratorBlockInfoJs`

Extracts generator-related block metadata.

**Parameters:**
- `blockBytes` (Buffer): The serialized FullBlock data

**Returns:** A `GeneratorBlockInfoJs` object with generator metadata

```javascript
const blockData = fs.readFileSync('block.bin')
const blockInfo = parser.parseBlockInfoFromBytes(blockData)
console.log(`Previous hash: ${blockInfo.prevHeaderHash}`)
console.log(`Generator refs: ${blockInfo.transactionsGeneratorRefList.length}`)
```

### Advanced Parsing Features

#### Batch Block Processing

```javascript
const parser = new ChiaBlockParser()
const fs = require('fs')
const path = require('path')

// Process multiple block files
const blockFiles = fs.readdirSync('./blocks/').filter(f => f.endsWith('.bin'))

const stats = {
  totalBlocks: 0,
  totalSpends: 0,
  totalCoins: 0,
  generatorBlocks: 0
}

for (const filename of blockFiles) {
  const blockData = fs.readFileSync(path.join('./blocks/', filename))
  
  try {
    const parsed = parser.parseFullBlockFromBytes(blockData)
    
    stats.totalBlocks++
    stats.totalSpends += parsed.coinSpends.length
    stats.totalCoins += parsed.coinAdditions.length
    
    if (parsed.hasTransactionsGenerator) {
      stats.generatorBlocks++
    }
    
    console.log(`Processed block ${parsed.height} with ${parsed.coinSpends.length} spends`)
    
  } catch (error) {
    console.error(`Failed to parse ${filename}:`, error.message)
  }
}

console.log('\nBatch processing complete:')
console.log(`  Blocks processed: ${stats.totalBlocks}`)
console.log(`  Total spends: ${stats.totalSpends}`)
console.log(`  Total coins: ${stats.totalCoins}`)
console.log(`  Generator blocks: ${stats.generatorBlocks}`)
```

#### Generator Analysis

```javascript
const parser = new ChiaBlockParser()

function analyzeBlockGenerator(blockHex) {
  // Parse the full block
  const parsed = parser.parseFullBlockFromHex(blockHex)
  
  if (!parsed.hasTransactionsGenerator) {
    console.log('Block has no generator')
    return
  }
  
  // Extract just the generator for analysis
  const blockBytes = Buffer.from(blockHex, 'hex')
  const generator = parser.extractGeneratorFromBlockBytes(blockBytes)
  
  console.log(`Generator Analysis for Block ${parsed.height}:`)
  console.log(`  Generator size: ${parsed.generatorSize} bytes`)
  console.log(`  Hex length: ${generator.length} characters`)
  console.log(`  Coin spends extracted: ${parsed.coinSpends.length}`)
  console.log(`  Coins created: ${parsed.coinCreations.length}`)
  
  // Analyze coin spends
  parsed.coinSpends.forEach((spend, i) => {
    console.log(`  Spend ${i + 1}:`)
    console.log(`    Amount: ${spend.coin.amount} mojos`)
    console.log(`    Puzzle size: ${spend.puzzleReveal.length / 2} bytes`)
    console.log(`    Solution size: ${spend.solution.length / 2} bytes`)
    console.log(`    Creates ${spend.createdCoins.length} new coins`)
  })
}

// Example usage
const blockHex = "your_generator_block_hex"
analyzeBlockGenerator(blockHex)
```

#### Integration with Other Components

```javascript
const { ChiaBlockParser, ChiaPeerPool, DnsDiscoveryClient } = require('@dignetwork/chia-block-listener')

async function integratedBlockAnalysis() {
  // Set up components
  const parser = new ChiaBlockParser()
  const pool = new ChiaPeerPool()
  const discovery = new DnsDiscoveryClient()
  
  // Discover and connect to peers
  const peers = await discovery.discoverMainnetPeers()
  for (const peer of peers.ipv4Peers.slice(0, 3)) {
    await pool.addPeer(peer.host, peer.port, 'mainnet')
  }
  
  // Fetch blocks and parse with enhanced detail
  const heights = [5000000, 5000001, 5000002]
  
  for (const height of heights) {
    // Get block using peer pool
    const blockEvent = await pool.getBlockByHeight(height)
    
    // For more detailed analysis, you can also parse with ChiaBlockParser
    // if you have access to the raw block bytes
    console.log(`Block ${height}:`)
    console.log(`  From peer: ${blockEvent.peerId}`)
    console.log(`  Coin spends: ${blockEvent.coinSpends.length}`)
    console.log(`  Has generator: ${blockEvent.hasTransactionsGenerator}`)
    
    // Analyze puzzle patterns
    const puzzleHashes = new Set()
    blockEvent.coinSpends.forEach(spend => {
      puzzleHashes.add(spend.coin.puzzleHash)
    })
    console.log(`  Unique puzzle hashes: ${puzzleHashes.size}`)
  }
  
  await pool.shutdown()
}

integratedBlockAnalysis().catch(console.error)
```

### Data Types

#### `ParsedBlockJs`

```typescript
interface ParsedBlockJs {
  height: number                     // Block height
  weight: string                     // Block weight as string
  headerHash: string                 // Block header hash (hex)
  timestamp?: number                 // Block timestamp (Unix time)
  coinAdditions: CoinInfoJs[]        // New coins created
  coinRemovals: CoinInfoJs[]         // Coins spent
  coinSpends: CoinSpendInfoJs[]      // Detailed spend information
  coinCreations: CoinInfoJs[]        // Coins created by spends
  hasTransactionsGenerator: boolean  // Whether block has generator
  generatorSize?: number             // Generator size in bytes
}
```

#### `CoinInfoJs`

```typescript
interface CoinInfoJs {
  parentCoinInfo: string  // Parent coin ID (hex)
  puzzleHash: string      // Puzzle hash (hex)
  amount: string          // Amount as string (to avoid JS precision issues)
}
```

#### `CoinSpendInfoJs`

```typescript
interface CoinSpendInfoJs {
  coin: CoinInfoJs                // The coin being spent
  puzzleReveal: string            // CLVM puzzle bytecode (hex)
  solution: string                // CLVM solution bytecode (hex)
  realData: boolean               // Whether this is real transaction data
  parsingMethod: string           // Method used for parsing
  offset: number                  // Offset in generator bytecode
  createdCoins: CoinInfoJs[]      // Coins created by this spend
}
```

#### `GeneratorBlockInfoJs`

```typescript
interface GeneratorBlockInfoJs {
  prevHeaderHash: string                    // Previous block hash (hex)
  transactionsGenerator?: string            // Generator bytecode (hex)
  transactionsGeneratorRefList: number[]    // Referenced block heights
}
```

#### `BlockHeightInfoJs`

```typescript
interface BlockHeightInfoJs {
  height: number              // Block height
  isTransactionBlock: boolean // Whether block contains transactions
}
```

### When to Use ChiaBlockParser

- **Use ChiaBlockParser when:**
  - You have raw block data from external sources
  - You need detailed CLVM puzzle and solution analysis
  - You're implementing custom block processing logic
  - You need to extract generators for analysis
  - You want fine-grained control over parsing

- **Use with ChiaPeerPool when:**
  - You need both block retrieval and detailed parsing
  - You're analyzing historical blocks in detail
  - You want to combine network access with parsing

- **Use with ChiaBlockListener when:**
  - You're processing real-time blocks with custom logic
  - You need both live monitoring and detailed parsing

The `ChiaBlockParser` complements the other classes by providing the lowest-level access to the Chia block parsing engine, enabling sophisticated analysis and processing workflows.

## TypeScript Usage

```typescript
import { 
  ChiaBlockListener, 
  ChiaPeerPool,
  ChiaBlockParser,
  DnsDiscoveryClient,
  BlockReceivedEvent, 
  PeerConnectedEvent, 
  PeerDisconnectedEvent,
  NewPeakHeightEvent,
  DiscoveryResultJS,
  PeerAddressJS,
  AddressResult,
  CoinRecord,
  CoinSpend,
  ParsedBlockJs,
  CoinInfoJs,
  CoinSpendInfoJs,
  GeneratorBlockInfoJs,
  BlockHeightInfoJs,
  initTracing,
  getEventTypes
} from '@dignetwork/chia-block-listener'

// Initialize tracing for debugging
initTracing()

// Create listener with proper typing
const listener = new ChiaBlockListener()

// Type-safe event handlers
listener.on('blockReceived', (block: BlockReceivedEvent) => {
  console.log(`Block ${block.height} from peer ${block.peerId}`)
  
  // Process coin additions
  block.coinAdditions.forEach((coin: CoinRecord) => {
    console.log(`New coin: ${coin.amount} mojos`)
  })
  
  // Process coin spends
  block.coinSpends.forEach((spend: CoinSpend) => {
    console.log(`Spend: ${spend.coin.amount} mojos`)
    console.log(`Puzzle: ${spend.puzzleReveal}`)
    console.log(`Solution: ${spend.solution}`)
  })
})

listener.on('peerConnected', (peer: PeerConnectedEvent) => {
  console.log(`Connected: ${peer.peerId} at ${peer.host}:${peer.port}`)
})

listener.on('peerDisconnected', (peer: PeerDisconnectedEvent) => {
  console.log(`Disconnected: ${peer.peerId}`)
  if (peer.message) {
    console.log(`Reason: ${peer.message}`)
  }
})

// Connect to peers
const mainnetPeer = listener.addPeer('localhost', 8444, 'mainnet')
const testnetPeer = listener.addPeer('testnet-node.chia.net', 58444, 'testnet')

// Get historical blocks
async function getHistoricalBlocks() {
  try {
    const block = listener.getBlockByHeight(mainnetPeer, 1000000)
    console.log(`Block 1000000 hash: ${block.headerHash}`)
    
    const blocks = listener.getBlocksRange(mainnetPeer, 1000000, 1000010)
    console.log(`Retrieved ${blocks.length} blocks`)
  } catch (error) {
    console.error('Error getting blocks:', error)
  }
}

// Get event type constants
const eventTypes = getEventTypes()
console.log('Available events:', eventTypes)

// TypeScript support for ChiaPeerPool
const pool = new ChiaPeerPool()

// Type-safe event handling
pool.on('peerConnected', (event: PeerConnectedEvent) => {
  console.log(`Pool peer connected: ${event.peerId}`)
})

pool.on('newPeakHeight', (event: NewPeakHeightEvent) => {
  console.log(`New peak: ${event.oldPeak} → ${event.newPeak}`)
})

// Async/await with proper typing
async function fetchHistoricalData() {
  const block: BlockReceivedEvent = await pool.getBlockByHeight(5000000)
  const peers: string[] = await pool.getConnectedPeers()
  const peak: number | null = await pool.getPeakHeight()
  
  console.log(`Block ${block.height} has ${block.coinSpends.length} spends`)
  console.log(`Pool has ${peers.length} active peers`)
  console.log(`Current peak: ${peak || 'No peak yet'}`)
}

// TypeScript DNS Discovery
async function typedDnsDiscovery(): Promise<void> {
  const client = new DnsDiscoveryClient()
  
  // Type-safe discovery
  const result: DiscoveryResultJS = await client.discoverMainnetPeers()
  
  // Access with full type safety
  result.ipv4Peers.forEach((peer: PeerAddressJS) => {
    console.log(`IPv4: ${peer.host}:${peer.port} (${peer.displayAddress})`)
  })
  
  result.ipv6Peers.forEach((peer: PeerAddressJS) => {
    console.log(`IPv6: ${peer.displayAddress} (isIpv6: ${peer.isIpv6})`)
  })
  
  // Individual resolution with types
  const ipv4Result: AddressResult = await client.resolveIpv4('dns-introducer.chia.net')
  const ipv6Result: AddressResult = await client.resolveIpv6('dns-introducer.chia.net')
  
  console.log(`IPv4 count: ${ipv4Result.count}, IPv6 count: ${ipv6Result.count}`)
}

// TypeScript ChiaBlockParser
async function typedBlockParsing(): Promise<void> {
  const parser = new ChiaBlockParser()
  
  // Type-safe block parsing from hex
  const blockHex = "your_block_hex_data"
  const parsedBlock: ParsedBlockJs = parser.parseFullBlockFromHex(blockHex)
  
  console.log(`Parsed block ${parsedBlock.height}:`)
  console.log(`  Header hash: ${parsedBlock.headerHash}`)
  console.log(`  Weight: ${parsedBlock.weight}`)
  console.log(`  Coin additions: ${parsedBlock.coinAdditions.length}`)
  console.log(`  Coin spends: ${parsedBlock.coinSpends.length}`)
  console.log(`  Has generator: ${parsedBlock.hasTransactionsGenerator}`)
  
  // Type-safe access to coin spend details
  parsedBlock.coinSpends.forEach((spend: CoinSpendInfoJs) => {
    const coin: CoinInfoJs = spend.coin
    console.log(`Spend: ${coin.amount} mojos from ${coin.puzzleHash}`)
    console.log(`  Puzzle reveal size: ${spend.puzzleReveal.length / 2} bytes`)
    console.log(`  Solution size: ${spend.solution.length / 2} bytes`)
    console.log(`  Creates ${spend.createdCoins.length} new coins`)
    console.log(`  Real data: ${spend.realData}`)
    console.log(`  Parsing method: ${spend.parsingMethod}`)
  })
  
  // Type-safe generator extraction
  const blockBytes = Buffer.from(blockHex, 'hex')
  const generator: string | null = parser.extractGeneratorFromBlockBytes(blockBytes)
  if (generator) {
    console.log(`Generator found: ${generator.length / 2} bytes`)
  }
  
  // Type-safe height and tx status
  const heightInfo: BlockHeightInfoJs = parser.getHeightAndTxStatusFromBlockBytes(blockBytes)
  console.log(`Block ${heightInfo.height}, is transaction block: ${heightInfo.isTransactionBlock}`)
  
  // Type-safe block info extraction
  const blockInfo: GeneratorBlockInfoJs = parser.parseBlockInfoFromBytes(blockBytes)
  console.log(`Previous hash: ${blockInfo.prevHeaderHash}`)
  console.log(`Generator refs: ${blockInfo.transactionsGeneratorRefList.length}`)
  if (blockInfo.transactionsGenerator) {
    console.log(`Generator size: ${blockInfo.transactionsGenerator.length / 2} bytes`)
  }
}
```

## Advanced Usage

### Monitoring Specific Transactions

```javascript
// Monitor all coin spends for a specific puzzle hash
listener.on('blockReceived', (block) => {
  const targetPuzzleHash = '0x1234...' // Your puzzle hash
  
  block.coinSpends.forEach((spend) => {
    if (spend.coin.puzzleHash === targetPuzzleHash) {
      console.log('Found spend for our puzzle!')
      console.log('Amount:', spend.coin.amount)
      console.log('Solution:', spend.solution)
    }
  })
})
```

### Multiple Network Monitoring

```javascript
// Monitor both mainnet and testnet
const mainnetPeer = listener.addPeer('localhost', 8444, 'mainnet')
const testnetPeer = listener.addPeer('localhost', 58444, 'testnet')

listener.on('blockReceived', (block) => {
  if (block.peerId === mainnetPeer) {
    console.log(`Mainnet block ${block.height}`)
  } else if (block.peerId === testnetPeer) {
    console.log(`Testnet block ${block.height}`)
  }
})
```

### Connection Management

```javascript
// Automatic reconnection
listener.on('peerDisconnected', (peer) => {
  console.log(`Lost connection to ${peer.peerId}, reconnecting...`)
  
  // Reconnect after 5 seconds
  setTimeout(() => {
    try {
      listener.addPeer(peer.host, peer.port, 'mainnet')
      console.log('Reconnected successfully')
    } catch (error) {
      console.error('Reconnection failed:', error)
    }
  }, 5000)
})
```

## Utility Functions

### `initTracing(): void`

Initializes the Rust tracing system for debugging purposes. Call this before creating any `ChiaBlockListener` instances if you want to see debug output.

### `getEventTypes(): EventTypes`

Returns an object containing the event type constants:

```javascript
const eventTypes = getEventTypes()
console.log(eventTypes)
// Output: { blockReceived: "blockReceived", peerConnected: "peerConnected", peerDisconnected: "peerDisconnected" }
```



## Performance Tips

1. **Use specific event handlers**: Only listen for the events you need
2. **Process blocks efficiently**: Avoid heavy computation in event handlers
3. **Manage connections**: Don't create too many peer connections simultaneously
4. **Handle errors gracefully**: Always wrap peer operations in try-catch blocks

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (20 or later)
- [npm](https://www.npmjs.com/)

### Setup

```bash
# Clone and install dependencies
git clone <repository-url>
cd chia-block-listener
npm install

# Build the native module
npm run build

# Run tests
npm test
```

### Project Structure

```
chia-block-listener/
├── src/                     # Rust source code
│   ├── lib.rs              # Main NAPI bindings
│   ├── peer.rs             # Peer connection management
│   ├── protocol.rs         # Chia protocol implementation
│   ├── event_emitter.rs    # Event system
│   └── tls.rs              # TLS connection handling
├── crate/                  # Additional Rust crates
│   └── chia-generator-parser/ # CLVM parser
├── __test__/               # Test suite
├── npm/                    # Platform-specific binaries
├── .github/workflows/      # CI/CD pipeline
├── Cargo.toml              # Rust configuration
├── package.json            # Node.js configuration
└── index.d.ts              # TypeScript definitions
```

## CI/CD & Publishing

This project uses GitHub Actions for:
- Cross-platform builds (Windows, macOS, Linux)
- Multiple architectures (x64, ARM64)
- Automated testing on all platforms
- npm publishing based on git tags

## License

MIT

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes and add tests
4. Ensure all tests pass (`npm test`)
5. Commit your changes (`git commit -m 'Add some amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## Support

For issues and questions:
- GitHub Issues: [Report bugs or request features](https://github.com/DIG-Network/chia-block-listener/issues)
- Documentation: Check the TypeScript definitions in `index.d.ts`
- Examples: See the `examples/` directory for more usage examples 