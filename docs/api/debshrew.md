# Debshrew API Reference

This document provides a reference for the main Debshrew API.

## BlockSynchronizer

The `BlockSynchronizer` is the main component of Debshrew. It synchronizes with metashrew, processes blocks, and handles reorgs.

### Constructor

```rust
pub fn new<C: MetashrewClient>(
    client: C,
    runtime: WasmRuntime,
    sink: Box<dyn CdcSink>,
    cache_size: u32
) -> Result<Self>
```

Creates a new `BlockSynchronizer` with the specified metashrew client, WASM runtime, CDC sink, and cache size.

### Methods

#### run

```rust
pub async fn run(&mut self) -> Result<()>
```

Starts the block synchronizer and runs until stopped.

#### stop

```rust
pub fn stop(&mut self)
```

Stops the block synchronizer.

#### set_polling_interval

```rust
pub fn set_polling_interval(&mut self, interval: u64)
```

Sets the polling interval in milliseconds.

#### set_starting_height

```rust
pub fn set_starting_height(&mut self, height: u32)
```

Sets the starting block height.

#### get_current_height

```rust
pub fn get_current_height(&self) -> u32
```

Gets the current block height.

#### get_sink

```rust
pub fn get_sink(&self) -> Arc<Box<dyn CdcSink>>
```

Gets the CDC sink.

#### get_client

```rust
pub fn get_client(&self) -> Arc<C>
```

Gets the metashrew client.

## MetashrewClient

The `MetashrewClient` trait defines the interface for communicating with metashrew.

### Methods

#### get_height

```rust
async fn get_height(&self) -> Result<u32>
```

Gets the current block height from metashrew.

#### get_block_hash

```rust
async fn get_block_hash(&self, height: u32) -> Result<Vec<u8>>
```

Gets the block hash for the specified height.

#### call_view

```rust
async fn call_view(&self, name: &str, params: &[u8]) -> Result<Vec<u8>>
```

Calls a metashrew view function.

## JsonRpcClient

The `JsonRpcClient` is an implementation of the `MetashrewClient` trait that communicates with metashrew using JSON-RPC.

### Constructor

```rust
pub fn new(url: &str) -> Result<Self>
```

Creates a new `JsonRpcClient` with the specified URL.

### Methods

Implements all methods from the `MetashrewClient` trait.

## CdcSink

The `CdcSink` trait defines the interface for CDC sinks.

### Methods

#### send

```rust
async fn send(&self, messages: Vec<CdcMessage>) -> Result<()>
```

Sends CDC messages to the sink.

#### flush

```rust
async fn flush(&self) -> Result<()>
```

Flushes any buffered messages.

#### close

```rust
async fn close(&self) -> Result<()>
```

Closes the sink.

## Sink Implementations

### KafkaSink

```rust
pub fn new(
    bootstrap_servers: &str,
    topic: &str,
    client_id: Option<&str>,
    batch_size: usize,
    flush_interval: u64
) -> Result<Self>
```

Creates a new `KafkaSink` that sends CDC messages to a Kafka topic.

### PostgresSink

```rust
pub fn new(
    connection_string: &str,
    schema: &str,
    batch_size: usize,
    flush_interval: u64
) -> Result<Self>
```

Creates a new `PostgresSink` that applies CDC messages to a PostgreSQL database.

### FileSink

```rust
pub fn new(
    path: &str,
    append: bool,
    flush_interval: u64
) -> Result<Self>
```

Creates a new `FileSink` that writes CDC messages to a file.

### ConsoleSink

```rust
pub fn new(pretty: bool) -> Self
```

Creates a new `ConsoleSink` that outputs CDC messages to the console.

### NullSink

```rust
pub fn new() -> Self
```

Creates a new `NullSink` that discards all CDC messages.

## Configuration

### SinkConfig

The `SinkConfig` enum represents the configuration for a CDC sink.

```rust
pub enum SinkConfig {
    Kafka {
        bootstrap_servers: String,
        topic: String,
        client_id: Option<String>,
        batch_size: usize,
        flush_interval: u64,
    },
    Postgres {
        connection_string: String,
        schema: String,
        batch_size: usize,
        flush_interval: u64,
    },
    File {
        path: String,
        append: bool,
        flush_interval: u64,
    },
    Console {
        pretty: bool,
    },
    Null,
}
```

### create_sink

```rust
pub fn create_sink(config: &SinkConfig) -> Result<Box<dyn CdcSink>>
```

Creates a CDC sink from the specified configuration.

## BlockCache

The `BlockCache` maintains a cache of recent blocks and state snapshots.

### Constructor

```rust
pub fn new(max_size: u32) -> Result<Self>
```

Creates a new `BlockCache` with the specified maximum size.

### Methods

#### add_block

```rust
pub fn add_block(
    &mut self,
    metadata: BlockMetadata,
    transform_result: TransformResult
) -> Result<()>
```

Adds a block to the cache.

#### find_common_ancestor

```rust
pub fn find_common_ancestor(&self, new_hashes: &[(u32, String)]) -> Option<u32>
```

Finds the common ancestor between the current chain and a new chain.

#### rollback

```rust
pub fn rollback(&mut self, height: u32) -> Result<TransformState>
```

Rolls back to the specified height.

## Error Handling

### Error

The `Error` enum represents errors that can occur in Debshrew.

```rust
pub enum Error {
    BlockSynchronization(String),
    MetashrewClient(String),
    Wasm(String),
    Serialization(String),
    Sink(String),
    ReorgHandling(String),
    Configuration(String),
    Io(std::io::Error),
}
```

### Result

The `Result` type is a shorthand for `std::result::Result<T, Error>`.

```rust
pub type Result<T> = std::result::Result<T, Error>;