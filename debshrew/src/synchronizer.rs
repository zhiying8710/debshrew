//! Block synchronization with metashrew
//!
//! This module provides the block synchronizer, which is responsible for
//! synchronizing with metashrew, processing blocks, and handling reorgs.

use crate::block::BlockCache;
use crate::client::MetashrewClient;
use crate::error::{Error, Result};
use crate::sink::CdcSink;
use async_trait::async_trait;
use chrono::Utc;
use debshrew_runtime::WasmRuntime;
use debshrew_support::BlockMetadata;
use log::{debug, info, warn};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;

/// Block synchronizer
///
/// The block synchronizer is responsible for synchronizing with metashrew,
/// processing blocks, and handling reorgs.
pub struct BlockSynchronizer<C: MetashrewClient> {
    /// The metashrew client
    client: Arc<C>,
    
    /// The WASM runtime
    runtime: Arc<Mutex<WasmRuntime>>,
    
    /// The CDC sink
    sink: Arc<Box<dyn CdcSink>>,
    
    /// The block cache
    cache: Arc<Mutex<BlockCache>>,
    
    /// The current block height
    current_height: u32,
    
    /// Whether the synchronizer is running
    running: bool,
    
    /// The polling interval in milliseconds
    polling_interval: u64,
}

impl<C: MetashrewClient> BlockSynchronizer<C> {
    /// Create a new block synchronizer
    ///
    /// # Arguments
    ///
    /// * `client` - The metashrew client
    /// * `runtime` - The WASM runtime
    /// * `sink` - The CDC sink
    /// * `cache_size` - The block cache size
    ///
    /// # Returns
    ///
    /// A new block synchronizer
    ///
    /// # Errors
    ///
    /// Returns an error if the block synchronizer cannot be created
    pub fn new(client: C, runtime: WasmRuntime, sink: Box<dyn CdcSink>, cache_size: u32) -> Result<Self> {
        let cache = BlockCache::new(cache_size)?;
        
        Ok(Self {
            client: Arc::new(client),
            runtime: Arc::new(Mutex::new(runtime)),
            sink: Arc::new(sink),
            cache: Arc::new(Mutex::new(cache)),
            current_height: 0,
            running: false,
            polling_interval: 1000,
        })
    }
    
    /// Set the polling interval
    ///
    /// # Arguments
    ///
    /// * `interval` - The polling interval in milliseconds
    pub fn set_polling_interval(&mut self, interval: u64) {
        self.polling_interval = interval;
    }
    
    /// Set the starting block height
    ///
    /// # Arguments
    ///
    /// * `height` - The starting block height
    pub fn set_starting_height(&mut self, height: u32) {
        self.current_height = height;
    }
    
    /// Run the block synchronizer
    ///
    /// This method starts the block synchronizer and runs until stopped.
    ///
    /// # Returns
    ///
    /// Ok(()) if the synchronizer ran successfully
    ///
    /// # Errors
    ///
    /// Returns an error if the synchronizer encounters an error
    pub async fn run(&mut self) -> Result<()> {
        self.running = true;
        
        // If the current height is 0, get the latest height from metashrew
        if self.current_height == 0 {
            self.current_height = self.client.get_height().await?;
            info!("Starting at block height {}", self.current_height);
        }
        
        // Main synchronization loop
        while self.running {
            // Poll metashrew for the latest height
            let metashrew_height = self.client.get_height().await?;
            
            // Check if we need to process new blocks
            if metashrew_height > self.current_height {
                info!("Processing blocks {} to {}", self.current_height + 1, metashrew_height);
                
                // Process new blocks
                for height in (self.current_height + 1)..=metashrew_height {
                    self.process_block(height).await?;
                    self.current_height = height;
                }
            } else if metashrew_height < self.current_height {
                // Handle reorg
                warn!("Chain reorganization detected: metashrew height {} < current height {}", metashrew_height, self.current_height);
                self.handle_reorg(metashrew_height).await?;
                self.current_height = metashrew_height;
            }
            
            // Sleep for the polling interval
            time::sleep(Duration::from_millis(self.polling_interval)).await;
        }
        
        Ok(())
    }
    
    /// Stop the block synchronizer
    pub fn stop(&mut self) {
        self.running = false;
    }
    
    /// Process a block
    ///
    /// # Arguments
    ///
    /// * `height` - The block height
    ///
    /// # Returns
    ///
    /// Ok(()) if the block was processed successfully
    ///
    /// # Errors
    ///
    /// Returns an error if the block cannot be processed
    async fn process_block(&self, height: u32) -> Result<()> {
        // Get the block hash
        let hash = self.client.get_block_hash(height).await?;
        
        // Create block metadata
        let metadata = BlockMetadata {
            height,
            hash: hex::encode(&hash),
            timestamp: Utc::now(),
        };
        
        // Process the block with the transform module
        let mut runtime = self.runtime.lock().await;
        let transform_result = runtime.process_block(height, hash)?;
        
        // Add the block to the cache
        let mut cache = self.cache.lock().await;
        cache.add_block(metadata, transform_result.clone())?;
        
        // Send the CDC messages to the sink
        self.sink.send(transform_result.cdc_messages).await?;
        
        debug!("Processed block {}", height);
        
        Ok(())
    }
    
    /// Handle a chain reorganization
    ///
    /// # Arguments
    ///
    /// * `new_height` - The new block height
    ///
    /// # Returns
    ///
    /// Ok(()) if the reorg was handled successfully
    ///
    /// # Errors
    ///
    /// Returns an error if the reorg cannot be handled
    async fn handle_reorg(&self, new_height: u32) -> Result<()> {
        // Get the block hashes for the new chain
        let mut new_hashes = Vec::new();
        for height in 0..=new_height {
            let hash = self.client.get_block_hash(height).await?;
            new_hashes.push((height, hex::encode(&hash)));
        }
        
        // Find the common ancestor
        let cache = self.cache.lock().await;
        let common_ancestor = cache.find_common_ancestor(&new_hashes)
            .ok_or_else(|| Error::ReorgHandling("No common ancestor found".to_string()))?;
        
        info!("Found common ancestor at height {}", common_ancestor);
        
        // Get the state snapshot at the common ancestor
        let state_snapshot = cache.get_state_snapshot(common_ancestor)
            .ok_or_else(|| Error::ReorgHandling(format!("State snapshot not found for height {}", common_ancestor)))?;
        
        // Release the cache lock
        drop(cache);
        
        // Roll back the transform module to the common ancestor
        let mut runtime = self.runtime.lock().await;
        runtime.set_current_height(common_ancestor);
        runtime.set_state(state_snapshot);
        
        // Generate inverse CDC messages for the rolled back blocks
        let hash = self.client.get_block_hash(common_ancestor).await?;
        let rollback_result = runtime.rollback(common_ancestor, hash)?;
        
        // Send the inverse CDC messages to the sink
        self.sink.send(rollback_result.cdc_messages).await?;
        
        // Roll back the cache
        let mut cache = self.cache.lock().await;
        cache.rollback(common_ancestor)?;
        
        // Process the new chain
        for height in (common_ancestor + 1)..=new_height {
            // Release the cache lock
            drop(cache);
            
            // Process the block
            self.process_block(height).await?;
            
            // Reacquire the cache lock
            cache = self.cache.lock().await;
        }
        
        Ok(())
    }
    
    /// Get the current block height
    ///
    /// # Returns
    ///
    /// The current block height
    pub fn get_current_height(&self) -> u32 {
        self.current_height
    }
    
    /// Get the block cache
    ///
    /// # Returns
    ///
    /// The block cache
    pub async fn get_cache(&self) -> Arc<Mutex<BlockCache>> {
        self.cache.clone()
    }
    
    /// Get the WASM runtime
    ///
    /// # Returns
    ///
    /// The WASM runtime
    pub async fn get_runtime(&self) -> Arc<Mutex<WasmRuntime>> {
        self.runtime.clone()
    }
    
    /// Get the CDC sink
    ///
    /// # Returns
    ///
    /// The CDC sink
    pub fn get_sink(&self) -> Arc<Box<dyn CdcSink>> {
        self.sink.clone()
    }
    
    /// Get the metashrew client
    ///
    /// # Returns
    ///
    /// The metashrew client
    pub fn get_client(&self) -> Arc<C> {
        self.client.clone()
    }
}

/// Synchronizer trait
///
/// This trait defines the interface for block synchronizers.
#[async_trait]
pub trait Synchronizer: Send + Sync {
    /// Run the synchronizer
    ///
    /// # Returns
    ///
    /// Ok(()) if the synchronizer ran successfully
    ///
    /// # Errors
    ///
    /// Returns an error if the synchronizer encounters an error
    async fn run(&mut self) -> Result<()>;
    
    /// Stop the synchronizer
    fn stop(&mut self);
    
    /// Get the current block height
    ///
    /// # Returns
    ///
    /// The current block height
    fn get_current_height(&self) -> u32;
}

#[async_trait]
impl<C: MetashrewClient> Synchronizer for BlockSynchronizer<C> {
    async fn run(&mut self) -> Result<()> {
        self.run().await
    }
    
    fn stop(&mut self) {
        self.stop();
    }
    
    fn get_current_height(&self) -> u32 {
        self.get_current_height()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MockMetashrewClient;
    use crate::sink::{ConsoleSink, FileSink, NullSink};
    use debshrew_runtime::MockTransform;
    use debshrew_support::{CdcHeader, CdcMessage, CdcOperation, CdcPayload};
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::runtime::Runtime;

    #[test]
    fn test_block_synchronizer() {
        // Create a mock metashrew client
        let mut client = MockMetashrewClient::new();
        client.set_height(10);
        
        for i in 0..=10 {
            client.set_block_hash(i, vec![i as u8]);
        }
        
        // Create a mock transform
        let _transform = MockTransform::default();
        
        // Create a WASM runtime for testing
        let runtime = WasmRuntime::for_testing().unwrap();
        
        // Create a null sink
        let sink = Box::new(NullSink::new());
        
        // Create a block synchronizer
        let mut synchronizer = BlockSynchronizer::new(client, runtime, sink, 6).unwrap();
        
        // Set the starting height
        synchronizer.set_starting_height(5);
        
        // Set a short polling interval
        synchronizer.set_polling_interval(10);
        
        // Create a runtime for async tests
        let rt = Runtime::new().unwrap();
        
        // Run the synchronizer for a short time
        let handle = rt.spawn(async move {
            let _ = synchronizer.run().await;
        });
        
        // Wait for a short time
        std::thread::sleep(Duration::from_millis(100));
        
        // Abort the task
        handle.abort();
    }
    
    #[test]
    fn test_block_synchronizer_with_different_sinks() {
        // Create a mock metashrew client
        let mut client = MockMetashrewClient::new();
        client.set_height(10);
        
        for i in 0..=10 {
            client.set_block_hash(i, vec![i as u8]);
        }
        
        // We don't need this runtime since we create a new one for each test
        // let runtime = WasmRuntime::from_bytes(&[0]).unwrap();
        
        // Create a tokio runtime for async tests
        let rt = Runtime::new().unwrap();
        
        // Create a new runtime for each test to avoid cloning
        
        // Test with ConsoleSink
        {
            let console_sink = Box::new(ConsoleSink::new(false));
            let test_runtime = WasmRuntime::for_testing().unwrap();
            let synchronizer = BlockSynchronizer::new(client.clone(), test_runtime, console_sink, 6).unwrap();
            
            // Verify the sink type
            let sink = synchronizer.get_sink();
            rt.block_on(async {
                // Send a test message to verify the sink works
                let message = create_test_message();
                let result = sink.send(vec![message]).await;
                assert!(result.is_ok());
                
                // Flush and close the sink
                assert!(sink.flush().await.is_ok());
                assert!(sink.close().await.is_ok());
            });
        }
        
        // Test with FileSink
        {
            // Create a temporary directory for the file
            let dir = tempdir().unwrap();
            let file_path = dir.path().join("test.json");
            
            let file_sink = Box::new(FileSink::new(file_path.to_str().unwrap(), false, 1000).unwrap());
            let test_runtime = WasmRuntime::for_testing().unwrap();
            let synchronizer = BlockSynchronizer::new(client.clone(), test_runtime, file_sink, 6).unwrap();
            
            // Verify the sink type
            let sink = synchronizer.get_sink();
            rt.block_on(async {
                // Send a test message to verify the sink works
                let message = create_test_message();
                let result = sink.send(vec![message]).await;
                assert!(result.is_ok());
                
                // Flush and close the sink
                assert!(sink.flush().await.is_ok());
                assert!(sink.close().await.is_ok());
            });
        }
        
        // Test with a custom sink implementation
        {
            // Create a custom sink that counts messages
            #[derive(Clone)]
            struct CountingSink {
                count: Arc<tokio::sync::Mutex<usize>>,
            }
            
            impl CountingSink {
                fn new() -> Self {
                    Self {
                        count: Arc::new(tokio::sync::Mutex::new(0)),
                    }
                }
                
                async fn get_count(&self) -> usize {
                    let count = self.count.lock().await;
                    *count
                }
            }
            
            #[async_trait]
            impl CdcSink for CountingSink {
                async fn send(&self, messages: Vec<CdcMessage>) -> Result<()> {
                    let mut count = self.count.lock().await;
                    *count += messages.len();
                    Ok(())
                }
                
                async fn flush(&self) -> Result<()> {
                    Ok(())
                }
                
                async fn close(&self) -> Result<()> {
                    Ok(())
                }
            }
            
            let counting_sink = CountingSink::new();
            let counting_sink_clone = counting_sink.clone();
            let test_runtime = WasmRuntime::for_testing().unwrap();
            let synchronizer = BlockSynchronizer::new(client.clone(), test_runtime, Box::new(counting_sink), 6).unwrap();
            
            // Verify the sink works
            rt.block_on(async {
                // Get the sink from the synchronizer
                let sink = synchronizer.get_sink();
                
                // Send test messages
                let messages = vec![
                    create_test_message(),
                    create_test_message(),
                    create_test_message(),
                ];
                
                let result = sink.send(messages).await;
                assert!(result.is_ok());
                
                // Verify the count
                assert_eq!(counting_sink_clone.get_count().await, 3);
            });
        }
    }
    
    // Helper function to create a test CDC message
    fn create_test_message() -> CdcMessage {
        CdcMessage {
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
        }
    }
}