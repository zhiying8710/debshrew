# Debshrew Product Context

## Purpose and Problem Statement

Debshrew addresses several key challenges in the Bitcoin metaprotocol ecosystem:

1. **Data Integration Complexity**: Integrating Bitcoin metaprotocol data with existing data systems is complex and requires custom solutions for each protocol.

2. **Reorg Handling**: Bitcoin chain reorganizations can invalidate previously processed data, requiring complex rollback and recovery mechanisms.

3. **ETL Pipeline Fragmentation**: Each metaprotocol typically implements its own ETL (Extract, Transform, Load) pipeline, leading to duplication of effort and inconsistent approaches.

4. **Lack of Standardization**: No standard format exists for consuming metaprotocol state changes in traditional data systems.

5. **Operational Overhead**: Maintaining custom data pipelines for each metaprotocol increases operational complexity and resource requirements.

Debshrew solves these problems by providing a flexible, WebAssembly-powered framework that transforms metaprotocol state from metashrew into standardized Change Data Capture (CDC) streams compatible with the Debezium ecosystem.

## Target Users and Use Cases

### Primary Users

1. **Metaprotocol Developers**: Teams building protocols on top of Bitcoin who need to make their data available to traditional data systems.

2. **Data Engineers**: Engineers responsible for integrating blockchain data with existing data infrastructure.

3. **Application Developers**: Developers building applications that require access to metaprotocol data in standard formats.

4. **Infrastructure Providers**: Organizations providing Bitcoin data services to other applications.

### Key Use Cases

1. **Protocol State to PostgreSQL**: Streaming metaprotocol state changes directly into PostgreSQL databases.

2. **Metaprotocol Indexing to Kafka**: Publishing metaprotocol events to Kafka topics for downstream processing.

3. **Cross-Protocol Data Correlation**: Combining data from multiple metaprotocols in a consistent format.

4. **High-Availability Data Feeds**: Creating reliable, reorg-aware data feeds for critical applications.

5. **Analytics Pipelines**: Feeding metaprotocol data into analytics systems for business intelligence.

## How It Works

Debshrew operates on a simple yet powerful principle: developers implement transformation logic that converts metaprotocol state into CDC events, while the framework handles everything else.

### Core Workflow

1. **Block Retrieval**: Debshrew connects to a metashrew instance and retrieves blocks and metaprotocol state.

2. **WASM Execution**: For each block, Debshrew loads the developer's WebAssembly module and passes the metaprotocol state to it.

3. **CDC Generation**: The WASM module processes the state and generates CDC events (create, update, delete operations).

4. **Reorg Handling**: If a chain reorganization occurs, Debshrew automatically rolls back affected CDC events and regenerates them.

5. **CDC Output**: The generated CDC events are output in a Debezium-compatible format for consumption by downstream systems.

### Transform Program Implementation

Developers implement the `DebTransform` trait in their WASM modules:

```rust
use debshrew_runtime::*;

#[derive(Default)]
struct ExampleTransform {
    state: TransformState
}

impl DebTransform for ExampleTransform {
    fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
        // Query metashrew views
        // Process state
        // Generate CDC messages
    }
    
    fn rollback(&mut self) -> Result<Vec<CdcMessage>> {
        // Generate inverse operations for rollback
    }
}

declare_transform!(ExampleTransform);
```

## User Experience Goals

Debshrew aims to provide:

1. **Simplicity**: Reduce the complexity of integrating Bitcoin metaprotocol data with existing systems.

2. **Reliability**: Ensure correct handling of edge cases like chain reorganizations.

3. **Standardization**: Provide a consistent format for metaprotocol data across different protocols.

4. **Flexibility**: Support a wide range of use cases through its extensible architecture.

5. **Performance**: Provide efficient transformation and streaming capabilities, even for large-scale applications.

6. **Reusability**: Enable sharing and reuse of transformation components across projects.

## Competitive Landscape

Debshrew exists in an ecosystem with other blockchain data integration solutions:

1. **Custom ETL Pipelines**: Many projects build their own ETL pipelines from scratch.
   - *Difference*: Debshrew eliminates the need to build custom ETL infrastructure and provides automatic reorg handling.

2. **Blockchain ETL Tools**: Generic blockchain ETL tools like ethereum-etl.
   - *Difference*: Debshrew is specifically designed for Bitcoin metaprotocols and integrates directly with metashrew.

3. **Streaming Platforms**: General-purpose streaming platforms like Kafka Connect.
   - *Difference*: Debshrew provides Bitcoin-specific features like reorg handling and metaprotocol awareness.

4. **Data Integration Frameworks**: General-purpose data integration frameworks.
   - *Difference*: Debshrew's WASM-based approach offers more flexibility in processing logic and is optimized for Bitcoin.

## Success Metrics

The success of Debshrew can be measured by:

1. **Adoption**: Number of projects using Debshrew for their data integration needs.

2. **Ecosystem Growth**: Variety of metaprotocols integrated with Debshrew.

3. **Performance**: Transformation speed and CDC output rates compared to custom solutions.

4. **Stability**: Uptime and reliability in production environments.

5. **Developer Satisfaction**: Feedback from developers using the framework.

## Future Vision

The long-term vision for Debshrew includes:

1. **Expanded Ecosystem**: A rich ecosystem of reusable WASM modules for common transformation tasks.

2. **Enhanced Performance**: Continued optimization for handling larger metaprotocol datasets.

3. **Additional Connectors**: Support for more CDC consumers beyond the current Debezium ecosystem.

4. **Developer Tools**: Building better tools for developing, testing, and deploying WASM modules.

5. **Integration Standards**: Establishing standards for metaprotocol data integration.

6. **Cloud-Native Deployment**: Simplified deployment options for cloud environments.

## Relationship to Metashrew

Metashrew is the foundation upon which Debshrew is built:

1. **Data Source**: Metashrew provides the indexed metaprotocol state that Debshrew transforms.

2. **Complementary Tools**: Metashrew focuses on indexing and state management, while Debshrew focuses on data integration.

3. **Shared Architecture**: Both use WebAssembly for extensibility and customization.

4. **Ecosystem Synergy**: Improvements to Metashrew often benefit Debshrew, and vice versa.

The relationship between Metashrew and Debshrew exemplifies how specialized tools can work together to solve complex problems in the Bitcoin ecosystem.