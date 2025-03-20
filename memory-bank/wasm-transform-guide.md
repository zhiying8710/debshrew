# WASM Transform Development Guide

This document provides a comprehensive guide to developing WebAssembly (WASM) transform modules for the debshrew project, covering key concepts, patterns, and best practices.

## Overview

WASM transform modules are the core of debshrew's extensibility. They implement the logic for transforming metaprotocol state from metashrew into standardized Change Data Capture (CDC) streams. This guide will help you understand how to develop effective transform modules.

## Transform Module Interface

Transform modules implement the `DebTransform` trait:

```rust
pub trait DebTransform {
    fn process_block(&mut self) -> Result<Vec<CdcMessage>>;
    fn rollback(&mut self) -> Result<Vec<CdcMessage>>;
}
```

### `process_block`

The `process_block` method is called for each new block. It should:

1. Query metashrew views to get metaprotocol state
2. Compare with previous state to detect changes
3. Generate CDC messages for the changes
4. Update internal state

### `rollback`

The `rollback` method is called during chain reorganizations. It should:

1. Revert to the previous state
2. Generate inverse CDC messages that undo the changes from the rolled-back blocks

## Host Functions

The debshrew runtime provides the following host functions to transform modules:

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

### View Access Functions

- `call_view`: Calls a metashrew view function to query metaprotocol state
- `get_height`: Gets the current block height
- `get_block_hash`: Gets the current block hash

### State Management Functions

- `get_state`: Gets a value from the transform state
- `set_state`: Sets a value in the transform state
- `delete_state`: Deletes a value from the transform state

### Logging Functions

- `log`: Writes a message to the log

## Transform Module Structure

A typical transform module has the following structure:

```rust
use debshrew_runtime::*;
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};

#[derive(Default)]
struct ExampleTransform {
    state: TransformState
}

// Parameters for metashrew view functions
#[derive(Serialize, Deserialize)]
struct ExampleParams {
    // Parameters for the view function
}

// Result from metashrew view functions
#[derive(Serialize, Deserialize)]
struct ExampleViewResult {
    // Result from the view function
}

impl DebTransform for ExampleTransform {
    fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
        // Get current block info
        let height = get_height();
        let hash = get_block_hash();
        
        // Query metashrew views
        let params = serialize_params(&ExampleParams { /* ... */ })?;
        let result = call_view("example_view", &params)?;
        let view_data: ExampleViewResult = deserialize_result(&result)?;
        
        // Process state and generate CDC messages
        let mut messages = Vec::new();
        
        // Compare with previous state and generate CDC messages
        // ...
        
        // Update state
        // ...
        
        Ok(messages)
    }
    
    fn rollback(&mut self) -> Result<Vec<CdcMessage>> {
        // Generate inverse operations for rollback
        // ...
        
        Ok(messages)
    }
}

// Register the transform module
declare_transform!(ExampleTransform);
```

## State Management

Transform modules maintain state to track changes between blocks. The state is a key-value store that persists between block processing.

### State Structure

The state is typically organized by entity type and identifier:

```
entity_type:entity_id -> serialized_entity_data
```

For example:

```
token:BTC -> { "supply": 21000000, "holders": 1000000 }
balance:alice:BTC -> 100
balance:bob:BTC -> 200
```

### State Operations

```rust
// Get a value from state
let key = format!("balance:{}:{}", address, token).into_bytes();
let previous = self.state.get(&key)?;

// Set a value in state
self.state.set(&key, &serialize(&balance)?)?;

// Delete a value from state
self.state.delete(&key)?;

// Iterate over keys with a prefix
let prefix = "balance:".as_bytes();
for key in self.state.keys_with_prefix(prefix)? {
    // Process each key
}
```

### State Serialization

State values must be serialized and deserialized. Common formats include:

- **JSON**: Human-readable but less efficient
- **Bincode**: Efficient binary format for Rust types
- **Protocol Buffers**: Efficient binary format with schema evolution
- **MessagePack**: Compact binary format with good language support

Example using serde and bincode:

```rust
fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    bincode::serialize(value).map_err(|e| anyhow!("Serialization error: {}", e))
}

fn deserialize<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Result<T> {
    bincode::deserialize(data).map_err(|e| anyhow!("Deserialization error: {}", e))
}
```

## CDC Message Generation

Transform modules generate CDC messages for state changes. Each message represents a create, update, or delete operation.

### Message Structure

```rust
pub struct CdcMessage {
    pub header: CdcHeader,
    pub payload: CdcPayload,
}

pub struct CdcHeader {
    pub source: String,
    pub timestamp: u64,
    pub block_height: u32,
    pub block_hash: String,
    pub transaction_id: Option<String>,
}

pub struct CdcPayload {
    pub operation: CdcOperation,
    pub table: String,
    pub key: String,
    pub before: Option<serde_json::Value>,
    pub after: Option<serde_json::Value>,
}

pub enum CdcOperation {
    Create,
    Update,
    Delete,
}
```

### Message Generation Patterns

#### Create Operation

```rust
CdcMessage {
    header: CdcHeader {
        source: "example_protocol".to_string(),
        timestamp: now_ms(),
        block_height: height,
        block_hash: hex::encode(hash),
        transaction_id: None,
    },
    payload: CdcPayload {
        operation: CdcOperation::Create,
        table: "items".to_string(),
        key: item.id.to_string(),
        before: None,
        after: Some(serialize_to_json(&item)?),
    },
}
```

#### Update Operation

```rust
CdcMessage {
    header: CdcHeader {
        source: "example_protocol".to_string(),
        timestamp: now_ms(),
        block_height: height,
        block_hash: hex::encode(hash),
        transaction_id: None,
    },
    payload: CdcPayload {
        operation: CdcOperation::Update,
        table: "items".to_string(),
        key: item.id.to_string(),
        before: Some(serialize_to_json(&prev_item)?),
        after: Some(serialize_to_json(&item)?),
    },
}
```

#### Delete Operation

```rust
CdcMessage {
    header: CdcHeader {
        source: "example_protocol".to_string(),
        timestamp: now_ms(),
        block_height: height,
        block_hash: hex::encode(hash),
        transaction_id: None,
    },
    payload: CdcPayload {
        operation: CdcOperation::Delete,
        table: "items".to_string(),
        key: item.id.to_string(),
        before: Some(serialize_to_json(&prev_item)?),
        after: None,
    },
}
```

## Calling Metashrew Views

Transform modules call metashrew view functions to query metaprotocol state.

### View Function Parameters

Parameters for view functions are serialized to a binary format:

```rust
#[derive(Serialize, Deserialize)]
struct TokenBalanceParams {
    address: String,
    token: String,
}

let params = serialize_params(&TokenBalanceParams {
    address: "bc1q...".to_string(),
    token: "BTC".to_string(),
})?;

let result = call_view("get_token_balance", &params)?;
```

### View Function Results

Results from view functions are deserialized from a binary format:

```rust
#[derive(Serialize, Deserialize)]
struct TokenBalance {
    balance: u64,
    last_updated: u32,
}

let balance: TokenBalance = deserialize_result(&result)?;
```

### Common View Function Patterns

#### Get All Entities

```rust
fn get_all_tokens(&mut self) -> Result<HashMap<String, Token>> {
    let params = serialize_params(&EmptyParams {})?;
    let result = call_view("get_all_tokens", &params)?;
    deserialize_result(&result)
}
```

#### Get Entity by ID

```rust
fn get_token(&mut self, token_id: &str) -> Result<Option<Token>> {
    let params = serialize_params(&TokenParams { token_id: token_id.to_string() })?;
    let result = call_view("get_token", &params)?;
    deserialize_result(&result)
}
```

#### Get Entities by Filter

```rust
fn get_tokens_by_issuer(&mut self, issuer: &str) -> Result<Vec<Token>> {
    let params = serialize_params(&IssuerParams { issuer: issuer.to_string() })?;
    let result = call_view("get_tokens_by_issuer", &params)?;
    deserialize_result(&result)
}
```

## Reorg Handling

Transform modules must handle chain reorganizations by implementing the `rollback` method.

### Rollback Strategy

1. **State-Based Rollback**: Generate inverse operations based on the current state
2. **Log-Based Rollback**: Keep a log of operations and generate inverse operations from the log

### State-Based Rollback Example

```rust
fn rollback(&mut self) -> Result<Vec<CdcMessage>> {
    let height = get_height();
    let hash = get_block_hash();
    
    // Query current state from metashrew
    let params = serialize_params(&ExampleParams { /* ... */ })?;
    let result = call_view("example_view", &params)?;
    let current_state: ExampleViewResult = deserialize_result(&result)?;
    
    let mut messages = Vec::new();
    
    // Generate inverse operations based on the difference between current state and stored state
    // ...
    
    // Update state to match current state
    // ...
    
    Ok(messages)
}
```

### Log-Based Rollback Example

```rust
fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
    // ... normal processing ...
    
    // Log the operations for this block
    let block_log_key = format!("block_log:{}", get_height()).into_bytes();
    self.state.set(&block_log_key, &serialize(&messages)?)?;
    
    Ok(messages)
}

fn rollback(&mut self) -> Result<Vec<CdcMessage>> {
    let height = get_height();
    
    // Get the log for the block being rolled back
    let block_log_key = format!("block_log:{}", height + 1).into_bytes();
    let log_data = self.state.get(&block_log_key)?
        .ok_or_else(|| anyhow!("Missing log for block {}", height + 1))?;
    let log_messages: Vec<CdcMessage> = deserialize(&log_data)?;
    
    // Generate inverse operations
    let mut inverse_messages = Vec::new();
    for message in log_messages.iter().rev() {
        let inverse = match message.payload.operation {
            CdcOperation::Create => CdcMessage {
                header: CdcHeader {
                    source: message.header.source.clone(),
                    timestamp: now_ms(),
                    block_height: height,
                    block_hash: hex::encode(get_block_hash()),
                    transaction_id: None,
                },
                payload: CdcPayload {
                    operation: CdcOperation::Delete,
                    table: message.payload.table.clone(),
                    key: message.payload.key.clone(),
                    before: message.payload.after.clone(),
                    after: None,
                },
            },
            CdcOperation::Update => CdcMessage {
                header: CdcHeader {
                    source: message.header.source.clone(),
                    timestamp: now_ms(),
                    block_height: height,
                    block_hash: hex::encode(get_block_hash()),
                    transaction_id: None,
                },
                payload: CdcPayload {
                    operation: CdcOperation::Update,
                    table: message.payload.table.clone(),
                    key: message.payload.key.clone(),
                    before: message.payload.after.clone(),
                    after: message.payload.before.clone(),
                },
            },
            CdcOperation::Delete => CdcMessage {
                header: CdcHeader {
                    source: message.header.source.clone(),
                    timestamp: now_ms(),
                    block_height: height,
                    block_hash: hex::encode(get_block_hash()),
                    transaction_id: None,
                },
                payload: CdcPayload {
                    operation: CdcOperation::Create,
                    table: message.payload.table.clone(),
                    key: message.payload.key.clone(),
                    before: None,
                    after: message.payload.before.clone(),
                },
            },
        };
        
        inverse_messages.push(inverse);
    }
    
    // Remove the log for the rolled back block
    self.state.delete(&block_log_key)?;
    
    Ok(inverse_messages)
}
```

## Error Handling

Transform modules should implement robust error handling to ensure reliability.

### Error Propagation

Use the `anyhow` crate for error propagation:

```rust
use anyhow::{Result, anyhow, Context};

fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
    let params = serialize_params(&ExampleParams { /* ... */ })
        .context("Failed to serialize parameters")?;
    
    let result = call_view("example_view", &params)
        .context("Failed to call view function")?;
    
    let view_data: ExampleViewResult = deserialize_result(&result)
        .context("Failed to deserialize view result")?;
    
    // ...
    
    Ok(messages)
}
```

### Error Recovery

Implement recovery strategies for non-fatal errors:

```rust
fn get_token(&mut self, token_id: &str) -> Result<Option<Token>> {
    let params = serialize_params(&TokenParams { token_id: token_id.to_string() })?;
    
    match call_view("get_token", &params) {
        Ok(result) => {
            match deserialize_result(&result) {
                Ok(token) => Ok(Some(token)),
                Err(e) => {
                    log(&format!("Error deserializing token {}: {}", token_id, e));
                    Ok(None)
                }
            }
        },
        Err(e) => {
            log(&format!("Error calling get_token for {}: {}", token_id, e));
            Ok(None)
        }
    }
}
```

## Performance Optimization

Transform modules should be optimized for performance to handle high throughput.

### Minimize View Calls

Reduce the number of view calls by batching queries:

```rust
// Inefficient: Multiple view calls
for token_id in token_ids {
    let token = self.get_token(token_id)?;
    // Process token
}

// Efficient: Single view call
let tokens = self.get_tokens_by_ids(&token_ids)?;
for token in tokens {
    // Process token
}
```

### Efficient State Management

Use efficient state access patterns:

```rust
// Inefficient: Multiple state lookups
for token_id in token_ids {
    let key = format!("token:{}", token_id).into_bytes();
    let token_data = self.state.get(&key)?;
    // Process token
}

// Efficient: Batch state lookups with prefix
let prefix = "token:".as_bytes();
let token_data = self.state.get_with_prefix(prefix)?;
for (key, data) in token_data {
    // Process token
}
```

### Optimize Serialization

Use efficient serialization formats:

```rust
// Less efficient: JSON serialization
let json = serde_json::to_string(&token)?;
self.state.set(&key, json.as_bytes())?;

// More efficient: Binary serialization
let binary = bincode::serialize(&token)?;
self.state.set(&key, &binary)?;
```

### Incremental Processing

Only process changed entities:

```rust
fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
    // Get current state
    let current_state = self.get_current_state()?;
    
    // Get previous state
    let previous_state = self.get_previous_state()?;
    
    // Find changed entities
    let changed_entities = self.find_changed_entities(&previous_state, &current_state)?;
    
    // Process only changed entities
    let mut messages = Vec::new();
    for entity in changed_entities {
        // Generate CDC messages for changed entity
    }
    
    Ok(messages)
}
```

## Testing Transform Modules

Transform modules should be thoroughly tested to ensure correctness.

### Unit Testing

Test individual functions:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_serialize_deserialize() {
        let token = Token {
            id: "BTC".to_string(),
            supply: 21000000,
        };
        
        let serialized = serialize(&token).unwrap();
        let deserialized: Token = deserialize(&serialized).unwrap();
        
        assert_eq!(token.id, deserialized.id);
        assert_eq!(token.supply, deserialized.supply);
    }
    
    #[test]
    fn test_generate_cdc_message() {
        let transform = ExampleTransform::default();
        let message = transform.generate_create_message("BTC", &Token {
            id: "BTC".to_string(),
            supply: 21000000,
        }, 100, &[1, 2, 3]).unwrap();
        
        assert_eq!(message.payload.operation, CdcOperation::Create);
        assert_eq!(message.payload.table, "tokens");
        assert_eq!(message.payload.key, "BTC");
        assert!(message.payload.before.is_none());
        assert!(message.payload.after.is_some());
    }
}
```

### Integration Testing

Test the entire transform module:

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use debshrew_runtime::testing::*;
    
    #[test]
    fn test_process_block() {
        let mut mock_runtime = MockRuntime::new();
        mock_runtime.set_height(100);
        mock_runtime.set_block_hash([1, 2, 3].to_vec());
        
        // Mock view function results
        mock_runtime.mock_view_result("get_all_tokens", &[], serialize(&vec![
            Token { id: "BTC".to_string(), supply: 21000000 },
            Token { id: "ETH".to_string(), supply: 100000000 },
        ]).unwrap());
        
        let mut transform = ExampleTransform::default();
        let messages = transform.process_block().unwrap();
        
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].payload.operation, CdcOperation::Create);
        assert_eq!(messages[0].payload.key, "BTC");
        assert_eq!(messages[1].payload.operation, CdcOperation::Create);
        assert_eq!(messages[1].payload.key, "ETH");
    }
    
    #[test]
    fn test_rollback() {
        let mut mock_runtime = MockRuntime::new();
        mock_runtime.set_height(99);
        mock_runtime.set_block_hash([4, 5, 6].to_vec());
        
        // Set up state as if block 100 was processed
        let mut transform = ExampleTransform::default();
        transform.state.set("token:BTC".as_bytes(), &serialize(&Token {
            id: "BTC".to_string(),
            supply: 21000000,
        }).unwrap()).unwrap();
        
        // Mock view function results for the rolled back state
        mock_runtime.mock_view_result("get_all_tokens", &[], serialize(&vec![
            // Empty state after rollback
        ]).unwrap());
        
        let messages = transform.rollback().unwrap();
        
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].payload.operation, CdcOperation::Delete);
        assert_eq!(messages[0].payload.key, "BTC");
    }
}
```

### Property-Based Testing

Test with randomly generated inputs:

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_serialize_deserialize_roundtrip(
            id in "[A-Z]{3,5}",
            supply in 1..1000000000u64
        ) {
            let token = Token {
                id: id.clone(),
                supply,
            };
            
            let serialized = serialize(&token).unwrap();
            let deserialized: Token = deserialize(&serialized).unwrap();
            
            prop_assert_eq!(token.id, deserialized.id);
            prop_assert_eq!(token.supply, deserialized.supply);
        }
        
        #[test]
        fn test_inverse_operations(
            id in "[A-Z]{3,5}",
            old_supply in 1..1000000000u64,
            new_supply in 1..1000000000u64
        ) {
            let old_token = Token {
                id: id.clone(),
                supply: old_supply,
            };
            
            let new_token = Token {
                id: id.clone(),
                supply: new_supply,
            };
            
            let transform = ExampleTransform::default();
            
            // Generate update message
            let update = transform.generate_update_message(
                &id,
                &old_token,
                &new_token,
                100,
                &[1, 2, 3]
            ).unwrap();
            
            // Generate inverse operation
            let inverse = transform.generate_inverse_message(&update, 99, &[4, 5, 6]).unwrap();
            
            // Verify inverse operation
            prop_assert_eq!(inverse.payload.operation, CdcOperation::Update);
            prop_assert_eq!(inverse.payload.key, id);
            
            let before = inverse.payload.before.as_ref().unwrap();
            let after = inverse.payload.after.as_ref().unwrap();
            
            let before_supply = before.get("supply").unwrap().as_u64().unwrap();
            let after_supply = after.get("supply").unwrap().as_u64().unwrap();
            
            prop_assert_eq!(before_supply, new_supply);
            prop_assert_eq!(after_supply, old_supply);
        }
    }
}
```

## Debugging Transform Modules

Debugging WASM modules can be challenging. Here are some strategies:

### Logging

Use the `log` function for debugging:

```rust
fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
    log("Starting process_block");
    
    let height = get_height();
    log(&format!("Processing block at height {}", height));
    
    // ...
    
    log(&format!("Generated {} CDC messages", messages.len()));
    Ok(messages)
}
```

### State Inspection

Dump state for debugging:

```rust
fn dump_state(&self) -> Result<()> {
    log("Current state:");
    
    for key in self.state.keys()? {
        if let Ok(key_str) = String::from_utf8(key.clone()) {
            let value = self.state.get(&key)?;
            if let Some(data) = value {
                log(&format!("  {}: {} bytes", key_str, data.len()));
            } else {
                log(&format!("  {}: null", key_str));
            }
        }
    }
    
    Ok(())
}
```

### Mock Runtime

Use a mock runtime for testing:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use debshrew_runtime::testing::*;
    
    #[test]
    fn test_with_mock_runtime() {
        let mut mock_runtime = MockRuntime::new();
        
        // Set up mock state
        mock_runtime.set_state("token:BTC".as_bytes(), &serialize(&Token {
            id: "BTC".to_string(),
            supply: 21000000,
        }).unwrap());
        
        // Set up mock view results
        mock_runtime.mock_view_result("get_token", &serialize(&TokenParams {
            token_id: "BTC".to_string(),
        }).unwrap(), serialize(&Token {
            id: "BTC".to_string(),
            supply: 21000000,
        }).unwrap());
        
        // Run with mock runtime
        let mut transform = ExampleTransform::default();
        let result = mock_runtime.run(&mut transform);
        
        // Verify results
        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 0); // No changes
        
        // Verify logs
        let logs = mock_runtime.get_logs();
        assert!(logs.contains(&"Starting process_block".to_string()));
    }
}
```

## Conclusion

Developing WASM transform modules for debshrew requires understanding the transform interface, host functions, state management, CDC message generation, and reorg handling. By following the patterns and best practices outlined in this guide, you can create robust, efficient transform modules that generate high-quality CDC streams from metaprotocol state.

Remember that transform modules should be:

1. **Deterministic**: Produce the same output for the same input
2. **Efficient**: Minimize resource usage and optimize performance
3. **Robust**: Handle errors and edge cases gracefully
4. **Testable**: Include comprehensive tests for correctness
5. **Maintainable**: Follow good coding practices and documentation

With these principles in mind, you can develop transform modules that effectively bridge the gap between metaprotocol state and standardized CDC streams.