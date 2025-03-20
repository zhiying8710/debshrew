# Debshrew Documentation

Welcome to the Debshrew documentation! Debshrew is a framework for building deterministic CDC (Change Data Capture) streams from Bitcoin metaprotocol state.

## What is Debshrew?

Debshrew enables building highly available, reorg-aware ETL pipelines that transform metaprotocol state into standardized CDC streams consumable by the debezium ecosystem.

## Core Features

- **WASM-based transformation programs**: Write your transform logic in any language that compiles to WebAssembly
- **Deterministic CDC generation**: Generate consistent CDC messages from metashrew views
- **Automatic reorg handling**: Handle blockchain reorganizations with state rollbacks
- **6-block caching**: Protect against common reorgs with a configurable block cache
- **Debezium-compatible CDC output**: Integrate with existing Debezium consumers
- **Extensible metaprotocol support**: Support any metaprotocol through custom transform modules

## Getting Started

- [Installation Guide](installation.md)
- [Quick Start Guide](quickstart.md)
- [Configuration Guide](configuration.md)

## Core Concepts

- [Architecture Overview](architecture.md)
- [CDC Concepts](cdc-concepts.md)
- [Metashrew Integration](metashrew-integration.md)
- [WASM Transform Guide](wasm-transform-guide.md)

## API Reference

- [Debshrew API](api/debshrew.md)
- [Debshrew Runtime API](api/debshrew-runtime.md)
- [Debshrew Support API](api/debshrew-support.md)

## Examples

- [Simple Transform](examples/simple-transform.md)
- [Ordinals Transform](examples/ordinals-transform.md)
- [Custom Sink](examples/custom-sink.md)

## Contributing

- [Development Guide](development.md)
- [Code Style Guide](code-style.md)
- [Testing Guide](testing.md)