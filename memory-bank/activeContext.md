# Active Context

## Current Focus

We are currently implementing the debshrew-runtime based on the subrail approach. This involves:

1. Creating a WASM runtime that can execute transform modules
2. Implementing host functions for transform modules to interact with the host environment
3. Providing a mechanism for transform modules to generate CDC messages
4. Implementing automatic rollback generation for chain reorganizations

## Recent Changes

### WASM Runtime Implementation

We've implemented a new WASM runtime based on the subrail approach:

- Created a `declare_transform` macro that generates WASM exports for transform modules
- Implemented host functions for transform modules to interact with the host environment
- Added support for CDC message generation and automatic rollback
- Updated the BlockSynchronizer to use the new CDC message inversion approach

### Transform Module Interface

We've updated the `DebTransform` trait to use a push-based approach for CDC messages:

```rust
pub trait DebTransform: Default + Debug + Clone {
    fn process_block(&mut self) -> Result<()>;
    fn rollback(&mut self) -> Result<()> {
        // Default implementation does nothing
        // The runtime will automatically generate inverse CDC messages
        Ok(())
    }
}
```

Transform modules now push CDC messages to the host using the `push_message` method:

```rust
impl MyTransform {
    // Helper method to push CDC messages
    pub fn push_message(&self, message: CdcMessage) -> Result<()> {
        push_cdc_message(&message)
    }
}
```

### Automatic Rollback Generation

We've implemented automatic rollback generation for chain reorganizations:

- The runtime caches CDC messages for each block
- During reorgs, it automatically generates inverse CDC messages
- The inverse messages are sent to the sink to undo the changes

The inversion rules are:

1. **Create → Delete**: A Create operation becomes a Delete operation
2. **Update → Update**: An Update operation becomes another Update but with before/after states swapped
3. **Delete → Create**: A Delete operation becomes a Create operation

### Example Transform Module

We've created an example transform module that demonstrates the new approach:

- Implements the `DebTransform` trait
- Uses the host functions to interact with the host environment
- Generates CDC messages for changes in token balances
- Relies on automatic rollback generation for reorgs

## Next Steps

1. **Testing**: Test the new WASM runtime with different transform modules and scenarios
2. **Documentation**: Update the documentation to reflect the new approach
3. **Performance Optimization**: Optimize the WASM runtime for better performance
4. **Error Handling**: Improve error handling in the WASM runtime
5. **CDC Sink Implementation**: Complete the implementation of CDC sinks
6. **Metashrew Integration**: Finalize the integration with metashrew

## Active Decisions

1. **Push-Based CDC Messages**: We've decided to use a push-based approach for CDC messages, where transform modules push messages to the host rather than returning them directly. This allows for more flexibility and better error handling.

2. **Automatic Rollback Generation**: We've decided to implement automatic rollback generation for chain reorganizations, where the runtime automatically generates inverse CDC messages based on the original messages. This simplifies the implementation of transform modules and ensures consistent rollback behavior.

3. **State Management**: We've decided to use a key-value store for state management, where transform modules can store and retrieve state using the `set_state` and `get_state` functions. This allows for efficient state management and easy serialization.

4. **WASM Memory Management**: We've decided to use a static variable for state management in WASM modules, with the host responsible for taking memory snapshots for reorg handling. This simplifies the implementation of transform modules and ensures consistent state management.

## Current Challenges

1. **Deep Reorgs**: Handling reorgs deeper than the block cache size requires special handling. We're currently using a 6-block cache and warning on deeper reorgs.

2. **WASM Performance**: Complex transform modules may have performance issues. We're optimizing the WASM runtime for better performance.

3. **CDC Sink Reliability**: Different sinks have different reliability characteristics. We're implementing retry mechanisms and circuit breakers for failing sinks.

4. **Metashrew Dependency**: Debshrew depends on metashrew for data. We're implementing connection pooling and retry mechanisms for metashrew.