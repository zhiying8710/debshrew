# Debshrew System Patterns

## Core Architectural Patterns

Debshrew's architecture is built around several key design patterns that enable its flexibility, reliability, and extensibility.

### 1. WebAssembly Transform Pattern

The core of Debshrew is its WebAssembly transform interface, which defines a contract between the host (Debshrew) and the guest (WASM transform modules):

```
┌─────────────────────┐      ┌─────────────────────┐
│                     │      │                     │
│  WASM Transform     │      │  Debshrew Host      │
│  Module             │◄────►│                     │
│                     │      │                     │
└─────────────────────┘      └─────────────────────┘
        │                              │
        │ Implements                   │ Provides
        │ - DebTransform trait         │ - View access
        │ - process_block()            │ - Block data
        │ - rollback()                 │ - State persistence
        │                              │ - CDC output
```

#### Host Functions

1. **View Access**: Provides access to metashrew views for querying metaprotocol state.
2. **Block Data**: Provides block height, hash, and timestamp information.
3. **State Persistence**: Manages transform state persistence between blocks.
4. **CDC Output**: Handles the output of CDC messages to the configured sink.

#### Guest Functions

1. **`process_block()`**: Processes a block and generates CDC messages.
2. **`rollback()`**: Generates inverse CDC messages for rollback during reorgs.

This pattern enables language-agnostic communication between the host and any WASM module that implements the interface, allowing developers to write transforms in any language that compiles to WebAssembly.

### 2. Block Cache Pattern

Debshrew uses a block cache pattern for reorg handling:

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ Block N-6       │     │ Block N-5       │     │ Block N-4       │
│ State + CDC     │────►│ State + CDC     │────►│ State + CDC     │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                        │
                                                        │
                                                        ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ Block N-3       │     │ Block N-2       │     │ Block N-1       │
│ State + CDC     │◄────│ State + CDC     │◄────│ State + CDC     │
└─────────────────┘     └─────────────────┘     └─────────────────┘
        │
        │
        ▼
┌─────────────────┐
│ Block N (Head)  │
│ State + CDC     │
└─────────────────┘
```

Key characteristics:

1. **Fixed-Size Cache**: Maintains a cache of the last 6 blocks (configurable).
2. **State Snapshots**: Each cache entry includes a snapshot of the transform state.
3. **CDC Message History**: Each cache entry includes the CDC messages generated for that block.
4. **FIFO Eviction**: When a new block is processed, the oldest block is evicted from the cache.

Benefits:

1. **Reorg Handling**: Enables rolling back to a previous state during chain reorganizations.
2. **Deterministic Recovery**: Ensures consistent state recovery after reorgs.
3. **Performance**: Avoids reprocessing the entire chain after a reorg.
4. **Memory Efficiency**: Limits the memory footprint by maintaining a fixed-size cache.

### 3. CDC Message Pattern

Debshrew uses a standardized CDC message pattern for data integration:

```
┌─────────────────────────────────────────┐
│ CDC Message                             │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │ Header                          │    │
│  │ - source                        │    │
│  │ - timestamp                     │    │
│  │ - block_height                  │    │
│  │ - block_hash                    │    │
│  │ - transaction_id                │    │
│  └─────────────────────────────────┘    │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │ Payload                         │    │
│  │ - operation (create/update/delete) │
│  │ - table                         │    │
│  │ - key                           │    │
│  │ - before (optional)             │    │
│  │ - after (optional)              │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

This pattern:

1. **Standardizes Format**: Provides a consistent format for all CDC messages.
2. **Includes Metadata**: Each message includes metadata about the source block and transaction.
3. **Supports CRUD Operations**: Supports create, read, update, and delete operations.
4. **Enables Rollbacks**: The before/after fields enable generating inverse operations for rollbacks.

### 4. Dependency Injection Pattern

Debshrew uses dependency injection for CDC output sinks:

```rust
pub trait CdcSink {
    type Error: std::fmt::Debug;
    fn send(&mut self, messages: Vec<CdcMessage>) -> Result<(), Self::Error>;
    fn flush(&mut self) -> Result<(), Self::Error>;
}
```

This pattern:

1. **Abstracts Output**: Separates the CDC output interface from its implementation.
2. **Enables Multiple Sinks**: Allows for different output sinks (Kafka, PostgreSQL, etc.).
3. **Simplifies Testing**: Makes it easier to use mock implementations for testing.
4. **Promotes Modularity**: Keeps components loosely coupled.

### 5. Transform State Management Pattern

Debshrew uses a state management pattern for transform modules:

```rust
pub struct TransformState {
    inner: HashMap<Vec<u8>, Vec<u8>>,
    dirty: bool,
}

impl TransformState {
    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>>;
    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>);
    pub fn delete(&mut self, key: &[u8]);
    pub fn is_dirty(&self) -> bool;
    pub fn mark_clean(&mut self);
}
```

This pattern:

1. **Encapsulates State**: Provides a simple key-value store for transform state.
2. **Tracks Changes**: Tracks whether the state has been modified.
3. **Enables Snapshots**: Allows creating snapshots of the state for the block cache.
4. **Simplifies Persistence**: Handles state persistence between blocks.

## Data Flow Patterns

### 1. Block Processing Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│             │     │             │     │             │     │             │
│  Metashrew  │────►│  Debshrew   │────►│  WASM       │────►│  CDC        │
│  Instance   │     │  Service    │     │  Transform  │     │  Sink       │
│             │     │             │     │             │     │             │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

### 2. Reorg Handling Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│             │     │             │     │             │     │             │
│  Detect     │────►│  Rollback   │────►│  Generate   │────►│  Process    │
│  Reorg      │     │  to Common  │     │  Inverse    │     │  New        │
│             │     │  Ancestor   │     │  CDC        │     │  Chain      │
│             │     │             │     │             │     │             │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

### 3. CDC Output Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│             │     │             │     │             │
│  Generate   │────►│  Buffer     │────►│  Flush to   │
│  CDC        │     │  Messages   │     │  Sink       │
│  Messages   │     │             │     │             │
│             │     │             │     │             │
└─────────────┘     └─────────────┘     └─────────────┘
```

## Error Handling Patterns

### 1. Result Propagation

Debshrew uses Rust's `Result` type for error propagation:

```rust
fn process_block(&mut self, height: u32, hash: &[u8]) -> Result<(), Error> {
    let transform_result = self.transform.process_block(height, hash)
        .context("Failed to process block in transform")?;
    
    self.cache.add_block(height, hash, transform_result.state_snapshot, transform_result.cdc_messages)
        .context("Failed to add block to cache")?;
    
    self.sink.send(transform_result.cdc_messages)
        .context("Failed to send CDC messages to sink")?;
    
    Ok(())
}
```

This pattern:

1. **Propagates Context**: Adds context to errors as they propagate up the call stack.
2. **Handles Failures Gracefully**: Attempts to recover from failures when possible.
3. **Provides Detailed Errors**: Gives detailed error information for debugging.

### 2. Retry Pattern for CDC Output

Debshrew uses a retry pattern for CDC output to handle transient failures:

```rust
fn send_with_retry(&mut self, messages: Vec<CdcMessage>) -> Result<(), Error> {
    let mut attempts = 0;
    let max_attempts = self.config.max_retry_attempts;
    let backoff = self.config.retry_backoff_ms;
    
    while attempts < max_attempts {
        match self.sink.send(messages.clone()) {
            Ok(_) => return Ok(()),
            Err(e) if is_transient_error(&e) => {
                attempts += 1;
                if attempts < max_attempts {
                    thread::sleep(Duration::from_millis(backoff * attempts as u64));
                    continue;
                }
                return Err(e).context(format!("Failed after {} attempts", attempts));
            },
            Err(e) => return Err(e).context("Non-transient error sending CDC messages"),
        }
    }
    
    unreachable!()
}
```

This pattern:

1. **Handles Transient Failures**: Retries operations that fail due to transient issues.
2. **Implements Backoff**: Uses exponential backoff to avoid overwhelming the sink.
3. **Limits Retries**: Sets a maximum number of retry attempts to avoid infinite loops.
4. **Preserves Context**: Maintains error context for debugging.

## Integration Patterns with Metashrew

### 1. View Access Pattern

Debshrew provides access to metashrew views through a standardized interface:

```rust
pub trait ViewAccess {
    fn call_view(&self, view_name: &str, params: &[u8]) -> Result<Vec<u8>, Error>;
    fn get_height(&self) -> Result<u32, Error>;
    fn get_block_hash(&self, height: u32) -> Result<Vec<u8>, Error>;
}
```

This pattern:

1. **Abstracts Metashrew Access**: Provides a consistent interface for accessing metashrew views.
2. **Enables Mocking**: Makes it easier to mock view access for testing.
3. **Handles Serialization**: Manages serialization and deserialization of view parameters and results.

### 2. Block Synchronization Pattern

Debshrew synchronizes with metashrew using a polling pattern:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│             │     │             │     │             │
│  Poll       │────►│  Compare    │────►│  Process    │
│  Metashrew  │     │  Heights    │     │  New        │
│  Height     │     │             │     │  Blocks     │
│             │     │             │     │             │
└─────────────┘     └─────────────┘     └─────────────┘
        │                                      │
        │                                      │
        ▼                                      ▼
┌─────────────┐                        ┌─────────────┐
│             │                        │             │
│  Sleep      │                        │  Check for  │
│  Interval   │                        │  Reorgs     │
│             │                        │             │
└─────────────┘                        └─────────────┘
```

This pattern:

1. **Maintains Synchronization**: Keeps debshrew in sync with metashrew.
2. **Detects Reorgs**: Identifies chain reorganizations by comparing block hashes.
3. **Handles Backfilling**: Processes multiple blocks when catching up.
4. **Conserves Resources**: Uses polling with a configurable interval to avoid excessive CPU usage.

## Conclusion

Debshrew's system patterns enable a flexible, reliable framework for transforming metaprotocol state into standardized CDC streams. The combination of WebAssembly, block caching, and modular architecture creates a powerful platform that can support a wide range of data integration scenarios while handling the complexities of Bitcoin's blockchain, including chain reorganizations.

The integration with metashrew demonstrates how these patterns can be leveraged to build sophisticated data pipelines for Bitcoin metaprotocols, showcasing the framework's capabilities and flexibility.