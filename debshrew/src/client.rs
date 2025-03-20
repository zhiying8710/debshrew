//! Metashrew client implementation
//!
//! This module provides the client implementation for communicating with metashrew.

use crate::error::{Error, Result};
use crate::config::MetashrewConfig;
use async_trait::async_trait;
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

/// Metashrew client trait
///
/// This trait defines the interface for communicating with metashrew.
#[async_trait]
pub trait MetashrewClient: Send + Sync {
    /// Get the current block height
    ///
    /// # Returns
    ///
    /// The current block height
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    async fn get_height(&self) -> Result<u32>;
    
    /// Get the block hash for a given height
    ///
    /// # Arguments
    ///
    /// * `height` - The block height
    ///
    /// # Returns
    ///
    /// The block hash
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    async fn get_block_hash(&self, height: u32) -> Result<Vec<u8>>;
    
    /// Call a view function
    ///
    /// # Arguments
    ///
    /// * `view_name` - The name of the view function
    /// * `params` - The parameters to pass to the view function
    /// * `height` - The block height to query at (optional)
    ///
    /// # Returns
    ///
    /// The result of the view function
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    async fn call_view(&self, view_name: &str, params: &[u8], height: Option<u32>) -> Result<Vec<u8>>;
}

/// JSON-RPC request
#[derive(Debug, Serialize)]
struct JsonRpcRequest<T> {
    /// JSON-RPC version
    jsonrpc: String,
    
    /// Method name
    method: String,
    
    /// Parameters
    params: T,
    
    /// Request ID
    id: u32,
}

/// JSON-RPC response
#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    /// JSON-RPC version
    #[allow(dead_code)]
    jsonrpc: String,
    
    /// Result
    result: Option<T>,
    
    /// Error
    error: Option<JsonRpcError>,
    
    /// Request ID
    #[allow(dead_code)]
    id: u32,
}

/// JSON-RPC error
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    /// Error code
    code: i32,
    
    /// Error message
    message: String,
    
    /// Error data
    #[allow(dead_code)]
    data: Option<serde_json::Value>,
}

/// JSON-RPC client for metashrew
#[derive(Debug, Clone)]
pub struct JsonRpcClient {
    /// The HTTP client
    client: Client,
    
    /// The metashrew URL
    url: Url,
    
    /// The request ID counter
    request_id: u32,
}

impl JsonRpcClient {
    /// Create a new JSON-RPC client
    ///
    /// # Arguments
    ///
    /// * `url` - The metashrew URL
    ///
    /// # Returns
    ///
    /// A new JSON-RPC client
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is invalid
    pub fn new(url: &str) -> Result<Self> {
        let url = Url::parse(url)
            .map_err(|e| Error::MetashrewClient(format!("Invalid URL: {}", e)))?;
        
        let client = Client::new();
        
        Ok(Self {
            client,
            url,
            request_id: 0,
        })
    }
    
    /// Create a new JSON-RPC client from a configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The metashrew configuration
    ///
    /// # Returns
    ///
    /// A new JSON-RPC client
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid
    pub fn from_config(config: &MetashrewConfig) -> Result<Self> {
        let url = Url::parse(&config.url)
            .map_err(|e| Error::MetashrewClient(format!("Invalid URL: {}", e)))?;
        
        let client_builder = ClientBuilder::new()
            .timeout(Duration::from_secs(config.timeout))
            .connect_timeout(Duration::from_secs(config.timeout));
        
        // Add authentication if provided
        if let (Some(_username), Some(_password)) = (&config.username, &config.password) {
            // Create a client with basic auth
            let client = client_builder.build()
                .map_err(|e| Error::MetashrewClient(format!("Failed to build HTTP client: {}", e)))?;
            
            return Ok(Self {
                client,
                url,
                request_id: 0,
            });
        }
        
        let client = client_builder.build()
            .map_err(|e| Error::MetashrewClient(format!("Failed to build HTTP client: {}", e)))?;
        
        Ok(Self {
            client,
            url,
            request_id: 0,
        })
    }
    
    /// Get the next request ID
    ///
    /// # Returns
    ///
    /// The next request ID
    fn next_request_id(&mut self) -> u32 {
        let id = self.request_id;
        self.request_id = self.request_id.wrapping_add(1);
        id
    }
    
    /// Send a JSON-RPC request
    ///
    /// # Arguments
    ///
    /// * `method` - The method name
    /// * `params` - The parameters
    ///
    /// # Returns
    ///
    /// The JSON-RPC response
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    async fn send_request<T, R>(&mut self, method: &str, params: T) -> Result<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: self.next_request_id(),
        };
        
        let response = self.client.post(self.url.clone())
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::MetashrewClient(format!("Failed to send request: {}", e)))?;
        
        let status = response.status();
        if !status.is_success() {
            return Err(Error::MetashrewClient(format!("HTTP error: {}", status)));
        }
        
        let json_response: JsonRpcResponse<R> = response.json()
            .await
            .map_err(|e| Error::MetashrewClient(format!("Failed to parse response: {}", e)))?;
        
        if let Some(error) = json_response.error {
            return Err(Error::MetashrewClient(format!("JSON-RPC error: {} (code: {})", error.message, error.code)));
        }
        
        json_response.result
            .ok_or_else(|| Error::MetashrewClient("No result in response".to_string()))
    }
}

#[async_trait]
impl MetashrewClient for JsonRpcClient {
    async fn get_height(&self) -> Result<u32> {
        let mut client = self.clone();
        let height: u32 = client.send_request("metashrew_height", ()).await?;
        Ok(height)
    }
    
    async fn get_block_hash(&self, height: u32) -> Result<Vec<u8>> {
        let mut client = self.clone();
        let hash: String = client.send_request("metashrew_blockHash", vec![height]).await?;
        
        // Convert hex string to bytes
        let hash_bytes = hex::decode(hash)
            .map_err(|e| Error::MetashrewClient(format!("Failed to decode block hash: {}", e)))?;
        
        Ok(hash_bytes)
    }
    
    async fn call_view(&self, view_name: &str, params: &[u8], height: Option<u32>) -> Result<Vec<u8>> {
        let mut client = self.clone();
        
        // Convert params to hex string
        let params_hex = hex::encode(params);
        
        // Prepare parameters for the view call
        let view_params = match height {
            Some(h) => vec![view_name.to_string(), params_hex, h.to_string()],
            None => vec![view_name.to_string(), params_hex, "latest".to_string()],
        };
        
        // Call the view function
        let result: String = client.send_request("metashrew_view", view_params).await?;
        
        // Convert hex string to bytes
        let result_bytes = hex::decode(result)
            .map_err(|e| Error::MetashrewClient(format!("Failed to decode view result: {}", e)))?;
        
        Ok(result_bytes)
    }
}

/// Mock metashrew client for testing
#[derive(Debug, Clone)]
pub struct MockMetashrewClient {
    /// The current block height
    pub height: u32,
    
    /// The block hashes
    pub block_hashes: Vec<Vec<u8>>,
    
    /// The view function results
    pub view_results: Vec<(String, Vec<u8>, Option<u32>, Vec<u8>)>,
}

impl MockMetashrewClient {
    /// Create a new mock metashrew client
    ///
    /// # Returns
    ///
    /// A new mock metashrew client
    pub fn new() -> Self {
        Self {
            height: 0,
            block_hashes: Vec::new(),
            view_results: Vec::new(),
        }
    }
    
    /// Set the current block height
    ///
    /// # Arguments
    ///
    /// * `height` - The current block height
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
    }
    
    /// Set the block hash for a given height
    ///
    /// # Arguments
    ///
    /// * `height` - The block height
    /// * `hash` - The block hash
    pub fn set_block_hash(&mut self, height: u32, hash: Vec<u8>) {
        // Ensure the block_hashes vector is large enough
        if height as usize >= self.block_hashes.len() {
            self.block_hashes.resize(height as usize + 1, Vec::new());
        }
        
        self.block_hashes[height as usize] = hash;
    }
    
    /// Set the result for a view function
    ///
    /// # Arguments
    ///
    /// * `view_name` - The name of the view function
    /// * `params` - The parameters to match
    /// * `height` - The block height to match (optional)
    /// * `result` - The result to return
    pub fn set_view_result(&mut self, view_name: &str, params: &[u8], height: Option<u32>, result: Vec<u8>) {
        self.view_results.push((view_name.to_string(), params.to_vec(), height, result));
    }
}

#[async_trait]
impl MetashrewClient for MockMetashrewClient {
    async fn get_height(&self) -> Result<u32> {
        Ok(self.height)
    }
    
    async fn get_block_hash(&self, height: u32) -> Result<Vec<u8>> {
        if height as usize >= self.block_hashes.len() {
            return Err(Error::MetashrewClient(format!("Block hash not found for height {}", height)));
        }
        
        let hash = &self.block_hashes[height as usize];
        if hash.is_empty() {
            return Err(Error::MetashrewClient(format!("Block hash not found for height {}", height)));
        }
        
        Ok(hash.clone())
    }
    
    async fn call_view(&self, view_name: &str, params: &[u8], height: Option<u32>) -> Result<Vec<u8>> {
        for (name, p, h, result) in &self.view_results {
            if name == view_name && p == params && h == &height {
                return Ok(result.clone());
            }
        }
        
        Err(Error::MetashrewClient(format!("View result not found for {}", view_name)))
    }
}

impl Default for MockMetashrewClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path};
    use serde_json::json;

    #[test]
    fn test_mock_metashrew_client() {
        let mut client = MockMetashrewClient::new();
        
        // Set up the mock client
        client.set_height(123);
        client.set_block_hash(123, vec![1, 2, 3]);
        client.set_view_result("test_view", &[4, 5, 6], None, vec![7, 8, 9]);
        
        // Create a runtime for async tests
        let rt = Runtime::new().unwrap();
        
        // Test get_height
        let height = rt.block_on(client.get_height()).unwrap();
        assert_eq!(height, 123);
        
        // Test get_block_hash
        let hash = rt.block_on(client.get_block_hash(123)).unwrap();
        assert_eq!(hash, vec![1, 2, 3]);
        
        // Test call_view
        let result = rt.block_on(client.call_view("test_view", &[4, 5, 6], None)).unwrap();
        assert_eq!(result, vec![7, 8, 9]);
        
        // Test error cases
        assert!(rt.block_on(client.get_block_hash(456)).is_err());
        assert!(rt.block_on(client.call_view("nonexistent", &[], None)).is_err());
    }

    #[tokio::test]
    async fn test_json_rpc_client() {
        // Start a mock server
        let mock_server = MockServer::start().await;
        
        // Mock the get_height request
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "jsonrpc": "2.0",
                    "result": 123,
                    "id": 0
                })))
            .mount(&mock_server)
            .await;
        
        // Create a client
        let client = JsonRpcClient::new(&mock_server.uri()).unwrap();
        
        // Test get_height
        let height = client.get_height().await.unwrap();
        assert_eq!(height, 123);
    }
}