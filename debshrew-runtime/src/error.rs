//! Error types for debshrew-runtime
//!
//! This module provides error types for debshrew-runtime.

use std::fmt;

/// Error type for debshrew-runtime
#[derive(Debug)]
pub enum Error {
    /// WASM error
    Wasm(String),
    
    /// View access error
    ViewAccess(String),
    
    /// Serialization error
    Serialization(String),
    
    /// State error
    State(String),
    
    /// CDC message error
    CdcMessage(String),
    
    /// Other error
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Wasm(msg) => write!(f, "WASM error: {}", msg),
            Error::ViewAccess(msg) => write!(f, "View access error: {}", msg),
            Error::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Error::State(msg) => write!(f, "State error: {}", msg),
            Error::CdcMessage(msg) => write!(f, "CDC message error: {}", msg),
            Error::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for debshrew-runtime
pub type Result<T> = std::result::Result<T, anyhow::Error>;