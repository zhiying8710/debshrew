use debshrew_runtime::*;

#[derive(Debug, Default, Clone)]
struct MyTransform {
    // State fields
}

impl DebTransform for MyTransform {
    fn process_block(&mut self) -> Result<()> {
        // Get current block info
        let height = get_height();
        let hash = get_block_hash();
        
        // Generate a simple CDC message
        let message = CdcMessage {
            header: CdcHeader {
                source: "my_transform".to_string(),
                timestamp: chrono::Utc::now(),
                block_height: height,
                block_hash: hex::encode(&hash),
                transaction_id: None,
            },
            payload: CdcPayload {
                operation: CdcOperation::Create,
                table: "blocks".to_string(),
                key: height.to_string(),
                before: None,
                after: Some(serde_json::json!({
                    "height": height,
                    "hash": hex::encode(&hash),
                    "timestamp": chrono::Utc::now()
                })),
            },
        };
        // Push CDC message
        self.push_message(message)?;

        // Serialize the parameters
        let params = serialize_params(&height)?;

        // Call the view function
        let result = view("get_block_transactions".to_string(), params)?;

        // Deserialize the result
        let transactions: Vec<Transaction> = deserialize_result(&result)?;
        
        // Push CDC messages for each transaction
        for transaction in transactions {
            let message = CdcMessage {
                header: CdcHeader {
                    source: "my_transform".to_string(),
                    timestamp: chrono::Utc::now(),
                    block_height: height,
                    block_hash: hex::encode(&hash),
                    transaction_id: Some(transaction.id),
                },
                payload: CdcPayload {
                    operation: CdcOperation::Create,
                    table: "transactions".to_string(),
                    key: transaction.id.to_string(),
                    before: None,
                    after: Some(serde_json::to_value(&transaction)?),
                },
            };
            self.push_message(message)?;
        }
        
        
        
        Ok(())
    }
    
    fn rollback(&mut self) -> Result<()> {
        // The default implementation does nothing
        // The runtime will automatically generate inverse CDC messages
        Ok(())
    }
}

// Declare the transform module
declare_transform!(MyTransform);