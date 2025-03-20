# Quick Start Guide

This guide will help you get started with Debshrew quickly.

## Prerequisites

Before you begin, make sure you have:

- [Installed Debshrew](installation.md)
- A running metashrew instance
- A transform module (or use the example one)

## Step 1: Create a Configuration File

Create a file named `config.json` with the following content:

```json
{
  "metashrew": {
    "url": "http://localhost:8080"
  },
  "transform": {
    "path": "examples/simple-transform/target/wasm32-unknown-unknown/release/simple_transform.wasm"
  },
  "sink": {
    "type": "console",
    "pretty": true
  },
  "cache_size": 6,
  "start_height": 100000,
  "log_level": "info"
}
```

Adjust the values as needed for your environment.

## Step 2: Build the Example Transform Module

If you don't have a transform module yet, you can build the example one:

```bash
cd examples/simple-transform
cargo build --target wasm32-unknown-unknown --release
cd ../..
```

## Step 3: Run Debshrew

Run Debshrew with your configuration file:

```bash
cargo run --release -- run --config config.json
```

Or using command-line arguments:

```bash
cargo run --release -- run \
  --metashrew-url http://localhost:8080 \
  --transform examples/simple-transform/target/wasm32-unknown-unknown/release/simple_transform.wasm \
  --sink-type console \
  --cache-size 6 \
  --start-height 100000
```

## Step 4: Observe the Output

Debshrew will start synchronizing with metashrew and processing blocks. You should see output similar to this:

```
INFO [debshrew] Starting at block height 100000
INFO [debshrew] Processing blocks 100001 to 100010
INFO [debshrew] Processed block 100001
INFO [debshrew] Processed block 100002
...
```

If you're using the console sink, you'll also see CDC messages printed to the console:

```json
{
  "header": {
    "source": "simple_transform",
    "timestamp": "2023-01-01T00:00:00Z",
    "block_height": 100001,
    "block_hash": "000000000000000000024bead8df69990852c202db0e0097c1a12ea637d7e96d",
    "transaction_id": null
  },
  "payload": {
    "operation": "create",
    "table": "blocks",
    "key": "100001",
    "before": null,
    "after": {
      "height": 100001,
      "hash": "000000000000000000024bead8df69990852c202db0e0097c1a12ea637d7e96d",
      "timestamp": "2023-01-01T00:00:00Z"
    }
  }
}
```

## Step 5: Create Your Own Transform Module

Now that you've seen Debshrew in action, you can create your own transform module:

1. Create a new Rust project:

```bash
cargo new --lib my-transform
cd my-transform
```

2. Add the necessary dependencies to `Cargo.toml`:

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
```

3. Implement your transform module in `src/lib.rs`:

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
        
        // Query metashrew views
        let result = call_view("my_view", &[])?;
        
        // Generate CDC messages
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
                    table: "my_table".to_string(),
                    key: "my_key".to_string(),
                    before: None,
                    after: Some(serde_json::json!({
                        "field1": "value1",
                        "field2": 42
                    })),
                },
            }
        ];
        
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

4. Build your transform module:

```bash
cargo build --target wasm32-unknown-unknown --release
```

5. Update your configuration to use your new transform module:

```json
{
  "transform": {
    "path": "my-transform/target/wasm32-unknown-unknown/release/my_transform.wasm"
  }
}
```

6. Run Debshrew with your new transform module:

```bash
cargo run --release -- run --config config.json
```

## Next Steps

Now that you have Debshrew up and running, you can:

- [Learn more about the architecture](architecture.md)
- [Explore the CDC concepts](cdc-concepts.md)
- [Understand the metashrew integration](metashrew-integration.md)
- [Dive deeper into WASM transforms](wasm-transform-guide.md)