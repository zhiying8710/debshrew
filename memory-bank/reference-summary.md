# Debshrew Reference Materials Summary

This document provides an overview of the reference materials created for the debshrew project, explaining how they relate to each other and how they should be used.

## Overview

The debshrew project is a framework for building deterministic CDC (Change Data Capture) streams from Bitcoin metaprotocol state by leveraging metashrew indexing. The reference materials in this directory provide comprehensive documentation on various aspects of the project, from high-level concepts to detailed implementation guides.

## Core Memory Bank Files

The core Memory Bank files provide the foundation for understanding the debshrew project:

1. **projectbrief.md** - The foundation document that defines the core requirements and goals of the debshrew project.
2. **productContext.md** - Explains why debshrew exists, the problems it solves, how it should work, and user experience goals.
3. **systemPatterns.md** - Documents the system architecture, key technical decisions, design patterns, and component relationships.
4. **techContext.md** - Covers the technologies used, development setup, technical constraints, and dependencies.
5. **activeContext.md** - Describes the current work focus, recent changes, next steps, and active decisions.
6. **progress.md** - Tracks what works, what's left to build, current status, and known issues.

## Reference Materials

In addition to the core Memory Bank files, we've created specialized reference materials that provide in-depth information on key aspects of the debshrew project:

### Metashrew Integration Guide

**File**: `metashrew-integration.md`

This guide explains how debshrew integrates with metashrew, which is the primary data source for debshrew. It covers:

- Metashrew architecture and WASM interface
- Metashrew data model and key-value storage
- Debshrew integration points (block synchronization, view access, transform interface, reorg handling)
- Metashrew view functions and memory layout
- Best practices for transform modules

This guide is essential for understanding how debshrew interacts with metashrew and how to leverage metashrew's capabilities in transform modules.

### CDC Concepts and Best Practices

**File**: `cdc-concepts.md`

This document provides a comprehensive overview of Change Data Capture (CDC) concepts, patterns, and best practices as they apply to the debshrew project. It covers:

- Core CDC concepts (event types, event structure, delivery guarantees, consistency models)
- Debshrew CDC implementation (message format, generation process, reorg handling)
- CDC sink types (Kafka, PostgreSQL, file)
- CDC best practices (message design, state management, error handling, performance optimization, testing)
- CDC patterns for metaprotocols (token balance tracking, NFT ownership tracking)

This document is crucial for understanding the CDC aspect of debshrew and how to design effective CDC streams.

### WASM Transform Development Guide

**File**: `wasm-transform-guide.md`

This guide provides detailed information on developing WebAssembly (WASM) transform modules for the debshrew project. It covers:

- Transform module interface (`DebTransform` trait, `process_block` and `rollback` methods)
- Host functions (view access, state management, logging)
- Transform module structure and state management
- CDC message generation patterns
- Calling metashrew views
- Reorg handling strategies
- Error handling and performance optimization
- Testing and debugging transform modules

This guide is essential for developers who want to create transform modules for debshrew.

## How to Use These Materials

The reference materials are designed to be used in conjunction with the core Memory Bank files. Here's a suggested approach for using these materials:

1. Start with **projectbrief.md** to understand the core requirements and goals of the debshrew project.
2. Read **productContext.md** to understand why debshrew exists and the problems it solves.
3. Study **systemPatterns.md** to understand the system architecture and key design patterns.
4. Review **techContext.md** to understand the technologies used and technical constraints.
5. Check **activeContext.md** and **progress.md** to understand the current state of the project.
6. Dive into the specialized reference materials based on your specific needs:
   - If you need to understand how debshrew integrates with metashrew, read **metashrew-integration.md**.
   - If you need to understand CDC concepts and best practices, read **cdc-concepts.md**.
   - If you need to develop transform modules, read **wasm-transform-guide.md**.

## Keeping Reference Materials Up-to-Date

As the debshrew project evolves, it's important to keep the reference materials up-to-date. Here are some guidelines:

1. Update the reference materials when significant changes are made to the corresponding aspects of the project.
2. Ensure that the reference materials remain consistent with the core Memory Bank files.
3. Add new reference materials as needed to cover new aspects of the project.
4. Review and update the reference materials periodically to ensure they remain accurate and useful.

## Conclusion

The reference materials provided in this directory offer a comprehensive understanding of the debshrew project, from high-level concepts to detailed implementation guides. By studying these materials, developers can gain the knowledge needed to effectively work with debshrew and create robust, efficient transform modules that generate high-quality CDC streams from metaprotocol state.