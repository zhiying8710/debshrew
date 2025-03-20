//! Common utilities and shared code for debshrew
//!
//! This crate provides common utilities and shared code for the debshrew project,
//! including error types, serialization helpers, and other shared functionality.

#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]

pub mod error;
pub mod serialization;
pub mod types;
pub mod utils;

/// Re-export common types and functions for convenience
pub use error::{Error, Result};
pub use serialization::{deserialize, serialize, serialize_to_json};
pub use types::*;