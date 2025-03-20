# CDC (Change Data Capture) Concepts and Best Practices

This document provides a comprehensive overview of Change Data Capture (CDC) concepts, patterns, and best practices as they apply to the debshrew project.

## What is Change Data Capture (CDC)?

Change Data Capture (CDC) is a set of software design patterns used to determine and track changes made to data so that actions can be taken using the changed data. CDC captures changes made to a data source and delivers those changes to a target system in real-time or near real-time.

In the context of debshrew, CDC is used to capture changes in metaprotocol state from metashrew and deliver those changes to downstream systems in a standardized format.

## Core CDC Concepts

### 1. Event Types

CDC systems typically capture the following types of events:

- **Create**: A new record has been created
- **Update**: An existing record has been modified
- **Delete**: An existing record has been removed
- **Truncate**: All records have been removed (less common)

### 2. Event Structure

CDC events typically include:

- **Metadata**: Information about the event itself (timestamp, source, etc.)
- **Before State**: The state of the record before the change (for updates and deletes)
- **After State**: The state of the record after the change (for creates and updates)
- **Operation Type**: The type of operation (create, update, delete)
- **Identifier**: A unique identifier for the affected record

### 3. Delivery Guarantees

CDC systems provide different delivery guarantees:

- **At-least-once delivery**: Events may be delivered multiple times, but will never be lost
- **Exactly-once delivery**: Events are delivered exactly once (harder to achieve)
- **In-order delivery**: Events are delivered in the same order they occurred

### 4. Consistency Models

CDC systems must handle consistency challenges:

- **Transactional Consistency**: Changes from a single transaction are captured together
- **Causal Consistency**: Related changes are captured in the correct order
- **Global Consistency**: All changes across the system are captured in a consistent order

## Debshrew CDC Implementation

### CDC Message Format

Debshrew uses a Debezium-compatible CDC message format:

```json
{
  "header": {
    "source": "debshrew",
    "timestamp": 1647123456789,
    "block_height": 123456,
    "block_hash": "000000000000000000024bead8df69990852c202db0e0097c1a12ea637d7e96d",
    "transaction_id": "tx_id_or_null_for_block_level_events"
  },
  "payload": {
    "operation": "create", // create, update, delete
    "table": "table_name",
    "key": "record_key",
    "before": null, // null for create operations
    "after": {
      // Record data after the operation
      "field1": "value1",
      "field2": "value2"
    }
  }
}
```

### CDC Generation Process

1. **State Tracking**: Transform modules track metaprotocol state changes
2. **Diff Detection**: Changes are detected by comparing current state with previous state
3. **Message Generation**: CDC messages are generated for each detected change
4. **Delivery**: Messages are delivered to configured sinks

### Reorg Handling

Debshrew handles blockchain reorganizations (reorgs) by:

1. **Detecting Reorgs**: Comparing block hashes with metashrew
2. **Rolling Back State**: Reverting to a previous state snapshot
3. **Generating Inverse Operations**: Creating CDC messages that undo previous changes
4. **Processing New Chain**: Generating CDC messages for the new chain

## CDC Sink Types

Debshrew supports multiple CDC sink types:

### 1. Kafka Sink

Sends CDC messages to Kafka topics:

```rust
pub struct KafkaSink {
    producer: KafkaProducer,
    topic: String,
    config: KafkaSinkConfig,
}

impl CdcSink for KafkaSink {
    fn send(&mut self, messages: Vec<CdcMessage>) -> Result<()> {
        for message in messages {
            let key = message.payload.key.clone();
            let value = serde_json::to_string(&message)?;
            self.producer.send(
                FutureRecord::to(&self.topic)
                    .key(&key)
                    .payload(&value),
                Duration::from_secs(self.config.timeout_seconds),
            )?;
        }
        Ok(())
    }
    
    fn flush(&mut self) -> Result<()> {
        self.producer.flush(Duration::from_secs(self.config.timeout_seconds))?;
        Ok(())
    }
}
```

### 2. PostgreSQL Sink

Applies CDC messages directly to a PostgreSQL database:

```rust
pub struct PostgresSink {
    client: Client,
    config: PostgresSinkConfig,
}

impl CdcSink for PostgresSink {
    fn send(&mut self, messages: Vec<CdcMessage>) -> Result<()> {
        let mut transaction = self.client.transaction()?;
        
        for message in messages {
            match message.payload.operation {
                CdcOperation::Create => {
                    // Execute INSERT statement
                },
                CdcOperation::Update => {
                    // Execute UPDATE statement
                },
                CdcOperation::Delete => {
                    // Execute DELETE statement
                },
            }
        }
        
        transaction.commit()?;
        Ok(())
    }
    
    fn flush(&mut self) -> Result<()> {
        // No-op for PostgreSQL
        Ok(())
    }
}
```

### 3. File Sink

Writes CDC messages to a file:

```rust
pub struct FileSink {
    file: File,
    config: FileSinkConfig,
}

impl CdcSink for FileSink {
    fn send(&mut self, messages: Vec<CdcMessage>) -> Result<()> {
        for message in messages {
            let json = serde_json::to_string(&message)?;
            writeln!(self.file, "{}", json)?;
        }
        Ok(())
    }
    
    fn flush(&mut self) -> Result<()> {
        self.file.flush()?;
        Ok(())
    }
}
```

## CDC Best Practices

### 1. Message Design

- **Include Sufficient Context**: Ensure messages contain enough context to be useful on their own
- **Minimize Message Size**: Avoid including unnecessary data in messages
- **Use Consistent Schemas**: Maintain consistent schemas across different message types
- **Include Metadata**: Add metadata to help with debugging and tracing
- **Handle Schema Evolution**: Design for schema changes over time

### 2. State Management

- **Track Minimal State**: Only store state needed for change detection
- **Use Efficient Data Structures**: Choose appropriate data structures for state tracking
- **Implement Snapshots**: Create periodic state snapshots for recovery
- **Handle State Growth**: Implement state pruning for long-running systems
- **Ensure Determinism**: State changes should be deterministic for consistent CDC output

### 3. Error Handling

- **Implement Retries**: Retry failed sink operations with backoff
- **Use Circuit Breakers**: Prevent cascading failures with circuit breakers
- **Buffer Messages**: Buffer messages during sink outages
- **Log Failures**: Log detailed error information for debugging
- **Monitor Delivery**: Implement monitoring for message delivery

### 4. Performance Optimization

- **Batch Messages**: Send messages in batches for better throughput
- **Optimize Serialization**: Use efficient serialization formats
- **Minimize Memory Usage**: Avoid unnecessary memory allocations
- **Profile Critical Paths**: Identify and optimize bottlenecks
- **Implement Backpressure**: Handle slow consumers with backpressure mechanisms

### 5. Testing

- **Test Reorg Scenarios**: Verify correct behavior during reorgs
- **Simulate Sink Failures**: Test recovery from sink failures
- **Verify Message Order**: Ensure messages are delivered in the correct order
- **Validate Message Content**: Verify message content matches expected changes
- **Benchmark Performance**: Measure and optimize performance under load

## CDC Patterns for Metaprotocols

### 1. Token Balance Tracking

Track token balances and generate CDC events for changes:

```rust
fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
    let height = get_height();
    let hash = get_block_hash();
    
    // Get current token balances
    let params = serialize_params(&TokenBalanceParams { /* ... */ });
    let result = call_view("get_token_balances", &params)?;
    let balances: HashMap<String, u64> = deserialize_result(&result)?;
    
    let mut messages = Vec::new();
    
    // Compare with previous balances and generate CDC messages
    for (address, balance) in &balances {
        let key = format!("balance:{}", address);
        let previous = self.state.get(&key.as_bytes())?;
        
        if let Some(prev_data) = previous {
            let prev_balance: u64 = deserialize(&prev_data)?;
            
            if prev_balance != *balance {
                // Balance changed, generate update message
                messages.push(CdcMessage {
                    header: CdcHeader {
                        source: "token_protocol".to_string(),
                        timestamp: now_ms(),
                        block_height: height,
                        block_hash: hex::encode(hash),
                        transaction_id: None,
                    },
                    payload: CdcPayload {
                        operation: CdcOperation::Update,
                        table: "balances".to_string(),
                        key: address.clone(),
                        before: Some(json!({ "balance": prev_balance })),
                        after: Some(json!({ "balance": balance })),
                    },
                });
            }
        } else {
            // New balance, generate create message
            messages.push(CdcMessage {
                header: CdcHeader {
                    source: "token_protocol".to_string(),
                    timestamp: now_ms(),
                    block_height: height,
                    block_hash: hex::encode(hash),
                    transaction_id: None,
                },
                payload: CdcPayload {
                    operation: CdcOperation::Create,
                    table: "balances".to_string(),
                    key: address.clone(),
                    before: None,
                    after: Some(json!({ "balance": balance })),
                },
            });
        }
        
        // Update state
        self.state.set(&key.as_bytes(), &serialize(balance)?)?;
    }
    
    // Check for deleted balances
    for key in self.state.keys()? {
        if let Ok(key_str) = String::from_utf8(key.clone()) {
            if key_str.starts_with("balance:") {
                let address = key_str.strip_prefix("balance:").unwrap();
                
                if !balances.contains_key(address) {
                    // Balance was deleted
                    let prev_data = self.state.get(&key)?
                        .ok_or_else(|| anyhow!("Missing value for key"))?;
                    let prev_balance: u64 = deserialize(&prev_data)?;
                    
                    messages.push(CdcMessage {
                        header: CdcHeader {
                            source: "token_protocol".to_string(),
                            timestamp: now_ms(),
                            block_height: height,
                            block_hash: hex::encode(hash),
                            transaction_id: None,
                        },
                        payload: CdcPayload {
                            operation: CdcOperation::Delete,
                            table: "balances".to_string(),
                            key: address.to_string(),
                            before: Some(json!({ "balance": prev_balance })),
                            after: None,
                        },
                    });
                    
                    // Remove from state
                    self.state.delete(&key)?;
                }
            }
        }
    }
    
    Ok(messages)
}
```

### 2. NFT Ownership Tracking

Track NFT ownership changes and generate CDC events:

```rust
fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
    let height = get_height();
    let hash = get_block_hash();
    
    // Get current NFT ownership
    let params = serialize_params(&NftOwnershipParams { /* ... */ });
    let result = call_view("get_nft_ownership", &params)?;
    let ownership: HashMap<String, String> = deserialize_result(&result)?;
    
    let mut messages = Vec::new();
    
    // Compare with previous ownership and generate CDC messages
    for (token_id, owner) in &ownership {
        let key = format!("nft:{}", token_id);
        let previous = self.state.get(&key.as_bytes())?;
        
        if let Some(prev_data) = previous {
            let prev_owner: String = deserialize(&prev_data)?;
            
            if prev_owner != *owner {
                // Ownership changed, generate update message
                messages.push(CdcMessage {
                    header: CdcHeader {
                        source: "nft_protocol".to_string(),
                        timestamp: now_ms(),
                        block_height: height,
                        block_hash: hex::encode(hash),
                        transaction_id: None,
                    },
                    payload: CdcPayload {
                        operation: CdcOperation::Update,
                        table: "nft_ownership".to_string(),
                        key: token_id.clone(),
                        before: Some(json!({ "owner": prev_owner })),
                        after: Some(json!({ "owner": owner })),
                    },
                });
            }
        } else {
            // New NFT, generate create message
            messages.push(CdcMessage {
                header: CdcHeader {
                    source: "nft_protocol".to_string(),
                    timestamp: now_ms(),
                    block_height: height,
                    block_hash: hex::encode(hash),
                    transaction_id: None,
                },
                payload: CdcPayload {
                    operation: CdcOperation::Create,
                    table: "nft_ownership".to_string(),
                    key: token_id.clone(),
                    before: None,
                    after: Some(json!({ "owner": owner })),
                },
            });
        }
        
        // Update state
        self.state.set(&key.as_bytes(), &serialize(owner)?)?;
    }
    
    // Check for burned NFTs
    for key in self.state.keys()? {
        if let Ok(key_str) = String::from_utf8(key.clone()) {
            if key_str.starts_with("nft:") {
                let token_id = key_str.strip_prefix("nft:").unwrap();
                
                if !ownership.contains_key(token_id) {
                    // NFT was burned
                    let prev_data = self.state.get(&key)?
                        .ok_or_else(|| anyhow!("Missing value for key"))?;
                    let prev_owner: String = deserialize(&prev_data)?;
                    
                    messages.push(CdcMessage {
                        header: CdcHeader {
                            source: "nft_protocol".to_string(),
                            timestamp: now_ms(),
                            block_height: height,
                            block_hash: hex::encode(hash),
                            transaction_id: None,
                        },
                        payload: CdcPayload {
                            operation: CdcOperation::Delete,
                            table: "nft_ownership".to_string(),
                            key: token_id.to_string(),
                            before: Some(json!({ "owner": prev_owner })),
                            after: None,
                        },
                    });
                    
                    // Remove from state
                    self.state.delete(&key)?;
                }
            }
        }
    }
    
    Ok(messages)
}
```

## Conclusion

Change Data Capture (CDC) is a powerful pattern for tracking and propagating state changes in a system. In the context of debshrew, CDC enables transforming metaprotocol state from metashrew into standardized streams that can be consumed by various downstream systems.

By following the best practices and patterns outlined in this document, developers can create robust, efficient transform modules that generate high-quality CDC streams from metaprotocol state.