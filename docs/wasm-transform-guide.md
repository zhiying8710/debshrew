# WASM Transform Guide

This guide explains how to create WASM transform modules for Debshrew.

## What are WASM Transform Modules?

WASM transform modules are WebAssembly modules that implement the `DebTransform` trait. They are responsible for:

- Processing blocks
- Querying metashrew views
- Detecting changes
- Generating CDC messages
- Handling rollbacks

## Prerequisites

Before creating a WASM transform module, you'll need:

- Rust installed (version 1.70 or later)
- The `wasm32-unknown-unknown` target installed:

```bash
rustup target add wasm32-unknown-unknown
```

## Creating a New Transform Module

### Step 1: Create a New Rust Project

```bash
cargo new --lib my-transform
cd my-transform
```

### Step 2: Configure Cargo.toml

Add the following to your `Cargo.toml`:

```toml
[package]
name = "my-transform"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
debshrew-runtime = { path = "../debshrew-runtime" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
hex = "0.4"
```

### Step 3: Implement the Transform Module

Create a basic transform module in `src/lib.rs`:

```rust
use debshrew_runtime::*;

#[derive(Debug, Default)]
struct MyTransform {
    state: TransformState
}

impl DebTransform for MyTransform {
    fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
        // Get current block info
        let height = get_height();
        let hash = get_block_hash();
        
        // Generate a simple CDC message
        let messages = vec![
            CdcMessage {
                header: CdcHeader {
                    source: "my_transform".to_string(),
                    timestamp: chrono::Utc::now(),
                    block_height: height,
                    block_hash: hex::encode(&hash),
                    transaction_id: None,
                },
                payload: CdcPayload {
                    operation: CdcOperation::Create,
                    table: "blocks".to_string(),
                    key: height.to_string(),
                    before: None,
                    after: Some(serde_json::json!({
                        "height": height,
                        "hash": hex::encode(&hash),
                        "timestamp": chrono::Utc::now()
                    })),
                },
            }
        ];
        
        Ok(messages)
    }
    
    fn rollback(&mut self) -> Result<Vec<CdcMessage>> {
        // Get current block info
        let height = get_height();
        
        // Generate an inverse CDC message
        let messages = vec![
            CdcMessage {
                header: CdcHeader {
                    source: "my_transform".to_string(),
                    timestamp: chrono::Utc::now(),
                    block_height: height,
                    block_hash: hex::encode(&get_block_hash()),
                    transaction_id: None,
                },
                payload: CdcPayload {
                    operation: CdcOperation::Delete,
                    table: "blocks".to_string(),
                    key: height.to_string(),
                    before: Some(serde_json::json!({
                        "height": height,
                        "hash": hex::encode(&get_block_hash()),
                        "timestamp": chrono::Utc::now()
                    })),
                    after: None,
                },
            }
        ];
        
        Ok(messages)
    }
}

// Declare the transform module
declare_transform!(MyTransform);
```

### Step 4: Build the Transform Module

```bash
cargo build --target wasm32-unknown-unknown --release
```

The WASM module will be available at `target/wasm32-unknown-unknown/release/my_transform.wasm`.

## The DebTransform Trait

The `DebTransform` trait is the core of a transform module. It has two required methods:

### process_block

This method is called for each new block. It should:

1. Get the current block information
2. Query metashrew views
3. Detect changes
4. Generate CDC messages

```rust
fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
    // Implementation
}
```

### rollback

This method is called during a reorg. It should:

1. Get the current block information
2. Generate inverse CDC messages

```rust
fn rollback(&mut self) -> Result<Vec<CdcMessage>> {
    // Implementation
}
```

## Host Functions

Debshrew provides several host functions that transform modules can use:

### Block Information

- `get_height()`: Get the current block height
- `get_block_hash()`: Get the current block hash

### State Management

- `get_state(key: &str)`: Get a value from the state
- `set_state(key: &str, value: &str)`: Set a value in the state
- `delete_state(key: &str)`: Delete a value from the state
- `get_state_keys()`: Get all keys in the state
- `get_state_keys_with_prefix(prefix: &str)`: Get all keys with a specific prefix

### View Functions

- `call_view(name: &str, params: &[u8])`: Call a metashrew view function

### Serialization

- `serialize(value: &T)`: Serialize a value to bytes
- `deserialize(bytes: &[u8])`: Deserialize bytes to a value
- `serialize_to_json(value: &T)`: Serialize a value to JSON

### Logging

- `log(message: &str)`: Log a message

## State Management

Transform modules can maintain state between blocks using the state management functions. The state is:

1. Persisted between block processing
2. Rolled back during reorgs
3. Managed by the WASM runtime
4. Accessible through host functions

Example:

```rust
// Get a value from the state
let count_str = get_state("count").unwrap_or("0".to_string());
let mut count: u32 = count_str.parse().unwrap_or(0);

// Update the value
count += 1;

// Store the updated value
set_state("count", &count.to_string());
```

## Calling Metashrew Views

Transform modules can call metashrew views to get data:

```rust
// Serialize the parameters
let params = serialize(&height)?;

// Call the view function
let result = call_view("get_block_transactions", &params)?;

// Deserialize the result
let transactions: Vec<Transaction> = deserialize(&result)?;
```

## Generating CDC Messages

Transform modules generate CDC messages to represent changes:

```rust
let message = CdcMessage {
    header: CdcHeader {
        source: "my_transform".to_string(),
        timestamp: chrono::Utc::now(),
        block_height: height,
        block_hash: hex::encode(&hash),
        transaction_id: Some(tx_id),
    },
    payload: CdcPayload {
        operation: CdcOperation::Create,
        table: "transactions".to_string(),
        key: tx_id,
        before: None,
        after: Some(serde_json::to_value(tx)?),
    },
};
```

## Handling Rollbacks

During a reorg, transform modules need to generate inverse CDC messages:

```rust
fn rollback(&mut self) -> Result<Vec<CdcMessage>> {
    // Get the records that were created in this block
    let records = get_records_for_block(get_height())?;
    
    // Generate inverse CDC messages
    let mut messages = Vec::new();
    
    for record in records {
        messages.push(CdcMessage {
            header: CdcHeader {
                source: "my_transform".to_string(),
                timestamp: chrono::Utc::now(),
                block_height: get_height(),
                block_hash: hex::encode(&get_block_hash()),
                transaction_id: record.tx_id.clone(),
            },
            payload: CdcPayload {
                operation: CdcOperation::Delete,
                table: record.table.clone(),
                key: record.key.clone(),
                before: Some(record.data.clone()),
                after: None,
            },
        });
    }
    
    Ok(messages)
}
```

## Advanced Techniques

### Batching CDC Messages

For better performance, batch CDC messages:

```rust
let mut messages = Vec::new();

for tx in transactions {
    // Process transaction
    // ...
    
    // Add CDC message
    messages.push(cdc_message);
}

Ok(messages)
```

### Incremental Processing

For large datasets, use incremental processing:

```rust
// Get the last processed transaction index
let last_index_str = get_state("last_tx_index").unwrap_or("0".to_string());
let last_index: usize = last_index_str.parse().unwrap_or(0);

// Process transactions starting from the last index
for (i, tx) in transactions.iter().enumerate().skip(last_index) {
    // Process transaction
    // ...
    
    // Update the last processed index
    set_state("last_tx_index", &(i + 1).to_string());
}
```

### Error Handling

Handle errors gracefully:

```rust
fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
    // Try to call a view function
    let result = match call_view("my_view", &[]) {
        Ok(result) => result,
        Err(e) => {
            log(&format!("Error calling view: {:?}", e));
            return Ok(vec![]); // Return empty vector instead of propagating the error
        }
    };
    
    // Continue processing
    // ...
}
```

## Testing Transform Modules

### Unit Testing

You can unit test your transform module using the `MockTransform` struct:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use debshrew_runtime::MockTransform;
    
    #[test]
    fn test_process_block() {
        let mut transform = MockTransform::default();
        
        // Set up mock state
        transform.state.set("count", "42");
        
        // Call process_block
        let messages = transform.process_block().unwrap();
        
        // Verify the messages
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].payload.table, "blocks");
        assert_eq!(messages[0].payload.operation, CdcOperation::Create);
    }
}
```

### Integration Testing

For integration testing, you can use Debshrew's test utilities:

```rust
use debshrew::test_utils::{TestMetashrewClient, TestSink};

#[test]
fn test_transform_integration() {
    // Create a test metashrew client
    let client = TestMetashrewClient::new();
    
    // Create a test sink
    let sink = TestSink::new();
    
    // Create a WASM runtime with your transform module
    let runtime = WasmRuntime::new("path/to/transform.wasm").unwrap();
    
    // Create a block synchronizer
    let mut synchronizer = BlockSynchronizer::new(client, runtime, Box::new(sink.clone()), 6).unwrap();
    
    // Process a block
    synchronizer.process_block(123).unwrap();
    
    // Verify the CDC messages
    let messages = sink.get_messages();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].payload.table, "blocks");
    assert_eq!(messages[0].payload.operation, CdcOperation::Create);
}
```

## Debugging Transform Modules

Debugging WASM modules can be challenging. Here are some tips:

1. Use the `log` function to output debug information
2. Check the Debshrew logs for errors
3. Use the `console` sink to see CDC messages
4. Add debug assertions in your code
5. Use the `--debug` flag when running Debshrew

## Performance Optimization

To optimize your transform module:

1. Minimize view function calls
2. Batch CDC messages
3. Use incremental processing
4. Optimize state management
5. Profile your code

## Security Considerations

When writing transform modules, consider:

1. Input validation
2. Error handling
3. Resource usage
4. Data privacy
5. Determinism