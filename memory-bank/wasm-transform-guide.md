# WASM Transform Guide

This guide explains how to create WASM transform modules for debshrew.

## Overview

Transform modules are WebAssembly (WASM) modules that implement the `DebTransform` trait. They are responsible for:

1. Processing blocks by querying metashrew views
2. Detecting changes in metaprotocol state
3. Generating CDC messages for those changes
4. Handling rollbacks during chain reorganizations

## Creating a Transform Module

### Project Setup

Create a new Rust crate with the following configuration:

```toml
[package]
name = "my-transform"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
debshrew-runtime = { path = "/path/to/debshrew-runtime" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Implementing the DebTransform Trait

Your transform module must implement the `DebTransform` trait:

```rust
use debshrew_runtime::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug)]
pub struct MyTransform {
    // State fields
}

impl DebTransform for MyTransform {
    fn process_block(&mut self) -> Result<()> {
        // Process the current block
        // Generate CDC messages
        // Update state
        Ok(())
    }
    
    // Optional: Custom rollback implementation
    // If not implemented, the runtime will automatically generate inverse operations
    fn rollback(&mut self) -> Result<()> {
        // Custom rollback logic
        Ok(())
    }
}

// Register the transform
declare_transform!(MyTransform);
```

## Host Functions

The debshrew-runtime provides several host functions that your transform module can use:

### Block Information

```rust
// Get the current block height
let height = get_height();

// Get the current block hash
let hash = get_block_hash();
```

### State Management

```rust
// Get a value from the transform state
let value = get_state(key);

// Set a value in the transform state
set_state(key, value);

// Delete a value from the transform state
delete_state(key);
```

### View Access

```rust
// Call a metashrew view
let result = view("view_name".to_string(), params);
```

### CDC Message Generation

```rust
// Create and push a CDC message
let message = CdcMessage {
    header: CdcHeader {
        source: "my_protocol".to_string(),
        timestamp: chrono::Utc::now(),
        block_height: get_height(),
        block_hash: hex::encode(&get_block_hash()),
        transaction_id: None,
    },
    payload: CdcPayload {
        operation: CdcOperation::Create,
        table: "my_table".to_string(),
        key: "my_key".to_string(),
        before: None,
        after: Some(serde_json::json!({ "field": "value" })),
    },
};

// Push the message to the host
self.push_message(message)?;
```

### Serialization Utilities

```rust
// Serialize parameters for a view function
let params = serialize_params(&my_params)?;

// Deserialize the result from a view function
let result: MyResult = deserialize_result(&view_result)?;
```

### Logging

```rust
// Log a message
println!("Processing block {}", get_height());
```

## CDC Message Operations

CDC messages have three operation types:

1. **Create**: A new record was created
   ```rust
   CdcOperation::Create
   ```

2. **Update**: An existing record was updated
   ```rust
   CdcOperation::Update
   ```

3. **Delete**: An existing record was deleted
   ```rust
   CdcOperation::Delete
   ```

## Automatic Rollback Generation

The debshrew-runtime automatically generates inverse CDC messages during rollbacks:

1. **Create → Delete**: A Create operation becomes a Delete operation
2. **Update → Update**: An Update operation becomes another Update but with before/after states swapped
3. **Delete → Create**: A Delete operation becomes a Create operation

This means you don't need to implement the `rollback` method unless you need custom rollback logic.

## Complete Example

Here's a complete example of a transform module that tracks token balances:

```rust
use debshrew_runtime::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug)]
pub struct TokenTransform {
    // State fields
}

#[derive(Serialize, Deserialize)]
struct TokenBalanceParams {
    address: String,
}

#[derive(Serialize, Deserialize)]
struct TokenBalance {
    balance: u64,
}

impl DebTransform for TokenTransform {
    fn process_block(&mut self) -> Result<()> {
        let height = get_height();
        let hash = get_block_hash();
        
        // Query metashrew views
        let params = serialize_params(&TokenBalanceParams { 
            address: "bc1q...".to_string() 
        })?;
        
        let result = view("get_token_balance".to_string(), params)?;
        let balance: TokenBalance = deserialize_result(&result)?;
        
        // Check if balance changed
        let key = format!("balance:bc1q...").into_bytes();
        if let Some(prev_data) = get_state(&key) {
            let prev_balance: TokenBalance = deserialize_result(&prev_data)?;
            
            if prev_balance.balance != balance.balance {
                // Balance changed, generate update message
                let message = CdcMessage {
                    header: CdcHeader {
                        source: "token_protocol".to_string(),
                        timestamp: chrono::Utc::now(),
                        block_height: height,
                        block_hash: hex::encode(&hash),
                        transaction_id: None,
                    },
                    payload: CdcPayload {
                        operation: CdcOperation::Update,
                        table: "balances".to_string(),
                        key: "bc1q...".to_string(),
                        before: Some(serde_json::to_value(&prev_balance)?),
                        after: Some(serde_json::to_value(&balance)?),
                    },
                };
                
                // Push CDC message
                self.push_message(message)?;
            }
        } else {
            // New balance, generate create message
            let message = CdcMessage {
                header: CdcHeader {
                    source: "token_protocol".to_string(),
                    timestamp: chrono::Utc::now(),
                    block_height: height,
                    block_hash: hex::encode(&hash),
                    transaction_id: None,
                },
                payload: CdcPayload {
                    operation: CdcOperation::Create,
                    table: "balances".to_string(),
                    key: "bc1q...".to_string(),
                    before: None,
                    after: Some(serde_json::to_value(&balance)?),
                },
            };
            
            // Push CDC message
            self.push_message(message)?;
        }
        
        // Update state
        set_state(&key, &serialize_params(&balance)?);
        
        Ok(())
    }
}

// Register the transform
declare_transform!(TokenTransform);
```

## Building and Running

Build your transform module for WebAssembly:

```bash
cargo build --target wasm32-unknown-unknown --release
```

The resulting WASM file will be in `target/wasm32-unknown-unknown/release/my_transform.wasm`.

Run debshrew with your transform module:

```bash
debshrew --metashrew-rpc-url http://localhost:8332 --transform-path target/wasm32-unknown-unknown/release/my_transform.wasm
```

## Best Practices

1. **State Management**: Store only the minimum state needed for change detection
2. **Error Handling**: Use proper error handling with the `Result` type
3. **CDC Message Design**: Design your CDC messages to be easily consumable by downstream systems
4. **Performance**: Minimize view calls and state operations for better performance
5. **Testing**: Test your transform module with different scenarios, including reorgs