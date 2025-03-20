# Debshrew Technical Context

## Technologies Used

### Programming Languages

- **Rust**: The primary language used throughout the project, chosen for its performance, memory safety, and strong type system.
- **WebAssembly (WASM)**: The compilation target for transform modules, enabling portable and sandboxed execution.
- **AssemblyScript** (supported transform language): A TypeScript-like language that compiles to WebAssembly, making it accessible for web developers.
- **C/C++** (supported transform language): Can be compiled to WebAssembly using Emscripten.

### Runtime Environments

- **wasmtime**: WebAssembly runtime used for executing WASM transform modules.
- **tokio**: Asynchronous runtime for handling concurrent operations.
- **actix-web**: Web framework for the API server (if applicable).

### Storage

- **In-memory Block Cache**: Stores recent blocks, state snapshots, and CDC messages for reorg handling.
- **Persistent State Storage**: Optional persistent storage for transform state to enable restarts without reprocessing.

### Serialization

- **Protocol Buffers**: Used for data serialization in the runtime.
- **serde/serde_json**: Used for JSON serialization and deserialization.

### CDC Output Sinks

- **Kafka**: For streaming CDC messages to Kafka topics.
- **PostgreSQL**: For direct integration with PostgreSQL databases.
- **File**: For writing CDC messages to files for testing or batch processing.
- **Custom Sinks**: Extensible architecture allows for custom sink implementations.

### Metashrew Integration

- **JSON-RPC Client**: For communicating with metashrew instances.
- **View Access**: For querying metaprotocol state from metashrew views.

### Utilities

- **anyhow**: Error handling library.
- **clap**: Command-line argument parsing.
- **log/env_logger**: Logging infrastructure.
- **metrics**: For collecting and exposing metrics.

## CDC Message Format

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

Key characteristics:

1. **Header**: Contains metadata about the source block and transaction.
2. **Payload**: Contains the operation type, table, key, and before/after states.
3. **Operation Types**: Supports create, update, and delete operations.
4. **Before/After States**: Enables generating inverse operations for rollbacks.

## Transform Module Interface

Transform modules implement the `DebTransform` trait:

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

## Development Setup

### Prerequisites

- **Rust Toolchain**: Latest stable version
- **Metashrew Instance**: Running metashrew instance with the relevant metaprotocol indexer
- **WebAssembly Tools**: For WASM module development
- **CDC Sink Dependencies**: Depending on the sink (e.g., Kafka, PostgreSQL)

### Building Debshrew

```sh
# Clone the repository
git clone https://github.com/example/debshrew
cd debshrew

# Build the binary
cargo build --release

# Or build with specific features
cargo build --release --features kafka,postgres
```

### Running Debshrew

```sh
# Basic usage
./target/release/debshrew \
  --metashrew-url http://localhost:8080 \
  --transform path/to/transform.wasm \
  --sink-type kafka \
  --sink-config kafka.json \
  --cache-size 6

# With PostgreSQL sink
./target/release/debshrew \
  --metashrew-url http://localhost:8080 \
  --transform path/to/transform.wasm \
  --sink-type postgres \
  --sink-config postgres.json \
  --cache-size 6
```

### Developing Transform Modules

#### Rust Example

```rust
use debshrew_runtime::*;

#[derive(Default)]
struct ExampleTransform {
    state: TransformState
}

impl DebTransform for ExampleTransform {
    fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
        // Get current block info
        let height = get_height();
        let hash = get_block_hash();
        
        // Query metashrew views
        let params = serialize_params(&ExampleParams { /* ... */ });
        let result = call_view("example_view", &params)?;
        let view_data: ExampleViewResult = deserialize_result(&result)?;
        
        // Process state and generate CDC messages
        let mut messages = Vec::new();
        
        for item in view_data.items {
            // Check if we've seen this item before
            let key = format!("item:{}", item.id).into_bytes();
            let previous = self.state.get(&key);
            
            if let Some(prev_data) = previous {
                // Item exists, check if it changed
                let prev_item: Item = deserialize(&prev_data)?;
                
                if prev_item != item {
                    // Generate update message
                    messages.push(CdcMessage {
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
                    });
                }
            } else {
                // New item, generate create message
                messages.push(CdcMessage {
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
                });
            }
            
            // Update state
            self.state.set(key, serialize(&item)?);
        }
        
        // Check for deleted items
        // ...
        
        Ok(messages)
    }
    
    fn rollback(&mut self) -> Result<Vec<CdcMessage>> {
        // Generate inverse operations for rollback
        // ...
    }
}

declare_transform!(ExampleTransform);
```

#### AssemblyScript Example

```typescript
// transform.ts
import { CdcMessage, CdcHeader, CdcPayload, CdcOperation } from "debshrew-runtime";

export class ExampleTransform {
  private state: Map<string, string> = new Map();
  
  processBlock(): CdcMessage[] {
    // Get current block info
    const height = getHeight();
    const hash = getBlockHash();
    
    // Query metashrew views
    const params = serializeParams({ /* ... */ });
    const result = callView("example_view", params);
    const viewData = deserializeResult(result);
    
    // Process state and generate CDC messages
    const messages: CdcMessage[] = [];
    
    for (const item of viewData.items) {
      // Check if we've seen this item before
      const key = `item:${item.id}`;
      const previous = this.state.get(key);
      
      if (previous) {
        // Item exists, check if it changed
        const prevItem = JSON.parse(previous);
        
        if (JSON.stringify(prevItem) !== JSON.stringify(item)) {
          // Generate update message
          messages.push({
            header: {
              source: "example_protocol",
              timestamp: Date.now(),
              block_height: height,
              block_hash: hash.toString("hex"),
              transaction_id: null,
            },
            payload: {
              operation: CdcOperation.Update,
              table: "items",
              key: item.id.toString(),
              before: prevItem,
              after: item,
            },
          });
        }
      } else {
        // New item, generate create message
        messages.push({
          header: {
            source: "example_protocol",
            timestamp: Date.now(),
            block_height: height,
            block_hash: hash.toString("hex"),
            transaction_id: null,
          },
          payload: {
            operation: CdcOperation.Create,
            table: "items",
            key: item.id.toString(),
            before: null,
            after: item,
          },
        });
      }
      
      // Update state
      this.state.set(key, JSON.stringify(item));
    }
    
    // Check for deleted items
    // ...
    
    return messages;
  }
  
  rollback(): CdcMessage[] {
    // Generate inverse operations for rollback
    // ...
  }
}

export function _start(): void {
  // Entry point
}
```

## Technical Constraints

### WebAssembly Constraints

- **Memory Limit**: WebAssembly modules have a limited memory space (currently 4GB maximum).
- **No Direct System Access**: WASM modules cannot directly access the file system or network.
- **Limited Standard Library**: Some standard library functions may not be available in WASM.
- **Performance Overhead**: There's some overhead compared to native code, though it's minimal.

### Metashrew Constraints

- **View Performance**: The performance of metashrew views affects the overall performance of debshrew.
- **View Availability**: Debshrew depends on the availability of metashrew views.
- **Reorg Handling**: Debshrew relies on metashrew for detecting chain reorganizations.

### CDC Sink Constraints

- **Throughput**: Different sinks have different throughput characteristics.
- **Latency**: Some sinks may introduce additional latency.
- **Reliability**: Sinks may have different reliability guarantees.
- **Ordering**: Some sinks may not preserve message ordering.

### Memory Constraints

- **Block Cache Size**: The block cache size affects memory usage and reorg handling capabilities.
- **Transform State Size**: Large transform states may impact performance and memory usage.
- **CDC Message Size**: Large CDC messages may impact performance and sink throughput.

## Performance Considerations

### Transform Module Optimization

- **Efficient State Management**: Minimize state size and access patterns.
- **Batch Processing**: Process multiple items in batches when possible.
- **Incremental Updates**: Only generate CDC messages for changed items.
- **Memory Usage**: Be mindful of memory allocations and deallocations.

### CDC Sink Optimization

- **Batching**: Batch CDC messages to improve throughput.
- **Compression**: Consider compressing CDC messages to reduce network usage.
- **Connection Pooling**: Use connection pooling for database sinks.
- **Backpressure Handling**: Implement backpressure mechanisms to avoid overwhelming sinks.

### Metashrew Integration Optimization

- **View Selection**: Choose efficient views that provide the necessary data.
- **Parameter Optimization**: Optimize view parameters to minimize data transfer.
- **Caching**: Consider caching view results when appropriate.
- **Connection Management**: Manage metashrew connections efficiently.

### Memory Management

- **Block Cache Tuning**: Adjust the block cache size based on available memory and reorg frequency.
- **State Pruning**: Implement state pruning for large transform states.
- **Memory Monitoring**: Monitor memory usage and implement limits.

## Security Considerations

### WebAssembly Sandboxing

- **Memory Isolation**: WASM modules have isolated memory spaces.
- **Limited Capabilities**: WASM modules can only access host functions explicitly provided.
- **Resource Limits**: The host can limit resources available to WASM modules.

### CDC Sink Security

- **Authentication**: Implement proper authentication for CDC sinks.
- **Encryption**: Use encryption for sensitive data.
- **Access Control**: Implement appropriate access controls for CDC sinks.
- **Input Validation**: Validate all input from transform modules.

### API Security

- **Authentication**: Implement proper authentication for APIs.
- **Rate Limiting**: Implement rate limiting to prevent abuse.
- **Input Validation**: Validate all API inputs.

## Deployment Considerations

### Resource Requirements

- **CPU**: Depends on the complexity of transform modules and the number of blocks processed.
- **Memory**: Depends on the block cache size and transform state size.
- **Disk**: Minimal for debshrew itself, but may be significant for CDC sinks.
- **Network**: Depends on the volume of CDC messages and metashrew communication.

### Monitoring

- **Metrics**: Monitor block processing rate, CDC message generation rate, and sink throughput.
- **Logging**: Implement comprehensive logging for debugging and auditing.
- **Alerting**: Set up alerts for critical conditions like processing delays or sink failures.

### High Availability

- **Multiple Instances**: Run multiple instances of debshrew for high availability.
- **State Synchronization**: Implement state synchronization between instances.
- **Failover**: Implement failover mechanisms for critical components.

### Scaling

- **Horizontal Scaling**: Scale horizontally by running multiple instances with different transforms.
- **Vertical Scaling**: Scale vertically by increasing resources for a single instance.
- **Sink Scaling**: Scale CDC sinks independently based on throughput requirements.

## Conclusion

Debshrew's technical architecture, centered around WebAssembly and CDC message generation, provides a flexible and powerful platform for transforming metaprotocol state into standardized data streams. The system is designed to handle the demands of various metaprotocols while maintaining flexibility for a wide range of use cases.

By understanding the technical constraints and performance considerations, developers can optimize their transforms and deployments for specific workloads and ensure reliable operation in production environments.