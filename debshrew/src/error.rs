//! Error types for debshrew
//!
//! This module defines the error types used throughout the debshrew project.

use thiserror::Error;

/// A specialized Result type for debshrew operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for debshrew operations
#[derive(Error, Debug)]
pub enum Error {
    /// Error from debshrew-runtime
    #[error("Runtime error: {0}")]
    Runtime(#[from] debshrew_runtime::error::Error),

    /// Error from debshrew-support
    #[error("Support error: {0}")]
    Support(#[from] debshrew_support::error::Error),

    /// Error occurred during metashrew client operations
    #[error("Metashrew client error: {0}")]
    MetashrewClient(String),

    /// Error occurred during block synchronization
    #[error("Block synchronization error: {0}")]
    BlockSynchronization(String),

    /// Error occurred during reorg handling
    #[error("Reorg handling error: {0}")]
    ReorgHandling(String),

    /// Error occurred during sink operations
    #[error("Sink error: {0}")]
    Sink(String),

    /// Error occurred during configuration
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Error occurred during I/O operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Error occurred during HTTP operations
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Error occurred during JSON operations
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error occurred during URL parsing
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    /// Error occurred during Kafka operations
    #[error("Kafka error: {0}")]
    Kafka(String),

    /// Error occurred during PostgreSQL operations
    #[error("PostgreSQL error: {0}")]
    Postgres(String),

    /// Error occurred during file operations
    #[error("File error: {0}")]
    File(String),

    /// Generic error with a message
    #[error("{0}")]
    Generic(String),
    
    /// Anyhow error
    #[error("Error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Generic(s.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Generic(s)
    }
}

impl From<rdkafka::error::KafkaError> for Error {
    fn from(e: rdkafka::error::KafkaError) -> Self {
        Error::Kafka(format!("Kafka error: {}", e))
    }
}

impl From<postgres::Error> for Error {
    fn from(e: postgres::Error) -> Self {
        Error::Postgres(format!("PostgreSQL error: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_from_string() {
        let error = Error::from("test error");
        assert!(matches!(error, Error::Generic(_)));
        if let Error::Generic(msg) = error {
            assert_eq!(msg, "test error");
        }
    }

    #[test]
    fn test_error_display() {
        let error = Error::Generic("test error".to_string());
        assert_eq!(error.to_string(), "test error");

        let error = Error::MetashrewClient("connection failed".to_string());
        assert_eq!(error.to_string(), "Metashrew client error: connection failed");
    }

    #[test]
    fn test_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = Error::from(io_error);
        assert!(matches!(error, Error::Io(_)));
        assert!(error.to_string().contains("file not found"));
    }
}