# Configuration Guide

Debshrew can be configured using a JSON configuration file or command-line arguments. This guide explains all available configuration options.

## Configuration File

The configuration file is a JSON file with the following structure:

```json
{
  "metashrew": {
    "url": "http://localhost:8080",
    "username": "user",
    "password": "password",
    "timeout": 30,
    "max_retries": 3,
    "retry_delay": 1000
  },
  "transform": {
    "path": "path/to/transform.wasm"
  },
  "sink": {
    "type": "kafka",
    "bootstrap_servers": "localhost:9092",
    "topic": "cdc-events",
    "client_id": "debshrew",
    "batch_size": 100,
    "flush_interval": 1000
  },
  "cache_size": 6,
  "start_height": 100000,
  "log_level": "info"
}
```

## Command-Line Arguments

Debshrew can also be configured using command-line arguments:

```bash
debshrew run \
  --metashrew-url http://localhost:8080 \
  --metashrew-username user \
  --metashrew-password password \
  --metashrew-timeout 30 \
  --metashrew-max-retries 3 \
  --metashrew-retry-delay 1000 \
  --transform path/to/transform.wasm \
  --sink-type kafka \
  --sink-config kafka-config.json \
  --cache-size 6 \
  --start-height 100000 \
  --log-level info
```

## Configuration Options

### Metashrew Configuration

| Option | Description | Default |
|--------|-------------|---------|
| `url` | The URL of the metashrew instance | `http://localhost:8080` |
| `username` | The username for authentication (optional) | None |
| `password` | The password for authentication (optional) | None |
| `timeout` | The timeout for requests in seconds | 30 |
| `max_retries` | The maximum number of retries for failed requests | 3 |
| `retry_delay` | The delay between retries in milliseconds | 1000 |

### Transform Configuration

| Option | Description | Default |
|--------|-------------|---------|
| `path` | The path to the WASM transform module | None (required) |

### Sink Configuration

#### Common Options

| Option | Description | Default |
|--------|-------------|---------|
| `type` | The type of sink (`kafka`, `postgres`, `file`, `console`) | None (required) |

#### Kafka Sink Options

| Option | Description | Default |
|--------|-------------|---------|
| `bootstrap_servers` | The Kafka bootstrap servers | None (required) |
| `topic` | The Kafka topic to send messages to | None (required) |
| `client_id` | The Kafka client ID | `debshrew` |
| `batch_size` | The number of messages to batch before sending | 100 |
| `flush_interval` | The interval to flush messages in milliseconds | 1000 |

#### PostgreSQL Sink Options

| Option | Description | Default |
|--------|-------------|---------|
| `connection_string` | The PostgreSQL connection string | None (required) |
| `schema` | The PostgreSQL schema to use | `public` |
| `batch_size` | The number of messages to batch before sending | 100 |
| `flush_interval` | The interval to flush messages in milliseconds | 1000 |

#### File Sink Options

| Option | Description | Default |
|--------|-------------|---------|
| `path` | The path to the output file | None (required) |
| `append` | Whether to append to the file or overwrite it | `true` |
| `flush_interval` | The interval to flush messages in milliseconds | 1000 |

#### Console Sink Options

| Option | Description | Default |
|--------|-------------|---------|
| `pretty` | Whether to pretty-print the JSON output | `true` |

### General Configuration

| Option | Description | Default |
|--------|-------------|---------|
| `cache_size` | The number of blocks to cache for reorg handling | 6 |
| `start_height` | The block height to start synchronization from | 0 (genesis) |
| `log_level` | The log level (`error`, `warn`, `info`, `debug`, `trace`) | `info` |

## Environment Variables

Debshrew also supports configuration through environment variables. Environment variables take precedence over configuration file values.

Environment variables should be prefixed with `DEBSHREW_` and use underscores instead of dots for nested properties.

For example:

```bash
export DEBSHREW_METASHREW_URL=http://localhost:8080
export DEBSHREW_METASHREW_USERNAME=user
export DEBSHREW_METASHREW_PASSWORD=password
export DEBSHREW_TRANSFORM_PATH=path/to/transform.wasm
export DEBSHREW_SINK_TYPE=kafka
export DEBSHREW_SINK_BOOTSTRAP_SERVERS=localhost:9092
export DEBSHREW_SINK_TOPIC=cdc-events
export DEBSHREW_CACHE_SIZE=6
export DEBSHREW_START_HEIGHT=100000
export DEBSHREW_LOG_LEVEL=info
```

## Configuration Precedence

Debshrew uses the following precedence order for configuration (highest to lowest):

1. Command-line arguments
2. Environment variables
3. Configuration file
4. Default values