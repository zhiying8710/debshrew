//! WASM runtime implementation for debshrew
//!
//! This module provides the WASM runtime implementation for debshrew,
//! including loading and executing WASM modules, providing host functions,
//! and managing WASM memory.

use crate::error::{Error, Result};
use crate::host;
use crate::transform::{DebTransform, TransformResult};
use debshrew_support::{CdcMessage, TransformState};
use std::path::Path;
use wasmtime::{Engine, Instance, Module, Store};

/// WASM runtime for executing transform modules
pub struct WasmRuntime {
    /// The wasmtime engine
    engine: Engine,
    
    /// The WASM module
    module: Module,
    
    /// The current block height
    current_height: u32,
    
    /// The current block hash
    current_hash: Vec<u8>,
    
    /// The transform state
    state: TransformState,
}

impl std::fmt::Debug for WasmRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmRuntime")
            .field("current_height", &self.current_height)
            .field("current_hash", &self.current_hash)
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl WasmRuntime {
    /// Create a new WASM runtime
    ///
    /// # Arguments
    ///
    /// * `wasm_path` - The path to the WASM module
    ///
    /// # Returns
    ///
    /// A new WASM runtime
    ///
    /// # Errors
    ///
    /// Returns an error if the WASM module cannot be loaded
    pub fn new<P: AsRef<Path>>(wasm_path: P) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, wasm_path)
            .map_err(|e| Error::Wasm(format!("Failed to load WASM module: {}", e)))?;

        Ok(Self {
            engine,
            module,
            current_height: 0,
            current_hash: Vec::new(),
            state: TransformState::new(),
        })
    }

    /// Create a new WASM runtime from WASM bytes
    ///
    /// # Arguments
    ///
    /// * `wasm_bytes` - The WASM module bytes
    ///
    /// # Returns
    ///
    /// A new WASM runtime
    ///
    /// # Errors
    ///
    /// Returns an error if the WASM module cannot be loaded
    pub fn from_bytes(wasm_bytes: &[u8]) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::from_binary(&engine, wasm_bytes)
            .map_err(|e| Error::Wasm(format!("Failed to load WASM module from bytes: {}", e)))?;

        Ok(Self {
            engine,
            module,
            current_height: 0,
            current_hash: Vec::new(),
            state: TransformState::new(),
        })
    }

    /// Set the current block height
    ///
    /// # Arguments
    ///
    /// * `height` - The current block height
    pub fn set_current_height(&mut self, height: u32) {
        self.current_height = height;
        host::set_current_height(height);
    }

    /// Set the current block hash
    ///
    /// # Arguments
    ///
    /// * `hash` - The current block hash
    pub fn set_current_hash(&mut self, hash: Vec<u8>) {
        self.current_hash = hash.clone();
        host::set_current_hash(hash);
    }

    /// Set the transform state
    ///
    /// # Arguments
    ///
    /// * `state` - The transform state
    pub fn set_state(&mut self, state: TransformState) {
        self.state = state;
        
        // Update the host state
        host::clear_state();
        for (key, value) in self.state.iter() {
            host::set_state(key, value);
        }
    }

    /// Get the transform state
    ///
    /// # Returns
    ///
    /// The transform state
    pub fn get_state(&self) -> TransformState {
        self.state.clone()
    }

    /// Process a block
    ///
    /// # Arguments
    ///
    /// * `height` - The block height
    /// * `hash` - The block hash
    ///
    /// # Returns
    ///
    /// The result of processing the block, including CDC messages and a state snapshot
    ///
    /// # Errors
    ///
    /// Returns an error if block processing fails
    pub fn process_block(&mut self, height: u32, hash: Vec<u8>) -> Result<TransformResult> {
        // Set the current block height and hash
        self.set_current_height(height);
        self.set_current_hash(hash);
        
        // Update the host state
        host::clear_state();
        for (key, value) in self.state.iter() {
            host::set_state(key, value);
        }
        
        // Create a new store and instance
        let mut store = Store::new(&self.engine, ());
        let instance = Instance::new(&mut store, &self.module, &[])
            .map_err(|e| Error::Wasm(format!("Failed to instantiate WASM module: {}", e)))?;
        
        // Get the _start function
        let start = instance.get_typed_func::<(), ()>(&mut store, "_start")
            .map_err(|e| Error::Wasm(format!("Failed to get _start function: {}", e)))?;
        
        // Call the _start function
        start.call(&mut store, ())
            .map_err(|e| Error::Wasm(format!("Failed to call _start function: {}", e)))?;
        
        // Get the updated state
        let mut updated_state = TransformState::new();
        for key in host::get_state_keys() {
            if let Some(value) = host::get_state(&key) {
                updated_state.set(key, value);
            }
        }
        
        // Get the CDC messages
        // In a real implementation, we would get these from the WASM module
        // For now, we'll just create an empty vector
        let cdc_messages = Vec::new();
        
        // Update the internal state
        self.state = updated_state.clone();
        
        Ok(TransformResult::new(cdc_messages, updated_state))
    }

    /// Handle a rollback
    ///
    /// # Arguments
    ///
    /// * `height` - The height to roll back to
    /// * `hash` - The hash to roll back to
    ///
    /// # Returns
    ///
    /// The result of the rollback, including CDC messages and a state snapshot
    ///
    /// # Errors
    ///
    /// Returns an error if the rollback fails
    pub fn rollback(&mut self, height: u32, hash: Vec<u8>) -> Result<TransformResult> {
        // Set the current block height and hash
        self.set_current_height(height);
        self.set_current_hash(hash);
        
        // Update the host state
        host::clear_state();
        for (key, value) in self.state.iter() {
            host::set_state(key, value);
        }
        
        // Create a new store and instance
        let mut store = Store::new(&self.engine, ());
        let instance = Instance::new(&mut store, &self.module, &[])
            .map_err(|e| Error::Wasm(format!("Failed to instantiate WASM module: {}", e)))?;
        
        // Get the rollback function
        let rollback = instance.get_typed_func::<(), i32>(&mut store, "rollback")
            .map_err(|e| Error::Wasm(format!("Failed to get rollback function: {}", e)))?;
        
        // Call the rollback function
        let result_ptr = rollback.call(&mut store, ())
            .map_err(|e| Error::Wasm(format!("Failed to call rollback function: {}", e)))?;
        
        // Get the memory
        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| Error::Wasm("Failed to get memory".to_string()))?;
        
        // Read the result from memory
        let memory_data = memory.data(&store);
        let result_data = read_string_from_memory(memory_data, result_ptr as usize)
            .map_err(|e| Error::Wasm(format!("Failed to read result from memory: {}", e)))?;
        
        // Deserialize the CDC messages
        let cdc_messages: Vec<CdcMessage> = serde_json::from_str(&result_data)
            .map_err(|e| Error::Serialization(format!("Failed to deserialize CDC messages: {}", e)))?;
        
        // Get the updated state
        let mut updated_state = TransformState::new();
        for key in host::get_state_keys() {
            if let Some(value) = host::get_state(&key) {
                updated_state.set(key, value);
            }
        }
        
        // Update the internal state
        self.state = updated_state.clone();
        
        Ok(TransformResult::new(cdc_messages, updated_state))
    }
/// Register a view function
///
/// # Arguments
///
/// * `name` - The name of the view function
/// * `func` - The view function implementation
pub fn register_view_function(
    &self,
    name: &str,
    func: Box<dyn Fn(&[u8]) -> Result<Vec<u8>> + Send>,
) {
    host::register_view_function(name, func);
}

/// Create a mock WasmRuntime for testing
///
/// This method creates a WasmRuntime with a minimal WASM module that can be used for testing.
///
/// # Returns
///
/// A new WasmRuntime for testing
///
/// # Errors
///
/// Returns an error if the WASM module cannot be created
#[cfg(any(test, feature = "testing"))]
pub fn for_testing() -> Result<Self> {
    use wat::parse_str;
    
    // Create a simple WASM module
    let wasm_bytes = parse_str(
        r#"
        (module
            (func $start (export "_start"))
            (func $rollback (export "rollback") (result i32)
                i32.const 0
            )
            (memory (export "memory") 1)
        )
        "#,
    )
    .map_err(|e| Error::Wasm(format!("Failed to create test WASM module: {}", e)))?;
    
    Self::from_bytes(&wasm_bytes)
}
}

/// Read a string from WASM memory
///
/// # Arguments
///
/// * `memory` - The WASM memory
/// * `ptr` - The pointer to the string
///
/// # Returns
///
/// The string
///
/// # Errors
///
/// Returns an error if the string cannot be read
fn read_string_from_memory(_memory: &[u8], _ptr: usize) -> Result<String> {
    // In a real implementation, we would read the string from memory
    // For now, we'll just return an empty string
    Ok("[]".to_string())
}

/// Mock transform for testing
///
/// This struct implements the `DebTransform` trait for testing purposes.
#[derive(Debug, Default)]
pub struct MockTransform {
    /// The transform state
    pub state: TransformState,
    
    /// The CDC messages to return from process_block
    pub process_block_messages: Vec<CdcMessage>,
    
    /// The CDC messages to return from rollback
    pub rollback_messages: Vec<CdcMessage>,
}

impl DebTransform for MockTransform {
    fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
        Ok(self.process_block_messages.clone())
    }
    
    fn rollback(&mut self) -> Result<Vec<CdcMessage>> {
        Ok(self.rollback_messages.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use debshrew_support::{CdcHeader, CdcOperation, CdcPayload};
    use chrono::Utc;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use wat::parse_str;

    #[test]
    fn test_wasm_runtime_from_bytes() {
        // Create a simple WASM module
        let wasm_bytes = parse_str(
            r#"
            (module
                (func $start (export "_start"))
                (func $rollback (export "rollback") (result i32)
                    i32.const 0
                )
                (memory (export "memory") 1)
            )
            "#,
        )
        .unwrap();

        // Create a WASM runtime
        let runtime = WasmRuntime::from_bytes(&wasm_bytes).unwrap();
        
        // Verify the runtime was created successfully
        assert_eq!(runtime.current_height, 0);
        assert_eq!(runtime.current_hash.len(), 0);
        assert!(runtime.state.is_empty());
    }

    #[test]
    fn test_wasm_runtime_new() {
        // Create a temporary directory
        let dir = tempdir().unwrap();
        let wasm_path = dir.path().join("test.wasm");
        
        // Create a simple WASM module
        let wasm_bytes = parse_str(
            r#"
            (module
                (func $start (export "_start"))
                (func $rollback (export "rollback") (result i32)
                    i32.const 0
                )
                (memory (export "memory") 1)
            )
            "#,
        )
        .unwrap();
        
        // Write the WASM module to a file
        let mut file = File::create(&wasm_path).unwrap();
        file.write_all(&wasm_bytes).unwrap();
        
        // Create a WASM runtime
        let runtime = WasmRuntime::new(&wasm_path).unwrap();
        
        // Verify the runtime was created successfully
        assert_eq!(runtime.current_height, 0);
        assert_eq!(runtime.current_hash.len(), 0);
        assert!(runtime.state.is_empty());
    }

    #[test]
    fn test_mock_transform() {
        let mut transform = MockTransform::default();
        
        // Set up the mock transform
        transform.process_block_messages = vec![CdcMessage {
            header: CdcHeader {
                source: "test".to_string(),
                timestamp: Utc::now(),
                block_height: 123,
                block_hash: "000000000000000000024bead8df69990852c202db0e0097c1a12ea637d7e96d".to_string(),
                transaction_id: None,
            },
            payload: CdcPayload {
                operation: CdcOperation::Create,
                table: "test_table".to_string(),
                key: "test_key".to_string(),
                before: None,
                after: Some(serde_json::json!({
                    "field1": "value1",
                    "field2": 42
                })),
            },
        }];
        
        transform.rollback_messages = vec![CdcMessage {
            header: CdcHeader {
                source: "test".to_string(),
                timestamp: Utc::now(),
                block_height: 122,
                block_hash: "000000000000000000024bead8df69990852c202db0e0097c1a12ea637d7e96d".to_string(),
                transaction_id: None,
            },
            payload: CdcPayload {
                operation: CdcOperation::Delete,
                table: "test_table".to_string(),
                key: "test_key".to_string(),
                before: Some(serde_json::json!({
                    "field1": "value1",
                    "field2": 42
                })),
                after: None,
            },
        }];
        
        // Test process_block
        let messages = transform.process_block().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].payload.table, "test_table");
        assert_eq!(messages[0].payload.operation, CdcOperation::Create);
        
        // Test rollback
        let messages = transform.rollback().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].payload.table, "test_table");
        assert_eq!(messages[0].payload.operation, CdcOperation::Delete);
    }
}