//! Simple transform module for debshrew
//!
//! This is a simple example transform module that demonstrates how to use
//! the debshrew framework to transform metaprotocol state into CDC streams.

use debshrew_runtime::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simple transform module
#[derive(Debug, Default)]
pub struct SimpleTransform {
    /// Transform state
    state: TransformState,
}

/// Token balance parameters
#[derive(Debug, Serialize, Deserialize)]
struct TokenBalanceParams {
    /// Address to query
    address: String,
}

/// Token balance result
#[derive(Debug, Serialize, Deserialize)]
struct TokenBalanceResult {
    /// Token balances
    balances: HashMap<String, u64>,
}

/// Token transfer parameters
#[derive(Debug, Serialize, Deserialize)]
struct TokenTransferParams {
    /// Block height
    height: u32,
}

/// Token transfer
#[derive(Debug, Serialize, Deserialize)]
struct TokenTransfer {
    /// Transaction ID
    txid: String,
    
    /// From address
    from: String,
    
    /// To address
    to: String,
    
    /// Token symbol
    token: String,
    
    /// Amount
    amount: u64,
}

/// Token transfer result
#[derive(Debug, Serialize, Deserialize)]
struct TokenTransferResult {
    /// Token transfers
    transfers: Vec<TokenTransfer>,
}

impl DebTransform for SimpleTransform {
    fn process_block(&mut self) -> Result<Vec<CdcMessage>> {
        // Get current block info
        let height = get_height();
        let hash = get_block_hash();
        let hash_hex = hex::encode(&hash);
        
        log(&format!("Processing block {} ({})", height, hash_hex));
        
        // Query token transfers in this block
        let params = serialize_params(&TokenTransferParams { height })?;
        let result = call_view("get_token_transfers", &params)?;
        let transfer_result: TokenTransferResult = deserialize_result(&result)?;
        
        let mut messages = Vec::new();
        
        // Process each transfer
        for transfer in &transfer_result.transfers {
            // Update balances
            self.update_balance(&mut messages, &transfer.from, &transfer.token, -(transfer.amount as i64), height, &hash_hex, &transfer.txid)?;
            self.update_balance(&mut messages, &transfer.to, &transfer.token, transfer.amount as i64, height, &hash_hex, &transfer.txid)?;
            
            // Generate transfer CDC message
            let transfer_key = format!("{}:{}", transfer.txid, transfer.token);
            
            messages.push(CdcMessage {
                header: CdcHeader {
                    source: "simple_transform".to_string(),
                    timestamp: chrono::Utc::now(),
                    block_height: height,
                    block_hash: hash_hex.clone(),
                    transaction_id: Some(transfer.txid.clone()),
                },
                payload: CdcPayload {
                    operation: CdcOperation::Create,
                    table: "token_transfers".to_string(),
                    key: transfer_key,
                    before: None,
                    after: Some(serde_json::json!({
                        "from": transfer.from,
                        "to": transfer.to,
                        "token": transfer.token,
                        "amount": transfer.amount,
                        "block_height": height,
                        "txid": transfer.txid,
                    })),
                },
            });
        }
        
        log(&format!("Generated {} CDC messages", messages.len()));
        
        Ok(messages)
    }
    
    fn rollback(&mut self) -> Result<Vec<CdcMessage>> {
        // Get current block info
        let height = get_height();
        let hash = get_block_hash();
        let hash_hex = hex::encode(&hash);
        
        log(&format!("Rolling back to block {} ({})", height, hash_hex));
        
        // Get all keys with the "balance:" prefix and collect them into a vector
        let balance_keys: Vec<Vec<u8>> = self.state.keys_with_prefix(b"balance:").cloned().collect();
        let mut messages = Vec::new();
        
        // Generate inverse CDC messages for balances
        for key in balance_keys {
            if let Ok(key_str) = String::from_utf8(key.clone()) {
                if key_str.starts_with("balance:") {
                    let parts: Vec<&str> = key_str.splitn(3, ':').collect();
                    if parts.len() == 3 {
                        let address = parts[1];
                        let token = parts[2];
                        
                        // Get the current balance
                        if let Some(balance_data) = self.state.get(&key) {
                            let balance: u64 = debshrew_support::deserialize(&balance_data)?;
                            
                            // Query the previous balance from metashrew
                            let params = serialize_params(&TokenBalanceParams {
                                address: address.to_string(),
                            })?;
                            
                            let result = call_view("get_token_balances", &params)?;
                            let balance_result: TokenBalanceResult = deserialize_result(&result)?;
                            
                            let previous_balance = balance_result.balances.get(token).cloned().unwrap_or(0);
                            
                            // Generate CDC message to revert to the previous balance
                            messages.push(CdcMessage {
                                header: CdcHeader {
                                    source: "simple_transform".to_string(),
                                    timestamp: chrono::Utc::now(),
                                    block_height: height,
                                    block_hash: hash_hex.clone(),
                                    transaction_id: None,
                                },
                                payload: CdcPayload {
                                    operation: CdcOperation::Update,
                                    table: "token_balances".to_string(),
                                    key: format!("{}:{}", address, token),
                                    before: Some(serde_json::json!({
                                        "address": address,
                                        "token": token,
                                        "balance": balance,
                                    })),
                                    after: Some(serde_json::json!({
                                        "address": address,
                                        "token": token,
                                        "balance": previous_balance,
                                    })),
                                },
                            });
                            
                            // Update the state
                            self.state.set(key.clone(), debshrew_support::serialize(&previous_balance)?);
                        }
                    }
                }
            }
        }
        
        // Generate inverse CDC messages for transfers
        // In a real implementation, we would query the transfers that need to be rolled back
        // For simplicity, we'll just log a message
        log(&format!("Generated {} CDC messages for rollback", messages.len()));
        
        Ok(messages)
    }
}

impl SimpleTransform {
    /// Update a token balance
    ///
    /// # Arguments
    ///
    /// * `messages` - The CDC messages to append to
    /// * `address` - The address
    /// * `token` - The token symbol
    /// * `amount` - The amount to add (can be negative)
    /// * `height` - The block height
    /// * `hash` - The block hash
    /// * `txid` - The transaction ID
    ///
    /// # Returns
    ///
    /// Ok(()) if the balance was updated successfully
    ///
    /// # Errors
    ///
    /// Returns an error if the balance cannot be updated
    fn update_balance(
        &mut self,
        messages: &mut Vec<CdcMessage>,
        address: &str,
        token: &str,
        amount: i64,
        height: u32,
        hash: &str,
        txid: &str,
    ) -> Result<()> {
        // Get the current balance
        let key = format!("balance:{}:{}", address, token).into_bytes();
        let current_balance = if let Some(data) = self.state.get(&key) {
            debshrew_support::deserialize::<u64>(&data)?
        } else {
            // If the balance doesn't exist, query it from metashrew
            let params = serialize_params(&TokenBalanceParams {
                address: address.to_string(),
            })?;
            
            let result = call_view("get_token_balances", &params)?;
            let balance_result: TokenBalanceResult = deserialize_result(&result)?;
            
            balance_result.balances.get(token).cloned().unwrap_or(0)
        };
        
        // Calculate the new balance
        let new_balance = if amount < 0 {
            let abs_amount = amount.abs() as u64;
            if current_balance < abs_amount {
                return Err(Error::Transform(format!("Insufficient balance: {} < {}", current_balance, abs_amount)));
            }
            current_balance - abs_amount
        } else {
            current_balance + amount as u64
        };
        
        // Generate CDC message
        let balance_key = format!("{}:{}", address, token);
        
        messages.push(CdcMessage {
            header: CdcHeader {
                source: "simple_transform".to_string(),
                timestamp: chrono::Utc::now(),
                block_height: height,
                block_hash: hash.to_string(),
                transaction_id: Some(txid.to_string()),
            },
            payload: CdcPayload {
                operation: if current_balance == 0 {
                    CdcOperation::Create
                } else {
                    CdcOperation::Update
                },
                table: "token_balances".to_string(),
                key: balance_key,
                before: if current_balance == 0 {
                    None
                } else {
                    Some(serde_json::json!({
                        "address": address,
                        "token": token,
                        "balance": current_balance,
                    }))
                },
                after: Some(serde_json::json!({
                    "address": address,
                    "token": token,
                    "balance": new_balance,
                })),
            },
        });
        
        // Update the state
        self.state.set(key, debshrew_support::serialize(&new_balance)?);
        
        Ok(())
    }
}

// Declare the transform module
declare_transform!(SimpleTransform);