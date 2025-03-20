# Active Context

## Current Work Focus

The debshrew project is currently focused on the following key areas:

1. **Core Service Implementation**
   - Implementing the main debshrew service
   - Developing the WASM host interface
   - Building the block synchronization mechanism
   - Implementing the block cache for reorg handling
   - Creating the CDC message generation pipeline

2. **Runtime Library Development**
   - Designing the DebTransform trait
   - Implementing the WASM host functions
   - Creating the state management system
   - Developing the view access interface
   - Building the CDC message generation utilities

3. **CDC Sink Implementations**
   - Implementing the Kafka sink
   - Developing the PostgreSQL sink
   - Creating the file sink for testing
   - Building the sink interface for custom implementations

4. **Metashrew Integration**
   - Implementing the metashrew client
   - Developing the view access layer
   - Building the block synchronization mechanism
   - Implementing reorg detection

5. **Transform Module Examples**
   - Creating example transform modules for common metaprotocols
   - Developing testing utilities for transform modules
   - Building documentation and tutorials

## Recent Changes

### Core Service Implementation

1. **Project Structure Setup**
   - Created the initial project structure with Cargo workspace
   - Set up the main crates: debshrew, debshrew-runtime, debshrew-support
   - Configured build system and dependencies
   - Established coding standards and documentation guidelines

2. **WASM Host Interface**
   - Defined the core interface between the host and WASM modules
   - Implemented the basic host functions for view access and state management
   - Created the WASM module loading and execution mechanism
   - Implemented memory management for WASM modules

3. **Block Synchronization**
   - Implemented the initial block synchronization mechanism
   - Created the polling logic for new blocks
   - Developed the block processing pipeline
   - Implemented basic reorg detection

### Runtime Library Development

1. **DebTransform Trait**
   - Defined the DebTransform trait for transform modules
   - Implemented the process_block and rollback methods
   - Created the transform state management system
   - Developed the CDC message generation utilities

2. **State Management**
   - Implemented the TransformState struct for state management
   - Created the state persistence mechanism
   - Developed the state snapshot system for the block cache
   - Implemented state rollback for reorgs

3. **View Access**
   - Created the view access interface for metashrew
   - Implemented the call_view function for querying metashrew views
   - Developed the parameter serialization and result deserialization utilities
   - Created the block information access functions

### CDC Sink Development

1. **Sink Interface**
   - Defined the CdcSink trait for sink implementations
   - Implemented the send and flush methods
   - Created the sink configuration system
   - Developed the sink factory for creating sink instances

2. **Initial Sink Implementations**
   - Started implementing the Kafka sink
   - Began development of the PostgreSQL sink
   - Created the file sink for testing
   - Implemented the null sink for benchmarking

## Next Steps

### Immediate Priorities

1. **Complete Core Service Implementation**
   - Finish the block cache implementation
   - Complete the reorg handling mechanism
   - Implement the CDC message buffering system
   - Develop the service configuration system
   - Create the service lifecycle management

2. **Enhance Runtime Library**
   - Improve the state management system
   - Optimize the view access interface
   - Enhance the CDC message generation utilities
   - Implement advanced error handling
   - Create comprehensive testing utilities

3. **Finalize CDC Sink Implementations**
   - Complete the Kafka sink with all features
   - Finish the PostgreSQL sink implementation
   - Enhance the file sink with more options
   - Implement the console sink for debugging
   - Create documentation for custom sink development

4. **Improve Metashrew Integration**
   - Enhance the metashrew client with connection pooling
   - Optimize the view access layer for performance
   - Improve the block synchronization mechanism
   - Enhance reorg detection and handling
   - Implement metrics and monitoring

5. **Develop Example Transform Modules**
   - Create transform modules for popular metaprotocols
   - Develop comprehensive examples for different use cases
   - Build testing utilities for transform modules
   - Create documentation and tutorials

### Medium-term Goals

1. **Performance Optimization**
   - Optimize WASM execution
   - Improve memory usage
   - Enhance CDC message generation performance
   - Optimize sink throughput
   - Implement benchmarking tools

2. **Advanced Features**
   - Implement transform module hot reloading
   - Develop advanced state management features
   - Create a transform module registry
   - Implement a plugin system for extensions
   - Develop a web interface for monitoring

3. **Integration Testing**
   - Create comprehensive integration tests
   - Develop testing utilities for different components
   - Implement continuous integration
   - Create performance testing tools
   - Develop stress testing scenarios

4. **Documentation and Examples**
   - Create comprehensive documentation
   - Develop tutorials and guides
   - Build example applications
   - Create video tutorials
   - Develop a documentation website

5. **Community Building**
   - Create a contribution guide
   - Develop a roadmap for future development
   - Implement a plugin ecosystem
   - Create a community forum
   - Develop a showcase of projects using debshrew

## Active Decisions and Considerations

1. **Block Cache Size**
   - Currently using a 6-block cache for reorg protection
   - Considering making this configurable based on the metaprotocol's needs
   - Evaluating the memory usage implications of larger cache sizes
   - Exploring options for persistent caching to handle larger reorgs

2. **CDC Message Format**
   - Using a Debezium-compatible format for CDC messages
   - Considering additional metadata fields for Bitcoin-specific information
   - Evaluating the performance implications of different serialization formats
   - Exploring options for schema evolution and versioning

3. **Transform Module Interface**
   - Current interface focuses on simplicity with process_block and rollback methods
   - Considering additional lifecycle methods for initialization and cleanup
   - Evaluating the need for more advanced state management features
   - Exploring options for transform module composition and reuse

4. **Sink Implementation Strategy**
   - Currently implementing sinks for Kafka, PostgreSQL, and files
   - Considering additional sinks for other popular systems
   - Evaluating the performance characteristics of different sink implementations
   - Exploring options for sink composition and chaining

5. **Metashrew Integration Approach**
   - Currently using a direct JSON-RPC client for metashrew integration
   - Considering a more decoupled approach with an abstraction layer
   - Evaluating the performance implications of different integration strategies
   - Exploring options for caching and batching metashrew requests

## Current Challenges

1. **Reorg Handling Complexity**
   - Ensuring correct state rollback during reorgs
   - Generating accurate inverse CDC messages for rollbacks
   - Handling deep reorgs that exceed the block cache size
   - Maintaining consistency across different sinks during reorgs

2. **WASM Performance**
   - Optimizing WASM execution for complex transform modules
   - Managing memory usage in WASM modules
   - Handling large state in WASM modules
   - Ensuring deterministic execution across different environments

3. **CDC Sink Reliability**
   - Ensuring reliable delivery of CDC messages to sinks
   - Handling sink failures and retries
   - Maintaining ordering guarantees across different sinks
   - Implementing backpressure mechanisms to avoid overwhelming sinks

4. **Metashrew Dependency**
   - Managing the dependency on metashrew for block data
   - Handling metashrew unavailability or performance issues
   - Ensuring compatibility with different metashrew versions
   - Optimizing metashrew view access for performance

5. **Transform Module Development Experience**
   - Creating a good developer experience for transform module authors
   - Providing comprehensive documentation and examples
   - Implementing testing utilities for transform modules
   - Supporting different languages for transform module development