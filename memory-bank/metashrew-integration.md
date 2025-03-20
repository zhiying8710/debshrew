# Metashrew Integration Guide

This document provides a comprehensive guide to how debshrew integrates with metashrew, explaining the key concepts, interfaces, and implementation details.

## Overview

Debshrew relies on metashrew as its primary data source, transforming metaprotocol state indexed by metashrew into standardized Change Data Capture (CDC) streams. Understanding metashrew's architecture and interfaces is essential for developing effective debshrew transform modules.

## Metashrew Architecture

Metashrew is a Bitcoin indexer framework powered by WebAssembly (WASM) with the following key components:

1. **Core Indexer**: Connects to a Bitcoin node, retrieves blocks, and passes them to WASM modules for processing.
2. **WASM Runtime**: Executes WASM modules with a standardized host interface.
3. **Append-Only Database**: Stores indexed data with versioning for historical queries and reorg handling.
4. **JSON-RPC Server**: Provides an API for querying indexed data through view functions.

## Metashrew WASM Interface

Metashrew provides the following host functions to WASM modules:

```
__host_len(): i32                      // Get input data length
__load_input(ptr: i32): void           // Load input data into WASM memory
__log(ptr: i32): void                  // Write to stdout (UTF-8 encoded)
__flush(ptr: i32): void                // Commit key-value pairs to database
__get_len(ptr: i32): i32               // Get value length for a key
__get(key_ptr: i32, value_ptr: i32)    // Read value for a key
```

WASM modules must export:

1. `_start()`: Main indexing function that processes blocks and updates the database.
2. View functions (optional): Custom named exports for querying indexed data.

## Metashrew Data Model

Metashrew uses an append-only database with the following characteristics:

1. **Versioned Keys**: Each key maintains a list of values, indexed by position.
2. **Height Annotation**: Values are annotated with the block height they were created at.
3. **Length Tracking**: Special length keys track the number of values for each key.
4. **Update Tracking**: For each block height, a list of updated keys is maintained.

The key-value pairs are flushed to the database using the `KeyValueFlush` protobuf message:

```protobuf
message KeyValueFlush {
  repeated bytes list = 1;  // Alternating key-value pairs
}
```

## Debshrew Integration Points

Debshrew integrates with metashrew at several key points:

### 1. Block Synchronization

Debshrew connects to a metashrew instance and synchronizes with its block processing:

```rust
pub struct BlockSynchronizer<T: MetashrewClient> {
    client: T,
    current_height: u32,
    cache: BlockCache,
    transform: Box<dyn DebTransform>,
    sink: Box<dyn CdcSink>,
}

impl<T: MetashrewClient> BlockSynchronizer<T> {
    pub fn poll(&mut self) -> Result<()> {
        let metashrew_height = self.client.get_height()?;
        
        if metashrew_height > self.current_height {
            // Process new blocks
            for height in self.current_height..=metashrew_height {
                self.process_block(height)?;
            }
            self.current_height = metashrew_height;
        } else if metashrew_height < self.current_height {
            // Handle reorg
            self.handle_reorg(metashrew_height)?;
        }
        
        Ok(())
    }
}
```

### 2. View Access

Debshrew provides access to metashrew views through a standardized interface:

```rust
pub trait MetashrewClient {
    fn get_height(&self) -> Result<u32>;
    fn get_block_hash(&self, height: u32) -> Result<Vec<u8>>;
    fn call_view(&self, view_name: &str, params: &[u8]) -> Result<Vec<u8>>;
}

pub struct JsonRpcClient {
    url: String,
    client: reqwest::Client,
}

impl MetashrewClient for JsonRpcClient {
    fn call_view(&self, view_name: &str, params: &[u8]) -> Result<Vec<u8>> {
        // Call metashrew view function via JSON-RPC
    }
    
    fn get_height(&self) -> Result<u32> {
        // Get current height via JSON-RPC
    }
    
    fn get_block_hash(&self, height: u32) -> Result<Vec<u8>> {
        // Get block hash via JSON-RPC
    }
}
```

### 3. Transform Interface

Debshrew transform modules implement the `DebTransform` trait:

```rust
pub trait DebTransform {
    fn process_block(&mut self) -> Result<Vec<CdcMessage>>;
    fn rollback(&mut self) -> Result<Vec<CdcMessage>>;
}
```

The host provides the following functions to transform modules:

```rust
// View access
pub fn call_view(view_name: &str, params: &[u8]) -> Result<Vec<u8>>;
pub fn get_height() -> u32;
pub fn get_block_hash() -> Vec<u8>;

// State management
pub fn get_state(key: &[u8]) -> Option<Vec<u8>>;
pub fn set_state(key: &[u8], value: &[u8]);
pub fn delete_state(key: &[u8]);

// Logging
pub fn log(message: &str);
```

### 4. Reorg Handling

Debshrew detects reorgs by comparing block hashes with metashrew:

```rust
impl<T: MetashrewClient> BlockSynchronizer<T> {
    pub fn handle_reorg(&mut self, new_height: u32) -> Result<()> {
        // Find common ancestor
        let mut common_height = new_height;
        while common_height > 0 {
            let cached_hash = self.cache.get_block_hash(common_height)?;
            let metashrew_hash = self.client.get_block_hash(common_height)?;
            
            if cached_hash == metashrew_hash {
                break;
            }
            
            common_height -= 1;
        }
        
        // Rollback to common ancestor
        for height in (common_height + 1..=self.current_height).rev() {
            let inverse_messages = self.transform.rollback()?;
            self.sink.send(inverse_messages)?;
            self.cache.remove_block(height)?;
        }
        
        // Process new chain
        for height in (common_height + 1)..=new_height {
            self.process_block(height)?;
        }
        
        self.current_height = new_height;
        Ok(())
    }
}
```

## Metashrew View Functions

Metashrew view functions are custom exports from WASM modules that provide query capabilities. Debshrew transform modules use these view functions to access metaprotocol state.

Example of calling a view function from a transform module:

```rust
fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
    let height = get_height();
    let hash = get_block_hash();
    
    // Call metashrew view function
    let params = serialize_params(&ExampleParams { /* ... */ });
    let result = call_view("example_view", &params)?;
    let view_data: ExampleViewResult = deserialize_result(&result)?;
    
    // Process view data and generate CDC messages
    // ...
}
```

## Memory Layout

Data passed between metashrew and WASM modules follows AssemblyScript's ArrayBuffer memory layout:
- 4 bytes for length (little-endian u32)
- Followed by actual data bytes

Debshrew uses the same memory layout for consistency.

## Reorg Handling Comparison

| Metashrew | Debshrew |
|-----------|----------|
| Maintains versioned key-value pairs with block height annotations | Maintains a block cache with state snapshots |
| Automatically rolls back database state during reorgs | Generates inverse CDC messages for rollbacks |
| Uses a list of updated keys per block for efficient rollbacks | Uses state snapshots for efficient rollbacks |
| Handles reorgs at the database level | Handles reorgs at the CDC message level |

## Best Practices for Transform Modules

1. **Efficient View Access**: Minimize the number of view calls and the amount of data transferred.
2. **Incremental Processing**: Only generate CDC messages for changed state.
3. **Proper State Management**: Maintain minimal state for efficient processing and rollbacks.
4. **Error Handling**: Implement robust error handling for view calls and state management.
5. **Deterministic Processing**: Ensure deterministic behavior for consistent CDC output.

## Example: Calling Metashrew Views

```rust
// Define parameters for the view function
#[derive(Serialize, Deserialize)]
struct TokenBalanceParams {
    address: String,
}

// Call the view function
let params = serde_json::to_vec(&TokenBalanceParams {
    address: "bc1q...".to_string(),
})?;

let result = call_view("get_token_balance", &params)?;

// Deserialize the result
let balance: u64 = serde_json::from_slice(&result)?;

// Generate CDC message
let message = CdcMessage {
    header: CdcHeader {
        source: "token_protocol".to_string(),
        timestamp: now_ms(),
        block_height: get_height(),
        block_hash: hex::encode(get_block_hash()),
        transaction_id: None,
    },
    payload: CdcPayload {
        operation: CdcOperation::Update,
        table: "balances".to_string(),
        key: address.to_string(),
        before: Some(json!({ "balance": self.state.get_balance(&address) })),
        after: Some(json!({ "balance": balance })),
    },
};
```

## Conclusion

Understanding metashrew's architecture and interfaces is crucial for developing effective debshrew transform modules. This guide provides the essential information needed to integrate with metashrew and leverage its capabilities for generating CDC streams from metaprotocol state.

By following the patterns and best practices outlined in this guide, developers can create robust, efficient transform modules that handle reorgs correctly and generate consistent CDC output.