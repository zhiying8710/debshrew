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
debshrew-runtime = { path = "../debshrew-runtime", default-features = false }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
hex = "0.4"
getrandom = { version = "0.2", features = ["js"] }
```

Note: The `default-features = false` is important to disable the host features that aren't compatible with the WASM target. The `getrandom` dependency with the `js` feature is required for UUID generation in WASM environments.

### Step 3: Implement the Transform Module

Create a basic transform module in `src/lib.rs`:

```rust
use debshrew_runtime::*;

#[derive(Debug, Default, Clone)]
struct MyTransform {
    // State fields
}

impl DebTransform for MyTransform {
    fn process_block(&mut self) -> Result<()> {
        // Get current block info
        let height = get_height();
        let hash = get_block_hash();
        
        // Generate a simple CDC message
        let message = CdcMessage {
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
        };
        
        // Push CDC message
        self.push_message(message)?;
        
        Ok(())
    }
    
    fn rollback(&mut self) -> Result<()> {
        // The default implementation does nothing
        // The runtime will automatically generate inverse CDC messages
        Ok(())
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
4. Generate and push CDC messages

```rust
fn process_block(&mut self) -> Result<()> {
    // Implementation
}
```

### rollback

This method is called during a reorg. It should:

1. Get the current block information
2. Generate inverse CDC messages

```rust
fn rollback(&mut self) -> Result<()> {
    // Implementation
}
```

The default implementation does nothing, as the runtime will automatically generate inverse CDC messages based on the original messages.

## Host Functions

Debshrew provides several host functions that transform modules can use:

### Block Information

- `get_height()`: Get the current block height
- `get_block_hash()`: Get the current block hash

### State Management

- `get_state(key: &[u8])`: Get a value from the state
- `set_state(key: &[u8], value: &[u8])`: Set a value in the state
- `delete_state(key: &[u8])`: Delete a value from the state

### View Functions

- `view(name: String, params: Vec<u8>)`: Call a metashrew view function

### Serialization

- `serialize_params<T: Serialize>(params: &T)`: Serialize parameters for a view function
- `deserialize_result<T: for<'de> Deserialize<'de>>(result: &[u8])`: Deserialize the result from a view function

### CDC Message Handling

- `push_cdc_message(message: &CdcMessage)`: Push a CDC message to the host

### Logging

- `write_stdout(msg: &str)`: Write to stdout
- `write_stderr(msg: &str)`: Write to stderr
- `println!()`: Macro for writing to stdout
- `eprintln!()`: Macro for writing to stderr

## State Management

Transform modules can maintain state between blocks using the state management functions. The state is:

1. Persisted between block processing
2. Rolled back during reorgs
3. Managed by the WASM runtime
4. Accessible through host functions

Example:

```rust
// Get a value from the state
let key = "count".as_bytes();
let count_bytes = get_state(key).unwrap_or_else(|| "0".as_bytes().to_vec());
let count_str = std::str::from_utf8(&count_bytes).unwrap_or("0");
let mut count: u32 = count_str.parse().unwrap_or(0);

// Update the value
count += 1;

// Store the updated value
set_state(key, count.to_string().as_bytes());
```

## Calling Metashrew Views

Transform modules can call metashrew views to get data:

```rust
// Serialize the parameters
let params = serialize_params(&height)?;

// Call the view function
let result = view("get_block_transactions".to_string(), params)?;

// Deserialize the result
let transactions: Vec<Transaction> = deserialize_result(&result)?;
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

// Push the message
self.push_message(message)?;
```

## Handling Rollbacks

During a reorg, transform modules need to generate inverse CDC messages. The default implementation does nothing, as the runtime will automatically generate inverse CDC messages based on the original messages.

If you need custom rollback behavior, you can implement the `rollback` method:

```rust
fn rollback(&mut self) -> Result<()> {
    // Get the current block height
    let height = get_height();
    
    // Get the records that were created in this block
    let records = get_records_for_block(height)?;
    
    // Generate inverse CDC messages
    for record in records {
        let message = CdcMessage {
            header: CdcHeader {
                source: "my_transform".to_string(),
                timestamp: chrono::Utc::now(),
                block_height: height,
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
        };
        
        // Push the message
        self.push_message(message)?;
    }
    
    Ok(())
}
```

## Advanced Techniques

### Batching CDC Messages

For better performance, batch CDC messages:

```rust
for tx in transactions {
    // Process transaction
    // ...
    
    // Create CDC message
    let message = CdcMessage { /* ... */ };
    
    // Push CDC message
    self.push_message(message)?;
}
```

### Incremental Processing

For large datasets, use incremental processing:

```rust
// Get the last processed transaction index
let key = "last_tx_index".as_bytes();
let last_index_bytes = get_state(key).unwrap_or_else(|| "0".as_bytes().to_vec());
let last_index_str = std::str::from_utf8(&last_index_bytes).unwrap_or("0");
let last_index: usize = last_index_str.parse().unwrap_or(0);

// Process transactions starting from the last index
for (i, tx) in transactions.iter().enumerate().skip(last_index) {
    // Process transaction
    // ...
    
    // Update the last processed index
    set_state(key, (i + 1).to_string().as_bytes());
}
```

### Error Handling

Handle errors gracefully:

```rust
fn process_block(&mut self) -> Result<()> {
    // Try to call a view function
    let result = match view("my_view".to_string(), vec![]) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error calling view: {:?}", e);
            return Ok(()); // Return Ok instead of propagating the error
        }
    };
    
    // Continue processing
    // ...
    
    Ok(())
}
```

## Debugging Transform Modules

Debugging WASM modules can be challenging. Here are some tips:

1. Use the `println!` and `eprintln!` macros to output debug information
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