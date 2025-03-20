//! Configuration types for debshrew
//!
//! This module defines the configuration types used throughout the debshrew project.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Configuration for the debshrew service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Metashrew client configuration
    pub metashrew: MetashrewConfig,
    
    /// Transform module configuration
    pub transform: TransformConfig,
    
    /// Sink configuration
    pub sink: SinkConfig,
    
    /// Block cache size
    #[serde(default = "default_cache_size")]
    pub cache_size: u32,
    
    /// Starting block height
    #[serde(default)]
    pub start_height: Option<u32>,
    
    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

/// Default cache size
fn default_cache_size() -> u32 {
    6
}

/// Default log level
fn default_log_level() -> String {
    "info".to_string()
}

impl Config {
    /// Load configuration from a file
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the configuration file
    ///
    /// # Returns
    ///
    /// The loaded configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file cannot be loaded or parsed
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)
            .map_err(|e| Error::Configuration(format!("Failed to open configuration file: {}", e)))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| Error::Configuration(format!("Failed to read configuration file: {}", e)))?;
        
        Self::from_str(&contents)
    }
    
    /// Load configuration from a string
    ///
    /// # Arguments
    ///
    /// * `s` - The configuration string
    ///
    /// # Returns
    ///
    /// The loaded configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration string cannot be parsed
    pub fn from_str(s: &str) -> Result<Self> {
        serde_json::from_str(s)
            .map_err(|e| Error::Configuration(format!("Failed to parse configuration: {}", e)))
    }
    
    /// Validate the configuration
    ///
    /// # Returns
    ///
    /// Ok(()) if the configuration is valid, an error otherwise
    pub fn validate(&self) -> Result<()> {
        // Validate metashrew configuration
        self.metashrew.validate()?;
        
        // Validate transform configuration
        self.transform.validate()?;
        
        // Validate sink configuration
        self.sink.validate()?;
        
        // Validate cache size
        if self.cache_size == 0 {
            return Err(Error::Configuration("Cache size must be greater than 0".to_string()));
        }
        
        Ok(())
    }
}

/// Configuration for the metashrew client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetashrewConfig {
    /// Metashrew URL
    pub url: String,
    
    /// Authentication username (optional)
    #[serde(default)]
    pub username: Option<String>,
    
    /// Authentication password (optional)
    #[serde(default)]
    pub password: Option<String>,
    
    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    
    /// Maximum number of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    
    /// Retry delay in milliseconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
}

/// Default timeout
fn default_timeout() -> u64 {
    30
}

/// Default maximum number of retries
fn default_max_retries() -> u32 {
    3
}

/// Default retry delay
fn default_retry_delay() -> u64 {
    1000
}

impl MetashrewConfig {
    /// Validate the metashrew configuration
    ///
    /// # Returns
    ///
    /// Ok(()) if the configuration is valid, an error otherwise
    pub fn validate(&self) -> Result<()> {
        // Validate URL
        url::Url::parse(&self.url)
            .map_err(|e| Error::Configuration(format!("Invalid metashrew URL: {}", e)))?;
        
        // Validate timeout
        if self.timeout == 0 {
            return Err(Error::Configuration("Timeout must be greater than 0".to_string()));
        }
        
        Ok(())
    }
}

/// Configuration for the transform module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformConfig {
    /// Path to the WASM module
    pub path: String,
}

impl TransformConfig {
    /// Validate the transform configuration
    ///
    /// # Returns
    ///
    /// Ok(()) if the configuration is valid, an error otherwise
    pub fn validate(&self) -> Result<()> {
        // Validate path
        if self.path.is_empty() {
            return Err(Error::Configuration("Transform path cannot be empty".to_string()));
        }
        
        // Check if the file exists
        if !Path::new(&self.path).exists() {
            return Err(Error::Configuration(format!("Transform file not found: {}", self.path)));
        }
        
        Ok(())
    }
}

/// Configuration for the CDC sink
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SinkConfig {
    /// Kafka sink configuration
    Kafka {
        /// Bootstrap servers
        bootstrap_servers: String,
        
        /// Topic
        topic: String,
        
        /// Client ID (optional)
        #[serde(default)]
        client_id: Option<String>,
        
        /// Batch size (optional)
        #[serde(default = "default_batch_size")]
        batch_size: usize,
        
        /// Flush interval in milliseconds (optional)
        #[serde(default = "default_flush_interval")]
        flush_interval: u64,
    },
    
    /// PostgreSQL sink configuration
    Postgres {
        /// Connection string
        connection_string: String,
        
        /// Schema (optional)
        #[serde(default = "default_schema")]
        schema: String,
        
        /// Batch size (optional)
        #[serde(default = "default_batch_size")]
        batch_size: usize,
        
        /// Flush interval in milliseconds (optional)
        #[serde(default = "default_flush_interval")]
        flush_interval: u64,
    },
    
    /// File sink configuration
    File {
        /// Path to the output file
        path: String,
        
        /// Append mode (optional)
        #[serde(default = "default_append")]
        append: bool,
        
        /// Flush interval in milliseconds (optional)
        #[serde(default = "default_flush_interval")]
        flush_interval: u64,
    },
    
    /// Console sink configuration
    Console {
        /// Pretty print (optional)
        #[serde(default = "default_pretty_print")]
        pretty_print: bool,
    },
}

/// Default batch size
fn default_batch_size() -> usize {
    100
}

/// Default flush interval
fn default_flush_interval() -> u64 {
    1000
}

/// Default schema
fn default_schema() -> String {
    "public".to_string()
}

/// Default append mode
fn default_append() -> bool {
    true
}

/// Default pretty print
fn default_pretty_print() -> bool {
    false
}

impl SinkConfig {
    /// Validate the sink configuration
    ///
    /// # Returns
    ///
    /// Ok(()) if the configuration is valid, an error otherwise
    pub fn validate(&self) -> Result<()> {
        match self {
            SinkConfig::Kafka { bootstrap_servers, topic, batch_size, .. } => {
                // Validate bootstrap servers
                if bootstrap_servers.is_empty() {
                    return Err(Error::Configuration("Bootstrap servers cannot be empty".to_string()));
                }
                
                // Validate topic
                if topic.is_empty() {
                    return Err(Error::Configuration("Topic cannot be empty".to_string()));
                }
                
                // Validate batch size
                if *batch_size == 0 {
                    return Err(Error::Configuration("Batch size must be greater than 0".to_string()));
                }
            }
            SinkConfig::Postgres { connection_string, batch_size, .. } => {
                // Validate connection string
                if connection_string.is_empty() {
                    return Err(Error::Configuration("Connection string cannot be empty".to_string()));
                }
                
                // Validate batch size
                if *batch_size == 0 {
                    return Err(Error::Configuration("Batch size must be greater than 0".to_string()));
                }
            }
            SinkConfig::File { path, .. } => {
                // Validate path
                if path.is_empty() {
                    return Err(Error::Configuration("File path cannot be empty".to_string()));
                }
                
                // Check if the directory exists
                if let Some(parent) = Path::new(path).parent() {
                    if !parent.exists() {
                        return Err(Error::Configuration(format!("Directory not found: {}", parent.display())));
                    }
                }
            }
            SinkConfig::Console { .. } => {
                // No validation needed for console sink
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_config_from_str() {
        let config_str = r#"
        {
            "metashrew": {
                "url": "http://localhost:8080"
            },
            "transform": {
                "path": "transform.wasm"
            },
            "sink": {
                "type": "kafka",
                "bootstrap_servers": "localhost:9092",
                "topic": "cdc-events"
            }
        }
        "#;
        
        let config = Config::from_str(config_str).unwrap();
        
        assert_eq!(config.metashrew.url, "http://localhost:8080");
        assert_eq!(config.transform.path, "transform.wasm");
        
        match config.sink {
            SinkConfig::Kafka { bootstrap_servers, topic, .. } => {
                assert_eq!(bootstrap_servers, "localhost:9092");
                assert_eq!(topic, "cdc-events");
            }
            _ => panic!("Expected Kafka sink"),
        }
        
        assert_eq!(config.cache_size, 6);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_config_from_file() {
        // Create a temporary directory
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        
        // Create a config file
        let config_str = r#"
        {
            "metashrew": {
                "url": "http://localhost:8080"
            },
            "transform": {
                "path": "transform.wasm"
            },
            "sink": {
                "type": "kafka",
                "bootstrap_servers": "localhost:9092",
                "topic": "cdc-events"
            }
        }
        "#;
        
        let mut file = File::create(&config_path).unwrap();
        file.write_all(config_str.as_bytes()).unwrap();
        
        // Load the config
        let config = Config::from_file(&config_path).unwrap();
        
        assert_eq!(config.metashrew.url, "http://localhost:8080");
        assert_eq!(config.transform.path, "transform.wasm");
        
        match config.sink {
            SinkConfig::Kafka { bootstrap_servers, topic, .. } => {
                assert_eq!(bootstrap_servers, "localhost:9092");
                assert_eq!(topic, "cdc-events");
            }
            _ => panic!("Expected Kafka sink"),
        }
    }

    #[test]
    fn test_sink_config_validation() {
        // Test Kafka sink
        let kafka_sink = SinkConfig::Kafka {
            bootstrap_servers: "localhost:9092".to_string(),
            topic: "cdc-events".to_string(),
            client_id: None,
            batch_size: 100,
            flush_interval: 1000,
        };
        
        assert!(kafka_sink.validate().is_ok());
        
        // Test invalid Kafka sink
        let invalid_kafka_sink = SinkConfig::Kafka {
            bootstrap_servers: "".to_string(),
            topic: "cdc-events".to_string(),
            client_id: None,
            batch_size: 100,
            flush_interval: 1000,
        };
        
        assert!(invalid_kafka_sink.validate().is_err());
        
        // Test PostgreSQL sink
        let postgres_sink = SinkConfig::Postgres {
            connection_string: "postgres://user:password@localhost/db".to_string(),
            schema: "public".to_string(),
            batch_size: 100,
            flush_interval: 1000,
        };
        
        assert!(postgres_sink.validate().is_ok());
        
        // Test invalid PostgreSQL sink
        let invalid_postgres_sink = SinkConfig::Postgres {
            connection_string: "".to_string(),
            schema: "public".to_string(),
            batch_size: 100,
            flush_interval: 1000,
        };
        
        assert!(invalid_postgres_sink.validate().is_err());
        
        // Test File sink
        let file_sink = SinkConfig::File {
            path: "output.json".to_string(),
            append: true,
            flush_interval: 1000,
        };
        
        // This will fail because the directory doesn't exist in the test environment
        assert!(file_sink.validate().is_err());
        
        // Test Console sink
        let console_sink = SinkConfig::Console {
            pretty_print: false,
        };
        
        assert!(console_sink.validate().is_ok());
    }
}