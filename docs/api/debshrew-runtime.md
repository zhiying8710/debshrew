# Debshrew Runtime API Reference

This document provides a reference for the Debshrew Runtime API.

## DebTransform

The `DebTransform` trait is the core of a transform module. It defines the interface for processing blocks and handling rollbacks.

```rust
pub trait DebTransform: Default + Debug {
    fn process_block(&mut self) -> Result<Vec<CdcMessage>>;
    fn rollback(&mut self) -> Result<Vec<CdcMessage>>;
}
```

### Methods

#### process_block

```rust
fn process_block(&mut self) -> Result<Vec<CdcMessage>>
```

Processes a block and generates CDC messages.

#### rollback

```rust
fn rollback(&mut self) -> Result<Vec<CdcMessage>>
```

Handles a rollback and generates inverse CDC messages.

## WasmRuntime

The `WasmRuntime` is responsible for executing WASM transform modules.

### Constructor

```rust
pub fn new<P: AsRef<Path>>(wasm_path: P) -> Result<Self>
```

Creates a new `WasmRuntime` from a WASM file.

### Methods

#### from_bytes

```rust
pub fn from_bytes(wasm_bytes: &[u8]) -> Result<Self>
```

Creates a new `WasmRuntime` from WASM bytes.

#### set_current_height

```rust
pub fn set_current_height(&mut self, height: u32)
```

Sets the current block height.

#### set_current_hash

```rust
pub fn set_current_hash(&mut self, hash: Vec<u8>)
```

Sets the current block hash.

#### set_state

```rust
pub fn set_state(&mut self, state: TransformState)
```

Sets the transform state.

#### get_state

```rust
pub fn get_state(&self) -> TransformState
```

Gets the transform state.

#### process_block

```rust
pub fn process_block(&mut self, height: u32, hash: Vec<u8>) -> Result<TransformResult>
```

Processes a block and returns the result.

#### rollback

```rust
pub fn rollback(&mut self, height: u32, hash: Vec<u8>) -> Result<TransformResult>
```

Handles a rollback and returns the result.

#### register_view_function

```rust
pub fn register_view_function(
    &self,
    name: &str,
    func: Box<dyn Fn(&[u8]) -> Result<Vec<u8>> + Send>
)
```

Registers a view function.

## TransformResult

The `TransformResult` struct represents the result of processing a block or handling a rollback.

```rust
pub struct TransformResult {
    pub cdc_messages: Vec<CdcMessage>,
    pub state_snapshot: TransformState,
}
```

### Constructor

```rust
pub fn new(cdc_messages: Vec<CdcMessage>, state_snapshot: TransformState) -> Self
```

Creates a new `TransformResult`.

## Host Functions

These functions are available to transform modules through the WASM host interface.

### Block Information

#### get_height

```rust
pub fn get_height() -> u32
```

Gets the current block height.

#### get_block_hash

```rust
pub fn get_block_hash() -> Vec<u8>
```

Gets the current block hash.

### State Management

#### get_state

```rust
pub fn get_state(key: &str) -> Option<String>
```

Gets a value from the state.

#### set_state

```rust
pub fn set_state(key: &str, value: &str)
```

Sets a value in the state.

#### delete_state

```rust
pub fn delete_state(key: &str)
```

Deletes a value from the state.

#### get_state_keys

```rust
pub fn get_state_keys() -> Vec<String>
```

Gets all keys in the state.

#### get_state_keys_with_prefix

```rust
pub fn get_state_keys_with_prefix(prefix: &str) -> Vec<String>
```

Gets all keys with a specific prefix.

### View Functions

#### call_view

```rust
pub fn call_view(name: &str, params: &[u8]) -> Result<Vec<u8>>
```

Calls a metashrew view function.

### Serialization

#### serialize

```rust
pub fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>>
```

Serializes a value to bytes.

#### deserialize

```rust
pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T>
```

Deserializes bytes to a value.

#### serialize_to_json

```rust
pub fn serialize_to_json<T: Serialize>(value: &T) -> Result<String>
```

Serializes a value to JSON.

### Logging

#### log

```rust
pub fn log(message: &str)
```

Logs a message.

## declare_transform

The `declare_transform` macro is used to declare a transform module.

```rust
macro_rules! declare_transform {
    ($transform_type:ty) => {
        // Implementation
    };
}
```

### Example

```rust
use debshrew_runtime::*;

#[derive(Debug, Default)]
struct MyTransform {
    state: TransformState
}

impl DebTransform for MyTransform {
    fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
        // Implementation
    }
    
    fn rollback(&mut self) -> Result<Vec<CdcMessage>> {
        // Implementation
    }
}

// Declare the transform module
declare_transform!(MyTransform);
```

## Error Handling

### Error

The `Error` enum represents errors that can occur in the Debshrew Runtime.

```rust
pub enum Error {
    Wasm(String),
    Serialization(String),
    View(String),
    Io(std::io::Error),
}
```

### Result

The `Result` type is a shorthand for `std::result::Result<T, Error>`.

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

## Testing

### MockTransform

The `MockTransform` struct is a mock implementation of the `DebTransform` trait for testing.

```rust
pub struct MockTransform {
    pub state: TransformState,
    pub process_block_messages: Vec<CdcMessage>,
    pub rollback_messages: Vec<CdcMessage>,
}
```

#### Default

```rust
impl Default for MockTransform
```

Creates a new `MockTransform` with default values.

#### DebTransform

```rust
impl DebTransform for MockTransform
```

Implements the `DebTransform` trait for `MockTransform`.