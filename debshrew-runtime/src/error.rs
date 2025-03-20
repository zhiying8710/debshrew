//! Error types for debshrew-runtime
//!
//! This module defines the error types used in the debshrew-runtime crate.

use thiserror::Error;

/// A specialized Result type for debshrew-runtime operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for debshrew-runtime operations
#[derive(Error, Debug)]
pub enum Error {
    /// Error from debshrew-support
    #[error("Support error: {0}")]
    Support(#[from] debshrew_support::error::Error),

    /// Error occurred during WASM operations
    #[error("WASM error: {0}")]
    Wasm(String),

    /// Error occurred during host function calls
    #[error("Host function error: {0}")]
    HostFunction(String),

    /// Error occurred during transform operations
    #[error("Transform error: {0}")]
    Transform(String),

    /// Error occurred during view access
    #[error("View access error: {0}")]
    ViewAccess(String),

    /// Error occurred during state management
    #[error("State error: {0}")]
    State(String),

    /// Error occurred during CDC message generation
    #[error("CDC message error: {0}")]
    CdcMessage(String),

    /// Error occurred during serialization or deserialization
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Error occurred during memory allocation
    #[error("Memory allocation error: {0}")]
    MemoryAllocation(String),

    /// Generic error with a message
    #[error("{0}")]
    Generic(String),
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

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Generic(format!("I/O error: {}", e))
    }
}

impl From<wasmtime::Error> for Error {
    fn from(e: wasmtime::Error) -> Self {
        Error::Wasm(format!("Wasmtime error: {}", e))
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serialization(format!("JSON error: {}", e))
    }
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Error::Serialization(format!("Bincode error: {}", e))
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

        let error = Error::Wasm("invalid module".to_string());
        assert_eq!(error.to_string(), "WASM error: invalid module");
    }

    #[test]
    fn test_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = Error::from(io_error);
        assert!(matches!(error, Error::Generic(_)));
        assert!(error.to_string().contains("file not found"));
    }
}