# CDC Concepts

Change Data Capture (CDC) is a set of software design patterns used to determine and track changes made to data so that action can be taken using the changed data. This document explains CDC concepts in the context of Debshrew.

## What is CDC?

CDC is a technique that captures changes made to data in a database or other data store and makes those changes available to other systems. CDC is commonly used for:

- Data replication
- Event-driven architectures
- Data warehousing and analytics
- Microservices integration
- Audit logging

## CDC in Debshrew

Debshrew uses CDC to capture changes in Bitcoin metaprotocol state and make those changes available to other systems. The key components of Debshrew's CDC implementation are:

1. **Transform Modules**: WASM modules that detect changes in metaprotocol state
2. **CDC Messages**: Standardized messages that represent changes
3. **CDC Sinks**: Components that deliver CDC messages to external systems

## CDC Message Format

Debshrew uses a Debezium-compatible CDC message format:

```json
{
  "header": {
    "source": "my_transform",
    "timestamp": "2023-01-01T00:00:00Z",
    "block_height": 123456,
    "block_hash": "000000000000000000024bead8df69990852c202db0e0097c1a12ea637d7e96d",
    "transaction_id": "tx123"
  },
  "payload": {
    "operation": "create",
    "table": "my_table",
    "key": "my_key",
    "before": null,
    "after": {
      "field1": "value1",
      "field2": 42
    }
  }
}
```

### Header

The header contains metadata about the change:

- **source**: The name of the transform module that generated the message
- **timestamp**: The timestamp when the message was generated
- **block_height**: The height of the block containing the change
- **block_hash**: The hash of the block containing the change
- **transaction_id**: The ID of the transaction containing the change (optional)

### Payload

The payload contains the actual change data:

- **operation**: The type of operation (`create`, `update`, or `delete`)
- **table**: The name of the table or collection being changed
- **key**: The primary key or identifier of the record being changed
- **before**: The state of the record before the change (null for `create` operations)
- **after**: The state of the record after the change (null for `delete` operations)

## CDC Operations

Debshrew supports three types of CDC operations:

### Create

A create operation represents a new record being added to a table:

```json
{
  "operation": "create",
  "table": "my_table",
  "key": "my_key",
  "before": null,
  "after": {
    "field1": "value1",
    "field2": 42
  }
}
```

### Update

An update operation represents an existing record being modified:

```json
{
  "operation": "update",
  "table": "my_table",
  "key": "my_key",
  "before": {
    "field1": "old_value",
    "field2": 42
  },
  "after": {
    "field1": "new_value",
    "field2": 42
  }
}
```

### Delete

A delete operation represents a record being removed:

```json
{
  "operation": "delete",
  "table": "my_table",
  "key": "my_key",
  "before": {
    "field1": "value1",
    "field2": 42
  },
  "after": null
}
```

## CDC and Reorgs

One of the challenges of working with blockchain data is handling reorganizations (reorgs). Debshrew handles reorgs by:

1. Detecting when a reorg has occurred
2. Finding the common ancestor between the old and new chain
3. Rolling back the state to the common ancestor
4. Generating inverse CDC messages for rolled back blocks
5. Processing the new chain from the common ancestor

### Inverse CDC Messages

Inverse CDC messages are CDC messages that undo the effects of previous CDC messages. For example:

- The inverse of a `create` operation is a `delete` operation
- The inverse of a `delete` operation is a `create` operation
- The inverse of an `update` operation is another `update` operation with the before and after states swapped

## CDC Sinks

CDC sinks are responsible for delivering CDC messages to external systems. Debshrew supports several types of sinks:

- **Kafka**: Sends CDC messages to Kafka topics
- **PostgreSQL**: Applies CDC messages to PostgreSQL databases
- **File**: Writes CDC messages to files
- **Console**: Outputs CDC messages to the console
- **Custom**: Implement the `CdcSink` trait for custom sinks

## CDC and Determinism

Debshrew is designed to be deterministic, meaning that given the same input, it will always produce the same output. This is important for CDC because it ensures that:

1. Different instances of Debshrew will generate the same CDC messages
2. Reprocessing blocks will generate the same CDC messages
3. CDC messages can be replayed to reconstruct state

## CDC and Idempotence

Idempotence is the property of certain operations that they can be applied multiple times without changing the result beyond the initial application. Debshrew's CDC messages are designed to be idempotent, meaning that:

1. Applying the same CDC message multiple times has the same effect as applying it once
2. CDC messages can be safely retried in case of failures
3. CDC consumers can deduplicate messages if needed

## CDC and Ordering

CDC messages in Debshrew are ordered by:

1. Block height
2. Transaction index within the block
3. Operation index within the transaction

This ordering ensures that CDC messages are processed in the correct order, which is important for maintaining consistency.