
# debshrew

A framework for building deterministic CDC (Change Data Capture) streams from Bitcoin metaprotocol state by leveraging metashrew indexing. debshrew enables building highly available, reorg-aware ETL pipelines that transform metaprotocol state into standardized CDC streams consumable by the debezium ecosystem.

## Core Features

- WASM-based transformation programs
- Deterministic CDC generation from metashrew views  
- Automatic reorg handling with state rollbacks
- 6-block caching for reorg protection
- Debezium-compatible CDC output
- Extensible metaprotocol support

## System Architecture

debshrew follows a similar architecture to subrail:

1. Core Service (`debshrew`)
   - Connects to metashrew instance
   - Manages WASM program execution
   - Handles block synchronization
   - Maintains block cache
   - Processes reorgs
   - Outputs CDC streams

2. Runtime Library (`debshrew-runtime`) 
   - WASM host interface
   - View access traits
   - CDC generation traits
   - Memory management
   - State persistence

3. Support Libraries
   - Protobuf definitions
   - CDC encoding/decoding
   - Type helpers
   - Protocol-specific tools

## Key Components

### Transform Programs
Written in Rust, compiled to WASM. Define:
- State tracking logic
- metashrew view queries
- CDC message generation
- Rollback handling

### CDC Output
- Debezium-compatible format
- Transaction boundaries match blocks
- Includes metadata for reorg tracking
- Supports standard CDC operations (create/update/delete)

### Reorg Handling
- 6-block state cache
- Automatic rollback generation
- Deterministic state recovery
- Transaction atomicity preservation

## Usage Pattern

1. Define transform program implementing DebTransform trait
2. Compile to WASM
3. Run debshrew service connecting to metashrew
4. Configure CDC consumers (PostgreSQL, Kafka, etc.)

## Determinism Guarantee

The system guarantees:
- Identical CDC output for same block height
- Proper state rollback on reorgs
- Consistent transaction boundaries
- Reliable reorg detection via metashrew

## Integration Points

- Input: metashrew JSON-RPC
- Output: Debezium CDC format
- Runtime: WASM
- Transport: Various Debezium connectors

## Development Workflow

1. Create protocol-specific transform
2. Test with cached block data
3. Deploy with metashrew connection
4. Configure CDC consumers
5. Monitor reorg handling

## Example Transform

Basic structure of a transform program:

```rust
use debshrew_runtime::*;

#[derive(Default)]
struct ExampleTransform {
    state: TransformState
}

impl DebTransform for ExampleTransform {
    fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
        // Query views
        // Process state
        // Generate CDC
    }
    
    fn rollback(&mut self) -> Result<Vec<CdcMessage>> {
        // Generate inverse operations
    }
}

declare_transform!(ExampleTransform);
```

## Target Use Cases

- Protocol state to PostgreSQL
- Metaprotocol indexing to Kafka
- Cross-protocol data correlation
- High-availability data feeds
- Analytics pipelines
