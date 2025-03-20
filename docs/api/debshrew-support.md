# Debshrew Support API Reference

This document provides a reference for the Debshrew Support API.

## CDC Messages

### CdcMessage

The `CdcMessage` struct represents a CDC message.

```rust
pub struct CdcMessage {
    pub header: CdcHeader,
    pub payload: CdcPayload,
}
```

### CdcHeader

The `CdcHeader` struct represents the header of a CDC message.

```rust
pub struct CdcHeader {
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub block_height: u32,
    pub block_hash: String,
    pub transaction_id: Option<String>,
}
```

### CdcPayload

The `CdcPayload` struct represents the payload of a CDC message.

```rust
pub struct CdcPayload {
    pub operation: CdcOperation,
    pub table: String,
    pub key: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
}
```

### CdcOperation

The `CdcOperation` enum represents the type of operation in a CDC message.

```rust
pub enum CdcOperation {
    Create,
    Update,
    Delete,
}
```

## Block Metadata

### BlockMetadata

The `BlockMetadata` struct represents metadata about a block.

```rust
pub struct BlockMetadata {
    pub height: u32,
    pub hash: String,
    pub timestamp: DateTime<Utc>,
}
```

## State Management

### TransformState

The `TransformState` struct represents the state of a transform module.

```rust
pub struct TransformState {
    // Internal implementation
}
```

#### Constructor

```rust
pub fn new() -> Self
```

Creates a new `TransformState`.

#### Methods

##### get

```rust
pub fn get(&self, key: &str) -> Option<String>
```

Gets a value from the state.

##### set

```rust
pub fn set(&mut self, key: String, value: String)
```

Sets a value in the state.

##### delete

```rust
pub fn delete(&mut self, key: &str)
```

Deletes a value from the state.

##### keys

```rust
pub fn keys(&self) -> Vec<String>
```

Gets all keys in the state.

##### keys_with_prefix

```rust
pub fn keys_with_prefix(&self, prefix: &str) -> Vec<String>
```

Gets all keys with a specific prefix.

##### iter

```rust
pub fn iter(&self) -> impl Iterator<Item = (&String, &String)>
```

Returns an iterator over the state.

##### is_empty

```rust
pub fn is_empty(&self) -> bool
```

Checks if the state is empty.

##### len

```rust
pub fn len(&self) -> usize
```

Gets the number of entries in the state.

##### clear

```rust
pub fn clear(&mut self)
```

Clears the state.

## Serialization

### serialize

```rust
pub fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>>
```

Serializes a value to bytes using bincode.

### deserialize

```rust
pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T>
```

Deserializes bytes to a value using bincode.

### serialize_to_json

```rust
pub fn serialize_to_json<T: Serialize>(value: &T) -> Result<String>
```

Serializes a value to JSON.

### deserialize_from_json

```rust
pub fn deserialize_from_json<T: DeserializeOwned>(json: &str) -> Result<T>
```

Deserializes JSON to a value.

## Utility Functions

### now_ms

```rust
pub fn now_ms() -> u64
```

Gets the current timestamp in milliseconds.

### now_utc

```rust
pub fn now_utc() -> DateTime<Utc>
```

Gets the current timestamp as a UTC DateTime.

### timestamp_ms_to_datetime

```rust
pub fn timestamp_ms_to_datetime(timestamp_ms: u64) -> DateTime<Utc>
```

Converts a Unix timestamp in milliseconds to a UTC DateTime.

### datetime_to_timestamp_ms

```rust
pub fn datetime_to_timestamp_ms(dt: &DateTime<Utc>) -> u64
```

Converts a UTC DateTime to a Unix timestamp in milliseconds.

### is_valid_block_hash

```rust
pub fn is_valid_block_hash(hash: &str) -> bool
```

Validates a Bitcoin block hash.

### is_valid_txid

```rust
pub fn is_valid_txid(txid: &str) -> bool
```

Validates a Bitcoin transaction ID.

### generate_cdc_message_id

```rust
pub fn generate_cdc_message_id(source: &str, table: &str, key: &str, block_height: u32) -> String
```

Generates a unique ID for a CDC message.

### truncate_string

```rust
pub fn truncate_string(s: &str, max_len: usize) -> String
```

Truncates a string to a maximum length with ellipsis.

### parse_url

```rust
pub fn parse_url(url_str: &str) -> Result<url::Url>
```

Parses a URL and validates it.

## Error Handling

### Error

The `Error` enum represents errors that can occur in the Debshrew Support library.

```rust
pub enum Error {
    Serialization(String),
    Validation(String),
    Io(std::io::Error),
}
```

### Result

The `Result` type is a shorthand for `std::result::Result<T, Error>`.

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

## Examples

### Creating a CDC Message

```rust
use debshrew_support::{CdcHeader, CdcMessage, CdcOperation, CdcPayload};
use chrono::Utc;
use serde_json::json;

let message = CdcMessage {
    header: CdcHeader {
        source: "my_transform".to_string(),
        timestamp: Utc::now(),
        block_height: 123456,
        block_hash: "000000000000000000024bead8df69990852c202db0e0097c1a12ea637d7e96d".to_string(),
        transaction_id: None,
    },
    payload: CdcPayload {
        operation: CdcOperation::Create,
        table: "my_table".to_string(),
        key: "my_key".to_string(),
        before: None,
        after: Some(json!({
            "field1": "value1",
            "field2": 42
        })),
    },
};
```

### Using TransformState

```rust
use debshrew_support::TransformState;

let mut state = TransformState::new();

// Set a value
state.set("count".to_string(), "42".to_string());

// Get a value
let count = state.get("count").unwrap_or("0".to_string());

// Delete a value
state.delete("count");

// Check if the state is empty
if state.is_empty() {
    println!("State is empty");
}
```

### Serialization

```rust
use debshrew_support::{serialize, deserialize};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct MyStruct {
    field1: String,
    field2: i32,
}

let my_struct = MyStruct {
    field1: "value1".to_string(),
    field2: 42,
};

// Serialize
let bytes = serialize(&my_struct).unwrap();

// Deserialize
let deserialized: MyStruct = deserialize(&bytes).unwrap();
```

### Utility Functions

```rust
use debshrew_support::utils::{now_ms, timestamp_ms_to_datetime, is_valid_block_hash};
use chrono::Datelike;

// Get current timestamp
let now = now_ms();

// Convert to DateTime
let dt = timestamp_ms_to_datetime(now);
println!("Year: {}, Month: {}, Day: {}", dt.year(), dt.month(), dt.day());

// Validate a block hash
let hash = "000000000000000000024bead8df69990852c202db0e0097c1a12ea637d7e96d";
if is_valid_block_hash(hash) {
    println!("Valid block hash");
}