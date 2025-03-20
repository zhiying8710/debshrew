# Metashrew Integration

Debshrew is designed to work closely with Metashrew, a Bitcoin indexing and view system. This document explains how Debshrew integrates with Metashrew.

## What is Metashrew?

[Metashrew](https://github.com/metashrew/metashrew) is a Bitcoin indexing and view system that allows you to:

- Index Bitcoin blocks and transactions
- Create custom views of Bitcoin data
- Query those views efficiently
- Build applications on top of Bitcoin data

Metashrew uses a WASM-based approach similar to Debshrew, allowing you to write indexers and views in any language that compiles to WebAssembly.

## How Debshrew Uses Metashrew

Debshrew uses Metashrew as its data source, connecting to a Metashrew instance to:

1. Get the latest block height
2. Get block hashes
3. Query views to detect changes
4. Generate CDC messages based on those changes

## Metashrew Client

Debshrew includes a Metashrew client that communicates with Metashrew using its JSON-RPC API. The client provides methods for:

- Getting the current block height
- Getting block hashes
- Calling view functions
- Handling errors and retries

## View Functions

Metashrew views are functions that transform Bitcoin data into application-specific data structures. Debshrew transform modules can call these view functions to get the data they need.

For example, a transform module might call a view function to get all NFTs owned by a specific address:

```rust
let nfts = call_view("get_nfts_by_owner", &serialize(&owner_address)?)?;
```

## Block Synchronization

Debshrew synchronizes with Metashrew by:

1. Polling Metashrew for the latest block height
2. Comparing it with the current height
3. If new blocks are available, processing them sequentially
4. If a reorg is detected, rolling back to the common ancestor and reprocessing blocks

## Reorg Handling

Metashrew provides the block hash for each height, which Debshrew uses to detect reorgs. When a reorg is detected, Debshrew:

1. Finds the common ancestor between the old and new chain
2. Rolls back the state to the common ancestor
3. Generates inverse CDC messages for rolled back blocks
4. Processes the new chain from the common ancestor

## Connection Configuration

Debshrew can be configured to connect to Metashrew using the following options:

```json
{
  "metashrew": {
    "url": "http://localhost:8080",
    "username": "user",
    "password": "password",
    "timeout": 30,
    "max_retries": 3,
    "retry_delay": 1000
  }
}
```

## Authentication

Metashrew supports basic authentication, which Debshrew can use by providing a username and password in the configuration.

## Error Handling

Debshrew includes robust error handling for Metashrew integration:

- Connection errors are retried with exponential backoff
- View function errors are propagated to the transform module
- Reorg detection and handling is automatic

## Performance Considerations

When integrating with Metashrew, consider the following performance factors:

- **Polling Interval**: How frequently Debshrew checks for new blocks
- **View Function Complexity**: Complex view functions can slow down processing
- **Network Latency**: High latency can impact synchronization speed
- **Batch Size**: Processing many blocks at once can improve throughput

## Example: Calling a Metashrew View

Here's an example of how a transform module might call a Metashrew view:

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
        
        // Call a Metashrew view function
        let params = serialize(&height)?;
        let result = call_view("get_block_transactions", &params)?;
        let transactions: Vec<Transaction> = deserialize(&result)?;
        
        // Generate CDC messages based on the view result
        let mut messages = Vec::new();
        
        for tx in transactions {
            messages.push(CdcMessage {
                header: CdcHeader {
                    source: "my_transform".to_string(),
                    timestamp: chrono::Utc::now(),
                    block_height: height,
                    block_hash: hex::encode(&hash),
                    transaction_id: Some(tx.id.clone()),
                },
                payload: CdcPayload {
                    operation: CdcOperation::Create,
                    table: "transactions".to_string(),
                    key: tx.id.clone(),
                    before: None,
                    after: Some(serde_json::to_value(tx)?),
                },
            });
        }
        
        Ok(messages)
    }
    
    fn rollback(&mut self) -> Result<Vec<CdcMessage>> {
        // Generate inverse CDC messages for rollback
        Ok(vec![])
    }
}

// Declare the transform module
declare_transform!(MyTransform);
```

## Setting Up a Local Metashrew Instance

For development and testing, you can set up a local Metashrew instance:

1. Clone the Metashrew repository:

```bash
git clone https://github.com/metashrew/metashrew.git
cd metashrew
```

2. Build and run Metashrew:

```bash
cargo build --release
cargo run --release
```

3. Configure Debshrew to connect to your local Metashrew instance:

```json
{
  "metashrew": {
    "url": "http://localhost:8080"
  }
}
```

## Advanced Integration

For advanced integration scenarios, consider:

- **Custom Views**: Create custom Metashrew views tailored to your application
- **Shared State**: Use Debshrew's state management to persist data between blocks
- **Optimized Queries**: Design view functions that minimize data transfer
- **Batched Requests**: Combine multiple view calls into a single request