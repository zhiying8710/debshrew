//! Host functions for transform modules
//!
//! This module provides the host functions that are available to transform modules
//! running in the debshrew WASM runtime.

use crate::error::{Error, Result};
use debshrew_support::{deserialize, serialize};
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;

// Thread-local storage for the current block height
thread_local! {
    static CURRENT_HEIGHT: RefCell<u32> = RefCell::new(0);
}

// Thread-local storage for the current block hash
thread_local! {
    static CURRENT_HASH: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

// Thread-local storage for the transform state
thread_local! {
    static TRANSFORM_STATE: RefCell<HashMap<Vec<u8>, Vec<u8>>> = RefCell::new(HashMap::new());
}

/// Global storage for view function implementations
static VIEW_FUNCTIONS: Lazy<Mutex<HashMap<String, Box<dyn Fn(&[u8]) -> Result<Vec<u8>> + Send>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Set the current block height
///
/// This function is called by the runtime to set the current block height
/// before executing a transform module.
///
/// # Arguments
///
/// * `height` - The current block height
pub fn set_current_height(height: u32) {
    CURRENT_HEIGHT.with(|h| {
        *h.borrow_mut() = height;
    });
}

/// Get the current block height
///
/// This function is called by transform modules to get the current block height.
///
/// # Returns
///
/// The current block height
///
/// # Examples
///
/// ```
/// use debshrew_runtime::host;
///
/// host::set_current_height(123);
/// assert_eq!(host::get_height(), 123);
/// ```
pub fn get_height() -> u32 {
    CURRENT_HEIGHT.with(|h| *h.borrow())
}

/// Set the current block hash
///
/// This function is called by the runtime to set the current block hash
/// before executing a transform module.
///
/// # Arguments
///
/// * `hash` - The current block hash
pub fn set_current_hash(hash: Vec<u8>) {
    CURRENT_HASH.with(|h| {
        *h.borrow_mut() = hash;
    });
}

/// Get the current block hash
///
/// This function is called by transform modules to get the current block hash.
///
/// # Returns
///
/// The current block hash
///
/// # Examples
///
/// ```
/// use debshrew_runtime::host;
///
/// host::set_current_hash(vec![1, 2, 3]);
/// assert_eq!(host::get_block_hash(), vec![1, 2, 3]);
/// ```
pub fn get_block_hash() -> Vec<u8> {
    CURRENT_HASH.with(|h| h.borrow().clone())
}

/// Set a value in the transform state
///
/// This function is called by transform modules to set a value in the transform state.
///
/// # Arguments
///
/// * `key` - The key to set
/// * `value` - The value to set
///
/// # Examples
///
/// ```
/// use debshrew_runtime::host;
///
/// host::set_state(b"key", b"value");
/// assert_eq!(host::get_state(b"key"), Some(b"value".to_vec()));
/// ```
pub fn set_state(key: &[u8], value: &[u8]) {
    TRANSFORM_STATE.with(|s| {
        s.borrow_mut().insert(key.to_vec(), value.to_vec());
    });
}

/// Get a value from the transform state
///
/// This function is called by transform modules to get a value from the transform state.
///
/// # Arguments
///
/// * `key` - The key to get
///
/// # Returns
///
/// The value associated with the key, or None if the key doesn't exist
///
/// # Examples
///
/// ```
/// use debshrew_runtime::host;
///
/// host::set_state(b"key", b"value");
/// assert_eq!(host::get_state(b"key"), Some(b"value".to_vec()));
/// assert_eq!(host::get_state(b"nonexistent"), None);
/// ```
pub fn get_state(key: &[u8]) -> Option<Vec<u8>> {
    TRANSFORM_STATE.with(|s| s.borrow().get(key).cloned())
}

/// Delete a value from the transform state
///
/// This function is called by transform modules to delete a value from the transform state.
///
/// # Arguments
///
/// * `key` - The key to delete
///
/// # Returns
///
/// True if the key existed and was removed, false otherwise
///
/// # Examples
///
/// ```
/// use debshrew_runtime::host;
///
/// host::set_state(b"key", b"value");
/// assert_eq!(host::get_state(b"key"), Some(b"value".to_vec()));
///
/// assert_eq!(host::delete_state(b"key"), true);
/// assert_eq!(host::get_state(b"key"), None);
///
/// assert_eq!(host::delete_state(b"nonexistent"), false);
/// ```
pub fn delete_state(key: &[u8]) -> bool {
    TRANSFORM_STATE.with(|s| s.borrow_mut().remove(key).is_some())
}

/// Clear the transform state
///
/// This function is called by the runtime to clear the transform state
/// before executing a transform module.
pub fn clear_state() {
    TRANSFORM_STATE.with(|s| {
        s.borrow_mut().clear();
    });
}

/// Get all keys in the transform state
///
/// This function is called by transform modules to get all keys in the transform state.
///
/// # Returns
///
/// A vector of all keys in the transform state
///
/// # Examples
///
/// ```
/// use debshrew_runtime::host;
///
/// host::clear_state();
/// host::set_state(b"key1", b"value1");
/// host::set_state(b"key2", b"value2");
///
/// let keys = host::get_state_keys();
/// assert_eq!(keys.len(), 2);
/// assert!(keys.contains(&b"key1".to_vec()));
/// assert!(keys.contains(&b"key2".to_vec()));
/// ```
pub fn get_state_keys() -> Vec<Vec<u8>> {
    TRANSFORM_STATE.with(|s| s.borrow().keys().cloned().collect())
}

/// Get all keys in the transform state with a given prefix
///
/// This function is called by transform modules to get all keys in the transform state
/// with a given prefix.
///
/// # Arguments
///
/// * `prefix` - The prefix to match
///
/// # Returns
///
/// A vector of all keys in the transform state with the given prefix
///
/// # Examples
///
/// ```
/// use debshrew_runtime::host;
///
/// host::clear_state();
/// host::set_state(b"prefix1_key1", b"value1");
/// host::set_state(b"prefix1_key2", b"value2");
/// host::set_state(b"prefix2_key1", b"value3");
///
/// let keys = host::get_state_keys_with_prefix(b"prefix1_");
/// assert_eq!(keys.len(), 2);
/// assert!(keys.contains(&b"prefix1_key1".to_vec()));
/// assert!(keys.contains(&b"prefix1_key2".to_vec()));
/// ```
pub fn get_state_keys_with_prefix(prefix: &[u8]) -> Vec<Vec<u8>> {
    TRANSFORM_STATE.with(|s| {
        s.borrow()
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect()
    })
}

/// Register a view function
///
/// This function is called by the runtime to register a view function
/// that can be called by transform modules.
///
/// # Arguments
///
/// * `name` - The name of the view function
/// * `func` - The view function implementation
///
/// # Examples
///
/// ```
/// use debshrew_runtime::host;
///
/// host::register_view_function("get_token_balance", Box::new(|params| {
///     // Parse parameters
///     // Call metashrew view
///     // Return result
///     Ok(vec![42, 0, 0, 0])
/// }));
///
/// let result = host::call_view("get_token_balance", &[]).unwrap();
/// assert_eq!(result, vec![42, 0, 0, 0]);
/// ```
pub fn register_view_function(
    name: &str,
    func: Box<dyn Fn(&[u8]) -> Result<Vec<u8>> + Send>,
) {
    let mut view_functions = VIEW_FUNCTIONS.lock().unwrap();
    view_functions.insert(name.to_string(), func);
}

/// Call a view function
///
/// This function is called by transform modules to call a view function
/// registered by the runtime.
///
/// # Arguments
///
/// * `name` - The name of the view function
/// * `params` - The parameters to pass to the view function
///
/// # Returns
///
/// The result of the view function
///
/// # Errors
///
/// Returns an error if the view function doesn't exist or if the view function
/// returns an error
///
/// # Examples
///
/// ```
/// use debshrew_runtime::host;
///
/// host::register_view_function("get_token_balance", Box::new(|params| {
///     // Parse parameters
///     // Call metashrew view
///     // Return result
///     Ok(vec![42, 0, 0, 0])
/// }));
///
/// let result = host::call_view("get_token_balance", &[]).unwrap();
/// assert_eq!(result, vec![42, 0, 0, 0]);
/// ```
pub fn call_view(name: &str, params: &[u8]) -> Result<Vec<u8>> {
    let view_functions = VIEW_FUNCTIONS.lock().unwrap();
    let func = view_functions
        .get(name)
        .ok_or_else(|| Error::ViewAccess(format!("View function not found: {}", name)))?;
    func(params)
}

/// Log a message
///
/// This function is called by transform modules to log a message.
///
/// # Arguments
///
/// * `message` - The message to log
///
/// # Examples
///
/// ```
/// use debshrew_runtime::host;
///
/// host::log("Hello, world!");
/// ```
pub fn log(message: &str) {
    println!("[Transform] {}", message);
}

/// Allocate a string in WASM memory
///
/// This function is used by the declare_transform macro to allocate a string
/// in WASM memory and return a pointer to it.
///
/// # Arguments
///
/// * `_s` - The string to allocate
///
/// # Returns
///
/// A pointer to the allocated string
pub fn alloc_string(_s: &str) -> i32 {
    // This is a stub implementation for the host environment
    // In the WASM environment, this would allocate memory and return a pointer
    0
}

/// Serialize parameters for a view function
///
/// This function is called by transform modules to serialize parameters
/// for a view function.
///
/// # Arguments
///
/// * `params` - The parameters to serialize
///
/// # Returns
///
/// The serialized parameters
///
/// # Errors
///
/// Returns an error if serialization fails
///
/// # Examples
///
/// ```
/// use debshrew_runtime::host;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Params {
///     token: String,
///     address: String,
/// }
///
/// let params = Params {
///     token: "BTC".to_string(),
///     address: "bc1q...".to_string(),
/// };
///
/// let serialized = host::serialize_params(&params).unwrap();
/// ```
pub fn serialize_params<T: serde::Serialize>(params: &T) -> Result<Vec<u8>> {
    serialize(params).map_err(|e| Error::Serialization(format!("Failed to serialize parameters: {}", e)))
}

/// Deserialize the result of a view function
///
/// This function is called by transform modules to deserialize the result
/// of a view function.
///
/// # Arguments
///
/// * `result` - The result to deserialize
///
/// # Returns
///
/// The deserialized result
///
/// # Errors
///
/// Returns an error if deserialization fails
///
/// # Examples
///
/// ```
/// use debshrew_runtime::host;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct TokenBalance {
///     balance: u64,
/// }
///
/// // Simulate a view function result
/// let result = vec![42, 0, 0, 0, 0, 0, 0, 0]; // u64 = 42 in little-endian
///
/// let balance: TokenBalance = host::deserialize_result(&result).unwrap();
/// assert_eq!(balance.balance, 42);
/// ```
pub fn deserialize_result<T: serde::de::DeserializeOwned>(result: &[u8]) -> Result<T> {
    deserialize(result).map_err(|e| Error::Serialization(format!("Failed to deserialize result: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_height_and_hash() {
        set_current_height(123);
        assert_eq!(get_height(), 123);

        set_current_hash(vec![1, 2, 3]);
        assert_eq!(get_block_hash(), vec![1, 2, 3]);
    }

    #[test]
    fn test_state_management() {
        clear_state();

        // Test set and get
        set_state(b"key1", b"value1");
        set_state(b"key2", b"value2");

        assert_eq!(get_state(b"key1"), Some(b"value1".to_vec()));
        assert_eq!(get_state(b"key2"), Some(b"value2".to_vec()));
        assert_eq!(get_state(b"nonexistent"), None);

        // Test delete
        assert_eq!(delete_state(b"key1"), true);
        assert_eq!(get_state(b"key1"), None);
        assert_eq!(delete_state(b"nonexistent"), false);

        // Test keys
        let keys = get_state_keys();
        assert_eq!(keys.len(), 1);
        assert!(keys.contains(&b"key2".to_vec()));

        // Test keys with prefix
        set_state(b"prefix1_key1", b"value3");
        set_state(b"prefix1_key2", b"value4");
        set_state(b"prefix2_key1", b"value5");

        let prefix1_keys = get_state_keys_with_prefix(b"prefix1_");
        assert_eq!(prefix1_keys.len(), 2);
        assert!(prefix1_keys.contains(&b"prefix1_key1".to_vec()));
        assert!(prefix1_keys.contains(&b"prefix1_key2".to_vec()));

        // Test clear
        clear_state();
        assert_eq!(get_state_keys().len(), 0);
    }

    #[test]
    fn test_view_functions() {
        register_view_function("test_view", Box::new(|params| {
            if params == b"param1" {
                Ok(b"result1".to_vec())
            } else {
                Ok(b"result2".to_vec())
            }
        }));

        assert_eq!(call_view("test_view", b"param1").unwrap(), b"result1".to_vec());
        assert_eq!(call_view("test_view", b"param2").unwrap(), b"result2".to_vec());
        assert!(call_view("nonexistent", b"").is_err());
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestParams {
        token: String,
        address: String,
    }

    #[test]
    fn test_serialization() {
        let params = TestParams {
            token: "BTC".to_string(),
            address: "bc1q...".to_string(),
        };

        let serialized = serialize_params(&params).unwrap();
        let deserialized: TestParams = deserialize_result(&serialized).unwrap();

        assert_eq!(deserialized, params);
    }
}