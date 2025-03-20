//! WASM export functions for debshrew-runtime
//!
//! This module provides utility functions for exporting data from WASM modules.

/// Convert a byte slice to an ArrayBuffer layout (length prefix + data)
///
/// # Arguments
///
/// * `v` - The byte slice
pub fn to_arraybuffer_layout<T: AsRef<[u8]>>(v: T) -> Vec<u8> {
    let mut buffer = Vec::<u8>::new();
    buffer.extend_from_slice(&(v.as_ref().len() as u32).to_le_bytes());
    buffer.extend_from_slice(v.as_ref());
    buffer
}

/// Export bytes to the host environment
///
/// This function leaks memory, but that's okay because the host will
/// read the data and then drop the WASM instance.
///
/// # Arguments
///
/// * `v` - The bytes to export
///
/// # Returns
///
/// A pointer to the exported bytes
pub fn export_bytes(v: Vec<u8>) -> i32 {
    let response: Vec<u8> = to_arraybuffer_layout(&v);
    Box::leak(Box::new(response)).as_mut_ptr() as usize as i32
}