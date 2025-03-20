//! WASM host interface and transform traits for debshrew
//!
//! This crate provides the WASM host interface and transform traits for debshrew,
//! enabling the development of transform modules that convert metaprotocol state
//! into CDC streams.

#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]

pub mod error;
pub mod host;
pub mod transform;
pub mod wasm;

/// Re-export common types and functions for convenience
pub use debshrew_support::{
    deserialize, serialize, serialize_to_json,
    CdcHeader, CdcMessage, CdcOperation, CdcPayload, TransformState,
};
pub use error::{Error, Result};
pub use host::*;
pub use transform::*;
pub use wasm::*;

/// Declare a transform module
///
/// This macro is used to declare a transform module, which implements the
/// `DebTransform` trait. It generates the necessary WASM exports for the
/// transform module to be used by the debshrew runtime.
///
/// # Arguments
///
/// * `transform_type` - The type that implements the `DebTransform` trait
///
/// # Examples
///
/// ```
/// use debshrew_runtime::*;
///
/// #[derive(Default, Debug)]
/// struct ExampleTransform {
///     state: TransformState
/// }
///
/// impl DebTransform for ExampleTransform {
///     fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
///         // Process block and generate CDC messages
///         Ok(vec![])
///     }
///
///     fn rollback(&mut self) -> Result<Vec<CdcMessage>> {
///         // Generate inverse operations for rollback
///         Ok(vec![])
///     }
/// }
///
/// // Declare the transform module
/// declare_transform!(ExampleTransform);
/// ```
#[macro_export]
macro_rules! declare_transform {
    ($transform_type:ty) => {
        static mut TRANSFORM: Option<$transform_type> = None;

        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub extern "C" fn _start() {
            unsafe {
                if TRANSFORM.is_none() {
                    TRANSFORM = Some(<$transform_type>::default());
                }

                if let Some(transform) = TRANSFORM.as_mut() {
                    match transform.process_block() {
                        Ok(_) => {}
                        Err(e) => {
                            $crate::host::log(&format!("Error in process_block: {:?}", e));
                        }
                    }
                }
            }
        }

        #[no_mangle]
        pub extern "C" fn rollback() -> i32 {
            unsafe {
                if TRANSFORM.is_none() {
                    TRANSFORM = Some(<$transform_type>::default());
                }

                if let Some(transform) = TRANSFORM.as_mut() {
                    match transform.rollback() {
                        Ok(messages) => {
                            let messages_json = match $crate::serialize_to_json(&messages) {
                                Ok(json) => json,
                                Err(e) => {
                                    $crate::host::log(&format!("Error serializing messages: {:?}", e));
                                    return -1;
                                }
                            };

                            let ptr = $crate::host::alloc_string(&messages_json);
                            return ptr;
                        }
                        Err(e) => {
                            $crate::host::log(&format!("Error in rollback: {:?}", e));
                            return -1;
                        }
                    }
                }

                -1
            }
        }
    };
}