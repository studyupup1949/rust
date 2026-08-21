//! Helper utilities matching Dart SDK convenience features
//!
//! This module provides high-level convenience APIs similar to the Dart SDK:
//! - SmartSigner: Auto-version tracking for transaction signing
//! - TxBody: Factory methods for creating transaction bodies
//! - KeyManager: Key page query and management
//! - QuickStart: Ultra-simple API for rapid development
//! - Polling utilities: Wait for balance, credits, transactions

#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::client::AccumulateClient;
use crate::json_rpc_client::JsonRpcError;
use crate::AccOptions;
use ed25519_dalek::{SigningKey, Signer};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::Url;

// =============================================================================
// KERMIT TESTNET ENDPOINTS
// =============================================================================

/// Kermit public testnet V2 endpoint
pub const KERMIT_V2: &str = "https://kermit.accumulatenetwork.io/v2";
/// Kermit public testnet V3 endpoint
pub const KERMIT_V3: &str = "https://kermit.accumulatenetwork.io/v3";

/// Local DevNet V2 endpoint
pub const DEVNET_V2: &str = "http://127.0.0.1:26660/v2";
/// Local DevNet V3 endpoint
pub const DEVNET_V3: &str = "http://127.0.0.1:26660/v3";

// =============================================================================
// TRANSACTION RESULT
// =============================================================================

/// Result of a transaction submission with wait
#[derive(Debug, Clone)]
pub struct TxResult {
    /// Whether the transaction succeeded
    pub success: bool,
    /// Transaction ID (if successful)
    pub txid: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Raw response data
    pub response: Option<Value>,
}

impl TxResult {
    /// Create a success result
    pub fn ok(txid: String, response: Value) -> Self {
        Self {
            success: true,
            txid: Some(txid),
            error: None,
            response: Some(response),
        }
    }

    /// Create a failure result
    pub fn err(error: String) -> Self {
        Self {
            success: false,
            txid: None,
            error: Some(error),
            response: None,
        }
    }
}

// =============================================================================
// TX BODY HELPERS
// =============================================================================

/// Factory methods for creating transaction bodies (matching Dart SDK TxBody)
#[derive(Debug)]
pub struct TxBody;

impl TxBody {
    /// Create an AddCredits transaction body
    pub fn add_credits(recipient: &str, amount: &str, oracle: u64) -> Value {
        json!({
            "type": "addCredits",
            "recipient": recipient,
            "amount": amount,
            "oracle": oracle
        })
    }

    /// Create a CreateIdentity transaction body
    pub fn create_identity(url: &str, key_book_url: &str, public_key_hash: &str) -> Value {
        json!({
            "type": "createIdentity",
            "url": url,
            "keyBookUrl": key_book_url,
            "keyHash": public_key_hash
        })
    }

    /// Create a CreateTokenAccount transaction body
    pub fn create_token_account(url: &str, token_url: &str) -> Value {
        json!({
            "type": "createTokenAccount",
            "url": url,
            "tokenUrl": token_url
        })
    }

    /// Create a CreateDataAccount transaction body
    pub fn create_data_account(url: &str) -> Value {
        json!({
            "type": "createDataAccount",
            "url": url
        })
    }

    /// Create a CreateToken transaction body
    pub fn create_token(url: &str, symbol: &str, precision: u64, supply_limit: Option<&str>) -> Value {
        let mut body = json!({
            "type": "createToken",
            "url": url,
            "symbol": symbol,
            "precision": precision
        });
        if let Some(limit) = supply_limit {
            body["supplyLimit"] = json!(limit);
        }
        body
    }

    /// Create a SendTokens transaction body for a single recipient
    pub fn send_tokens_single(to_url: &str, amount: &str) -> Value {
        json!({
            "type": "sendTokens",
            "to": [{
                "url": to_url,
                "amount": amount
            }]
        })
    }

    /// Create a SendTokens transaction body for multiple recipients
    pub fn send_tokens_multi(recipients: &[(&str, &str)]) -> Value {
        let to: Vec<Value> = recipients
            .iter()
            .map(|(url, amount)| json!({"url": url, "amount": amount}))
            .collect();
        json!({
            "type": "sendTokens",
            "to": to
        })
    }

    /// Create an IssueTokens transaction body for a single recipient
    pub fn issue_tokens_single(to_url: &str, amount: &str) -> Value {
        json!({
            "type": "issueTokens",
            "to": [{
                "url": to_url,
                "amount": amount
            }]
        })
    }

    /// Create a WriteData transaction body
    /// Data entries are hex-encoded and sent as a DoubleHash data entry
    pub fn write_data(entries: &[&str]) -> Value {
        // Convert each entry to hex string (not nested objects!)
        let entries_hex: Vec<Value> = entries
            .iter()
            .map(|e| Value::String(hex::encode(e.as_bytes())))
            .collect();
        json!({
            "type": "writeData",
            "entry": {
                "type": "doublehash",  // DataEntryType = 3
                "data": entries_hex    // Array of hex strings
            }
        })
    }

    /// Create a WriteData transaction body with hex entries
    pub fn write_data_hex(entries_hex: &[&str]) -> Value {
        // Convert to Value::String (not nested objects!)
        let entries: Vec<Value> = entries_hex
            .iter()
            .map(|e| Value::String((*e).to_string()))
            .collect();
        json!({
            "type": "writeData",
            "entry": {
                "type": "doublehash",  // DataEntryType = 3
                "data": entries        // Array of hex strings
            }
        })
    }

    /// Create a CreateKeyPage transaction body
    pub fn create_key_page(key_hashes: &[&[u8]]) -> Value {
        let keys: Vec<Value> = key_hashes
            .iter()
            .map(|h| json!({"publicKeyHash": hex::encode(h)}))
            .collect();
        json!({
            "type": "createKeyPage",
            "keys": keys
        })
    }

    /// Create a CreateKeyBook transaction body
    pub fn create_key_book(url: &str, public_key_hash: &str) -> Value {
        json!({
            "type": "createKeyBook",
            "url": url,
            "publicKeyHash": public_key_hash
        })
    }

    /// Create an UpdateKeyPage transaction body to add a key
    pub fn update_key_page_add_key(key_hash: &[u8]) -> Value {
        json!({
            "type": "updateKeyPage",
            "operation": [{
                "type": "add",
                "entry": {
                    "keyHash": hex::encode(key_hash)
                }
            }]
        })
    }

    /// Create an UpdateKeyPage transaction body to remove a key
    pub fn update_key_page_remove_key(key_hash: &[u8]) -> Value {
        json!({
            "type": "updateKeyPage",
            "operation": [{
                "type": "remove",
                "entry": {
                    "keyHash": hex::encode(key_hash)
                }
            }]
        })
    }

    /// Create an UpdateKeyPage transaction body to set threshold
    pub fn update_key_page_set_threshold(threshold: u64) -> Value {
        json!({
            "type": "updateKeyPage",
            "operation": [{
                "type": "setThreshold",
                "threshold": threshold
            }]
        })
    }

    /// Create a BurnTokens transaction body
    pub fn burn_tokens(amount: &str) -> Value {
        json!({
            "type": "burnTokens",
            "amount": amount
        })
    }

    /// Create a TransferCredits transaction body
    pub fn transfer_credits(to_url: &str, amount: u64) -> Value {
        json!({
            "type": "transferCredits",
            "to": [{
                "url": to_url,
                "amount": amount
            }]
        })
    }
}

// =============================================================================
// KEY PAGE STATE
// =============================================================================

/// Key page state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPageState {
    /// Key page URL
    pub url: String,
    /// Current version
    pub version: u64,
    /// Credit balance
    pub credit_balance: u64,
    /// Accept threshold (signatures required)
    pub accept_threshold: u64,
    /// Keys on the page
    pub keys: Vec<KeyEntry>,
}

/// Key entry in a key page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEntry {
    /// Public key hash (hex)
    pub key_hash: String,
    /// Delegate (if any)
    pub delegate: Option<String>,
}

// =============================================================================
// SMART SIGNER
// =============================================================================

/// Smart signer with auto-version tracking (matching Dart SDK SmartSigner)
#[derive(Debug)]
pub struct SmartSigner<'a> {
    /// Reference to the client
    client: &'a AccumulateClient,
    /// Signing key
    keypair: SigningKey,
    /// Signer URL (key page URL)
    signer_url: String,
    /// Cached version (updated automatically)
    cached_version: u64,
}

impl<'a> SmartSigner<'a> {
    /// Create a new SmartSigner
    pub fn new(client: &'a AccumulateClient, keypair: SigningKey, signer_url: &str) -> Self {
        Self {
            client,
            keypair,
            signer_url: signer_url.to_string(),
            cached_version: 1,
        }
    }

    /// Query and update the cached version
    pub async fn refresh_version(&mut self) -> Result<u64, JsonRpcError> {
        let params = json!({
            "scope": &self.signer_url,
            "query": {"queryType": "default"}
        });

        let result: Value = self.client.v3_client.call_v3("query", params).await?;

        if let Some(account) = result.get("account") {
            if let Some(version) = account.get("version").and_then(|v| v.as_u64()) {
                self.cached_version = version;
                return Ok(version);
            }
        }

        Ok(self.cached_version)
    }

    /// Get the current cached version
    pub fn version(&self) -> u64 {
        self.cached_version
    }

    /// Sign a transaction and return the envelope
    ///
    /// This uses proper binary encoding matching the Go core implementation:
    /// 1. Compute signature metadata hash (binary encoded)
    /// 2. Use that as the transaction initiator
    /// 3. Compute transaction hash using binary encoding
    /// 4. Create signing preimage = SHA256(sigMdHash + txHash)
    /// 5. Sign the preimage
    pub fn sign(&self, principal: &str, body: &Value, memo: Option<&str>) -> Result<Value, JsonRpcError> {
        use crate::codec::signing::{
            compute_ed25519_signature_metadata_hash,
            compute_transaction_hash,
            compute_write_data_body_hash,
            create_signing_preimage,
            marshal_transaction_header,
            sha256_bytes,
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| JsonRpcError::General(anyhow::anyhow!("Time error: {}", e)))?
            .as_micros() as u64;

        let public_key = self.keypair.verifying_key().to_bytes();

        // Step 1: Compute signature metadata hash
        // This is used as BOTH the transaction initiator AND for signing
        let sig_metadata_hash = compute_ed25519_signature_metadata_hash(
            &public_key,
            &self.signer_url,
            self.cached_version,
            timestamp,
        );
        let initiator_hex = hex::encode(&sig_metadata_hash);

        // Step 2: Marshal header with initiator
        let header_bytes = marshal_transaction_header(
            principal,
            &sig_metadata_hash,
            memo,
            None,
        );

        // Step 3 & 4: Compute transaction hash
        // For WriteData, use special Merkle hash algorithm
        let tx_type = body.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let tx_hash = if tx_type == "writeData" || tx_type == "writeDataTo" {
            // WriteData uses special hash: MerkleHash([bodyPartHash, entryHash])
            let header_hash = sha256_bytes(&header_bytes);

            // Extract entries from body.entry.data
            let mut entries_hex = Vec::new();
            if let Some(entry) = body.get("entry") {
                if let Some(data) = entry.get("data") {
                    if let Some(arr) = data.as_array() {
                        for item in arr {
                            if let Some(s) = item.as_str() {
                                entries_hex.push(s.to_string());
                            }
                        }
                    }
                }
            }
            let scratch = body.get("scratch").and_then(|s| s.as_bool()).unwrap_or(false);
            let write_to_state = body.get("writeToState").and_then(|w| w.as_bool()).unwrap_or(false);

            let body_hash = compute_write_data_body_hash(&entries_hex, scratch, write_to_state);

            // txHash = SHA256(SHA256(header) + bodyHash)
            let mut combined = Vec::with_capacity(64);
            combined.extend_from_slice(&header_hash);
            combined.extend_from_slice(&body_hash);
            sha256_bytes(&combined)
        } else {
            // Standard: SHA256(SHA256(header) + SHA256(body))
            let body_bytes = marshal_body_to_binary(body)?;
            compute_transaction_hash(&header_bytes, &body_bytes)
        };

        // Step 5: Create signing preimage and sign
        let preimage = create_signing_preimage(&sig_metadata_hash, &tx_hash);
        let signature = self.keypair.sign(&preimage);

        // Build transaction JSON (for submission)
        let mut tx = json!({
            "header": {
                "principal": principal,
                "initiator": &initiator_hex
            },
            "body": body
        });

        if let Some(m) = memo {
            tx["header"]["memo"] = json!(m);
        }

        // Build envelope with proper signature document
        // V3 API expects: envelope.signatures[].transactionHash
        let envelope = json!({
            "transaction": [tx],
            "signatures": [{
                "type": "ed25519",
                "publicKey": hex::encode(&public_key),
                "signature": hex::encode(signature.to_bytes()),
                "signer": &self.signer_url,
                "signerVersion": self.cached_version,
                "timestamp": timestamp,
                "transactionHash": hex::encode(&tx_hash)
            }]
        });

        Ok(envelope)
    }

    /// Sign, submit, and wait for transaction confirmation
    pub async fn sign_submit_and_wait(
        &mut self,
        principal: &str,
        body: &Value,
        memo: Option<&str>,
        max_attempts: u32,
    ) -> TxResult {
        // Refresh version before signing
        if let Err(e) = self.refresh_version().await {
            return TxResult::err(format!("Failed to refresh version: {}", e));
        }

        // Sign the transaction
        let envelope = match self.sign(principal, body, memo) {
            Ok(env) => env,
            Err(e) => return TxResult::err(format!("Failed to sign: {}", e)),
        };

        // Submit
        let submit_result: Result<Value, _> = self.client.v3_client.call_v3("submit", json!({
            "envelope": envelope
        })).await;

        let response = match submit_result {
            Ok(resp) => resp,
            Err(e) => return TxResult::err(format!("Submit failed: {}", e)),
        };

        // Extract transaction ID
        let txid = extract_txid(&response);
        if txid.is_none() {
            return TxResult::err("No transaction ID in response".to_string());
        }
        let txid = txid.unwrap();

        // Wait for confirmation
        // Extract just the hash for querying - format: acc://hash@unknown
        let tx_hash = if txid.starts_with("acc://") && txid.contains('@') {
            txid.split('@').next().unwrap_or(&txid).replace("acc://", "")
        } else {
            txid.clone()
        };
        let query_scope = format!("acc://{}@unknown", tx_hash);

        for _attempt in 0..max_attempts {
            tokio::time::sleep(Duration::from_secs(2)).await;

            // Query transaction status
            let query_result: Result<Value, _> = self.client.v3_client.call_v3("query", json!({
                "scope": &query_scope,
                "query": {"queryType": "default"}
            })).await;

            if let Ok(result) = query_result {
                // Check status - can be a String or a Map (matching Dart SDK)
                if let Some(status_value) = result.get("status") {
                    // Case 1: Status is a simple string like "delivered" or "pending"
                    if let Some(status_str) = status_value.as_str() {
                        if status_str == "delivered" {
                            return TxResult::ok(txid, response);
                        }
                        // "pending" - continue waiting
                        continue;
                    }

                    // Case 2: Status is a map with delivered/failed fields
                    if status_value.is_object() {
                        let delivered = status_value.get("delivered")
                            .and_then(|d| d.as_bool())
                            .unwrap_or(false);

                        if delivered {
                            // Check for errors
                            let failed = status_value.get("failed")
                                .and_then(|f| f.as_bool())
                                .unwrap_or(false);

                            if failed {
                                let error_msg = status_value.get("error")
                                    .and_then(|e| {
                                        if let Some(msg) = e.get("message").and_then(|m| m.as_str()) {
                                            Some(msg.to_string())
                                        } else {
                                            e.as_str().map(String::from)
                                        }
                                    })
                                    .unwrap_or_else(|| "Unknown error".to_string());
                                return TxResult::err(error_msg);
                            }

                            return TxResult::ok(txid, response);
                        }
                    }
                }
            }
        }

        TxResult::err(format!("Timeout waiting for delivery: {}", txid))
    }

    /// Add a key to the key page using SmartSigner
    pub async fn add_key(&mut self, public_key: &[u8]) -> TxResult {
        let key_hash = sha256_hash(public_key);
        let body = TxBody::update_key_page_add_key(&key_hash);
        self.sign_submit_and_wait(&self.signer_url.clone(), &body, Some("Add key"), 30).await
    }

    /// Remove a key from the key page using SmartSigner
    pub async fn remove_key(&mut self, public_key_hash: &[u8]) -> TxResult {
        let body = TxBody::update_key_page_remove_key(public_key_hash);
        self.sign_submit_and_wait(&self.signer_url.clone(), &body, Some("Remove key"), 30).await
    }

    /// Set the threshold for the key page
    pub async fn set_threshold(&mut self, threshold: u64) -> TxResult {
        let body = TxBody::update_key_page_set_threshold(threshold);
        self.sign_submit_and_wait(&self.signer_url.clone(), &body, Some("Set threshold"), 30).await
    }

    /// Get public key hash
    #[allow(dead_code)]
    fn public_key_hash(&self) -> [u8; 32] {
        sha256_hash(&self.keypair.verifying_key().to_bytes())
    }
}

/// Marshal a JSON transaction body to binary format
///
/// This handles different transaction types and converts them to proper binary encoding.
fn marshal_body_to_binary(body: &Value) -> Result<Vec<u8>, JsonRpcError> {
    use crate::codec::signing::{
        marshal_add_credits_body, marshal_send_tokens_body, marshal_create_identity_body,
        marshal_create_token_account_body, marshal_create_data_account_body,
        marshal_write_data_body, marshal_create_token_body, marshal_issue_tokens_body,
        marshal_key_page_operation, marshal_update_key_page_body,
        tx_types
    };
    use crate::codec::writer::BinaryWriter;

    let tx_type = body.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match tx_type {
        "addCredits" => {
            let recipient = body.get("recipient").and_then(|r| r.as_str()).unwrap_or("");
            let amount_str = body.get("amount").and_then(|a| a.as_str()).unwrap_or("0");
            let amount: u64 = amount_str.parse().unwrap_or(0);
            let oracle = body.get("oracle").and_then(|o| o.as_u64()).unwrap_or(0);
            Ok(marshal_add_credits_body(recipient, amount, oracle))
        }
        "sendTokens" => {
            let to_array = body.get("to").and_then(|t| t.as_array());
            let mut recipients = Vec::new();
            if let Some(to) = to_array {
                for recipient in to {
                    let url = recipient.get("url").and_then(|u| u.as_str()).unwrap_or("");
                    let amount_str = recipient.get("amount").and_then(|a| a.as_str()).unwrap_or("0");
                    let amount: u64 = amount_str.parse().unwrap_or(0);
                    recipients.push((url.to_string(), amount));
                }
            }
            Ok(marshal_send_tokens_body(&recipients))
        }
        "createIdentity" => {
            let url = body.get("url").and_then(|u| u.as_str()).unwrap_or("");
            let key_book_url = body.get("keyBookUrl").and_then(|k| k.as_str()).unwrap_or("");
            // Check both "keyHash" (preferred) and "publicKeyHash" (fallback)
            let key_hash_hex = body.get("keyHash")
                .or_else(|| body.get("publicKeyHash"))
                .and_then(|k| k.as_str())
                .unwrap_or("");
            let key_hash = hex::decode(key_hash_hex).unwrap_or_default();
            Ok(marshal_create_identity_body(url, &key_hash, key_book_url))
        }
        "createTokenAccount" => {
            let url = body.get("url").and_then(|u| u.as_str()).unwrap_or("");
            let token_url = body.get("tokenUrl").and_then(|t| t.as_str()).unwrap_or("");
            Ok(marshal_create_token_account_body(url, token_url))
        }
        "createDataAccount" => {
            let url = body.get("url").and_then(|u| u.as_str()).unwrap_or("");
            Ok(marshal_create_data_account_body(url))
        }
        "writeData" => {
            // Extract entries from nested entry.data structure
            let mut entries_hex = Vec::new();
            if let Some(entry) = body.get("entry") {
                if let Some(data) = entry.get("data") {
                    if let Some(arr) = data.as_array() {
                        for item in arr {
                            if let Some(s) = item.as_str() {
                                entries_hex.push(s.to_string());
                            }
                        }
                    }
                }
            }
            let scratch = body.get("scratch").and_then(|s| s.as_bool()).unwrap_or(false);
            let write_to_state = body.get("writeToState").and_then(|w| w.as_bool()).unwrap_or(false);
            Ok(marshal_write_data_body(&entries_hex, scratch, write_to_state))
        }
        "createToken" => {
            let url = body.get("url").and_then(|u| u.as_str()).unwrap_or("");
            let symbol = body.get("symbol").and_then(|s| s.as_str()).unwrap_or("");
            let precision = body.get("precision").and_then(|p| p.as_u64()).unwrap_or(0);
            let supply_limit = body.get("supplyLimit")
                .and_then(|s| s.as_str())
                .and_then(|s| s.parse::<u64>().ok());
            Ok(marshal_create_token_body(url, symbol, precision, supply_limit))
        }
        "issueTokens" => {
            let to_array = body.get("to").and_then(|t| t.as_array());
            let mut recipients: Vec<(&str, u64)> = Vec::new();
            if let Some(to) = to_array {
                for recipient in to {
                    let url = recipient.get("url").and_then(|u| u.as_str()).unwrap_or("");
                    let amount_str = recipient.get("amount").and_then(|a| a.as_str()).unwrap_or("0");
                    let amount: u64 = amount_str.parse().unwrap_or(0);
                    recipients.push((url, amount));
                }
            }
            Ok(marshal_issue_tokens_body(&recipients))
        }
        "updateKeyPage" => {
            // Parse operations array from JSON
            let op_array = body.get("operation").and_then(|o| o.as_array());
            let mut operations: Vec<Vec<u8>> = Vec::new();

            if let Some(ops) = op_array {
                for op in ops {
                    let op_type = op.get("type").and_then(|t| t.as_str()).unwrap_or("");

                    // Extract key hash from entry.keyHash (for add/remove operations)
                    // Go uses "keyHash" field in KeySpecParams
                    let key_hash: Option<Vec<u8>> = op.get("entry")
                        .and_then(|e| e.get("keyHash"))
                        .and_then(|h| h.as_str())
                        .and_then(|hex_str| hex::decode(hex_str).ok());

                    // Extract delegate URL if present
                    let delegate: Option<&str> = op.get("entry")
                        .and_then(|e| e.get("delegate"))
                        .and_then(|d| d.as_str());

                    // Extract old/new key hashes for update operation
                    let old_key_hash: Option<Vec<u8>> = op.get("oldEntry")
                        .and_then(|e| e.get("keyHash"))
                        .and_then(|h| h.as_str())
                        .and_then(|hex_str| hex::decode(hex_str).ok());

                    let new_key_hash: Option<Vec<u8>> = op.get("newEntry")
                        .and_then(|e| e.get("keyHash"))
                        .and_then(|h| h.as_str())
                        .and_then(|hex_str| hex::decode(hex_str).ok());

                    // Extract threshold for setThreshold operation
                    let threshold: Option<u64> = op.get("threshold").and_then(|t| t.as_u64());

                    // Marshal the operation
                    let op_bytes = marshal_key_page_operation(
                        op_type,
                        key_hash.as_deref(),
                        delegate,
                        old_key_hash.as_deref(),
                        new_key_hash.as_deref(),
                        threshold,
                    );
                    operations.push(op_bytes);
                }
            }

            Ok(marshal_update_key_page_body(&operations))
        }
        // For other transaction types, fall back to JSON encoding
        // This won't produce correct signatures but allows compilation
        _ => {
            // Create a minimal binary encoding with just the type
            let mut writer = BinaryWriter::new();

            // Map type string to numeric type
            let type_num = match tx_type {
                "createIdentity" => tx_types::CREATE_IDENTITY,
                "createTokenAccount" => tx_types::CREATE_TOKEN_ACCOUNT,
                "createDataAccount" => tx_types::CREATE_DATA_ACCOUNT,
                "writeData" => tx_types::WRITE_DATA,
                "writeDataTo" => tx_types::WRITE_DATA_TO,
                "acmeFaucet" => tx_types::ACME_FAUCET,
                "createToken" => tx_types::CREATE_TOKEN,
                "issueTokens" => tx_types::ISSUE_TOKENS,
                "burnTokens" => tx_types::BURN_TOKENS,
                "createLiteTokenAccount" => tx_types::CREATE_LITE_TOKEN_ACCOUNT,
                "createKeyPage" => tx_types::CREATE_KEY_PAGE,
                "createKeyBook" => tx_types::CREATE_KEY_BOOK,
                "updateKeyPage" => tx_types::UPDATE_KEY_PAGE,
                "updateAccountAuth" => tx_types::UPDATE_ACCOUNT_AUTH,
                "updateKey" => tx_types::UPDATE_KEY,
                "lockAccount" => tx_types::LOCK_ACCOUNT,
                "transferCredits" => tx_types::TRANSFER_CREDITS,
                "burnCredits" => tx_types::BURN_CREDITS,
                _ => 0,
            };

            // Write field 1: Type
            let _ = writer.write_uvarint(1);
            let _ = writer.write_uvarint(type_num);

            // Just return the type encoding - proper implementation needed
            // for each transaction type
            // TODO: Implement full binary encoding for all transaction types
            Ok(writer.into_bytes())
        }
    }
}

// =============================================================================
// KEY MANAGER
// =============================================================================

/// Key manager for key page operations (matching Dart SDK KeyManager)
#[derive(Debug)]
pub struct KeyManager<'a> {
    /// Reference to the client
    client: &'a AccumulateClient,
    /// Key page URL
    key_page_url: String,
}

impl<'a> KeyManager<'a> {
    /// Create a new KeyManager
    pub fn new(client: &'a AccumulateClient, key_page_url: &str) -> Self {
        Self {
            client,
            key_page_url: key_page_url.to_string(),
        }
    }

    /// Get the current key page state
    pub async fn get_key_page_state(&self) -> Result<KeyPageState, JsonRpcError> {
        let params = json!({
            "scope": &self.key_page_url,
            "query": {"queryType": "default"}
        });

        let result: Value = self.client.v3_client.call_v3("query", params).await?;

        let account = result.get("account")
            .ok_or_else(|| JsonRpcError::General(anyhow::anyhow!("No account in response")))?;

        let url = account.get("url")
            .and_then(|v| v.as_str())
            .unwrap_or(&self.key_page_url)
            .to_string();

        let version = account.get("version")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);

        let credit_balance = account.get("creditBalance")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let accept_threshold = account.get("acceptThreshold")
            .or_else(|| account.get("threshold"))
            .and_then(|v| v.as_u64())
            .unwrap_or(1);

        let keys: Vec<KeyEntry> = if let Some(keys_arr) = account.get("keys").and_then(|k| k.as_array()) {
            keys_arr.iter().map(|k| {
                let key_hash = k.get("publicKeyHash")
                    .or_else(|| k.get("publicKey"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let delegate = k.get("delegate").and_then(|v| v.as_str()).map(String::from);
                KeyEntry { key_hash, delegate }
            }).collect()
        } else {
            vec![]
        };

        Ok(KeyPageState {
            url,
            version,
            credit_balance,
            accept_threshold,
            keys,
        })
    }
}

// =============================================================================
// POLLING UTILITIES
// =============================================================================

/// Poll for account balance (matching Dart SDK pollForBalance)
pub async fn poll_for_balance(
    client: &AccumulateClient,
    account_url: &str,
    max_attempts: u32,
) -> Option<u64> {
    for i in 0..max_attempts {
        let params = json!({
            "scope": account_url,
            "query": {"queryType": "default"}
        });

        match client.v3_client.call_v3::<Value>("query", params).await {
            Ok(result) => {
                if let Some(account) = result.get("account") {
                    // Try balance as string first
                    if let Some(balance) = account.get("balance").and_then(|b| b.as_str()) {
                        if let Ok(bal) = balance.parse::<u64>() {
                            if bal > 0 {
                                return Some(bal);
                            }
                        }
                    }
                    // Try balance as number
                    if let Some(bal) = account.get("balance").and_then(|b| b.as_u64()) {
                        if bal > 0 {
                            return Some(bal);
                        }
                    }
                }
                println!("  Waiting for balance... (attempt {}/{})", i + 1, max_attempts);
            }
            Err(_) => {
                // Account may not exist yet
                println!("  Account not found yet... (attempt {}/{})", i + 1, max_attempts);
            }
        }

        if i < max_attempts - 1 {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
    None
}

/// Poll for key page credits (matching Dart SDK pollForKeyPageCredits)
pub async fn poll_for_credits(
    client: &AccumulateClient,
    key_page_url: &str,
    max_attempts: u32,
) -> Option<u64> {
    for i in 0..max_attempts {
        let params = json!({
            "scope": key_page_url,
            "query": {"queryType": "default"}
        });

        if let Ok(result) = client.v3_client.call_v3::<Value>("query", params).await {
            if let Some(account) = result.get("account") {
                if let Some(credits) = account.get("creditBalance").and_then(|c| c.as_u64()) {
                    if credits > 0 {
                        return Some(credits);
                    }
                }
            }
        }

        if i < max_attempts - 1 {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
    None
}

/// Wait for transaction confirmation
pub async fn wait_for_tx(
    client: &AccumulateClient,
    txid: &str,
    max_attempts: u32,
) -> bool {
    let tx_hash = txid.split('@').next().unwrap_or(txid).replace("acc://", "");

    for _ in 0..max_attempts {
        let params = json!({
            "scope": format!("acc://{}@unknown", tx_hash),
            "query": {"queryType": "default"}
        });

        if let Ok(result) = client.v3_client.call_v3::<Value>("query", params).await {
            if let Some(status) = result.get("status") {
                if status.get("delivered").and_then(|d| d.as_bool()).unwrap_or(false) {
                    return true;
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    false
}

// =============================================================================
// WALLET STRUCT
// =============================================================================

/// Simple wallet with lite identity and token account
#[derive(Debug, Clone)]
pub struct Wallet {
    /// Lite identity URL
    pub lite_identity: String,
    /// Lite token account URL
    pub lite_token_account: String,
    /// Signing key
    keypair: SigningKey,
}

impl Wallet {
    /// Get the signing key
    pub fn keypair(&self) -> &SigningKey {
        &self.keypair
    }

    /// Get the public key bytes
    pub fn public_key(&self) -> [u8; 32] {
        self.keypair.verifying_key().to_bytes()
    }

    /// Get the public key hash
    pub fn public_key_hash(&self) -> [u8; 32] {
        sha256_hash(&self.public_key())
    }
}

// =============================================================================
// ADI INFO STRUCT
// =============================================================================

/// ADI (Accumulate Digital Identity) information
#[derive(Debug, Clone)]
pub struct AdiInfo {
    /// ADI URL
    pub url: String,
    /// Key book URL
    pub key_book_url: String,
    /// Key page URL
    pub key_page_url: String,
    /// ADI signing key
    keypair: SigningKey,
}

impl AdiInfo {
    /// Get the signing key
    pub fn keypair(&self) -> &SigningKey {
        &self.keypair
    }

    /// Get the public key bytes
    pub fn public_key(&self) -> [u8; 32] {
        self.keypair.verifying_key().to_bytes()
    }
}

// =============================================================================
// KEY PAGE INFO
// =============================================================================

/// Key page information for QuickStart API
#[derive(Debug, Clone)]
pub struct KeyPageInfo {
    /// Credit balance
    pub credits: u64,
    /// Current version
    pub version: u64,
    /// Accept threshold
    pub threshold: u64,
    /// Number of keys
    pub key_count: usize,
}

// =============================================================================
// QUICKSTART API
// =============================================================================

/// Ultra-simple API for rapid development (matching Dart SDK QuickStart)
#[derive(Debug)]
pub struct QuickStart {
    /// The underlying client
    client: AccumulateClient,
}

impl QuickStart {
    /// Connect to local DevNet
    pub async fn devnet() -> Result<Self, JsonRpcError> {
        let v2_url = Url::parse(DEVNET_V2).map_err(|e| {
            JsonRpcError::General(anyhow::anyhow!("Invalid URL: {}", e))
        })?;
        let v3_url = Url::parse(DEVNET_V3).map_err(|e| {
            JsonRpcError::General(anyhow::anyhow!("Invalid URL: {}", e))
        })?;

        let client = AccumulateClient::new_with_options(v2_url, v3_url, AccOptions::default()).await?;
        Ok(Self { client })
    }

    /// Connect to Kermit testnet
    pub async fn kermit() -> Result<Self, JsonRpcError> {
        let v2_url = Url::parse(KERMIT_V2).map_err(|e| {
            JsonRpcError::General(anyhow::anyhow!("Invalid URL: {}", e))
        })?;
        let v3_url = Url::parse(KERMIT_V3).map_err(|e| {
            JsonRpcError::General(anyhow::anyhow!("Invalid URL: {}", e))
        })?;

        let client = AccumulateClient::new_with_options(v2_url, v3_url, AccOptions::default()).await?;
        Ok(Self { client })
    }

    /// Connect to custom endpoints
    pub async fn custom(v2_endpoint: &str, v3_endpoint: &str) -> Result<Self, JsonRpcError> {
        let v2_url = Url::parse(v2_endpoint).map_err(|e| {
            JsonRpcError::General(anyhow::anyhow!("Invalid V2 URL: {}", e))
        })?;
        let v3_url = Url::parse(v3_endpoint).map_err(|e| {
            JsonRpcError::General(anyhow::anyhow!("Invalid V3 URL: {}", e))
        })?;

        let client = AccumulateClient::new_with_options(v2_url, v3_url, AccOptions::default()).await?;
        Ok(Self { client })
    }

    /// Get the underlying client
    pub fn client(&self) -> &AccumulateClient {
        &self.client
    }

    /// Create a new wallet with lite identity and token account
    pub fn create_wallet(&self) -> Wallet {
        let keypair = AccumulateClient::generate_keypair();
        let public_key = keypair.verifying_key().to_bytes();

        // Derive lite identity URL
        let lite_identity = derive_lite_identity_url(&public_key);
        let lite_token_account = format!("{}/ACME", lite_identity);

        Wallet {
            lite_identity,
            lite_token_account,
            keypair,
        }
    }

    /// Fund wallet from faucet (multiple requests) using V3 API
    pub async fn fund_wallet(&self, wallet: &Wallet, times: u32) -> Result<(), JsonRpcError> {
        for i in 0..times {
            let params = json!({"account": &wallet.lite_token_account});
            match self.client.v3_client.call_v3::<Value>("faucet", params).await {
                Ok(response) => {
                    let txid = response.get("transactionHash")
                        .or_else(|| response.get("txid"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("submitted");
                    println!("  Faucet {}/{}: {}", i + 1, times, txid);
                }
                Err(e) => {
                    println!("  Faucet {}/{} failed: {}", i + 1, times, e);
                }
            }
            if i < times - 1 {
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }

        // Wait for faucet transactions to process
        println!("  Waiting for faucet to process...");
        tokio::time::sleep(Duration::from_secs(10)).await;

        // Poll for balance to confirm account is available
        let balance = poll_for_balance(&self.client, &wallet.lite_token_account, 30).await;
        if balance.is_none() || balance == Some(0) {
            println!("  Warning: Account balance not confirmed yet");
        }

        Ok(())
    }

    /// Get account balance (polls up to 30 times)
    pub async fn get_balance(&self, wallet: &Wallet) -> Option<u64> {
        poll_for_balance(&self.client, &wallet.lite_token_account, 30).await
    }

    /// Get oracle price from network status
    pub async fn get_oracle_price(&self) -> Result<u64, JsonRpcError> {
        let result: Value = self.client.v3_client.call_v3("network-status", json!({})).await?;

        result.get("oracle")
            .and_then(|o| o.get("price"))
            .and_then(|p| p.as_u64())
            .ok_or_else(|| JsonRpcError::General(anyhow::anyhow!("Oracle price not found")))
    }

    /// Calculate ACME amount for desired credits
    pub fn calculate_credits_amount(credits: u64, oracle: u64) -> u64 {
        // credits * 10^10 / oracle
        (credits as u128 * 10_000_000_000u128 / oracle as u128) as u64
    }

    /// Set up an ADI (handles all the complexity)
    pub async fn setup_adi(&self, wallet: &Wallet, adi_name: &str) -> Result<AdiInfo, JsonRpcError> {
        let adi_keypair = AccumulateClient::generate_keypair();
        let adi_public_key = adi_keypair.verifying_key().to_bytes();
        let adi_key_hash = sha256_hash(&adi_public_key);

        let identity_url = format!("acc://{}.acme", adi_name);
        let book_url = format!("{}/book", identity_url);
        let key_page_url = format!("{}/1", book_url);

        // First, add credits to lite identity
        let oracle = self.get_oracle_price().await?;
        let credits_amount = Self::calculate_credits_amount(1000, oracle);

        let mut signer = SmartSigner::new(&self.client, wallet.keypair.clone(), &wallet.lite_identity);

        // Add credits to lite identity
        let add_credits_body = TxBody::add_credits(
            &wallet.lite_identity,
            &credits_amount.to_string(),
            oracle,
        );

        let result = signer.sign_submit_and_wait(
            &wallet.lite_token_account,
            &add_credits_body,
            Some("Add credits to lite identity"),
            30,
        ).await;

        if !result.success {
            return Err(JsonRpcError::General(anyhow::anyhow!(
                "Failed to add credits: {:?}", result.error
            )));
        }

        // Create ADI
        let create_adi_body = TxBody::create_identity(
            &identity_url,
            &book_url,
            &hex::encode(adi_key_hash),
        );

        let result = signer.sign_submit_and_wait(
            &wallet.lite_token_account,
            &create_adi_body,
            Some("Create ADI"),
            30,
        ).await;

        if !result.success {
            return Err(JsonRpcError::General(anyhow::anyhow!(
                "Failed to create ADI: {:?}", result.error
            )));
        }

        Ok(AdiInfo {
            url: identity_url,
            key_book_url: book_url,
            key_page_url,
            keypair: adi_keypair,
        })
    }

    /// Buy credits for ADI key page (auto-fetches oracle)
    pub async fn buy_credits_for_adi(&self, wallet: &Wallet, adi: &AdiInfo, credits: u64) -> Result<TxResult, JsonRpcError> {
        let oracle = self.get_oracle_price().await?;
        let amount = Self::calculate_credits_amount(credits, oracle);

        let mut signer = SmartSigner::new(&self.client, wallet.keypair.clone(), &wallet.lite_identity);

        let body = TxBody::add_credits(&adi.key_page_url, &amount.to_string(), oracle);

        Ok(signer.sign_submit_and_wait(
            &wallet.lite_token_account,
            &body,
            Some("Buy credits for ADI"),
            30,
        ).await)
    }

    /// Get key page information
    pub async fn get_key_page_info(&self, key_page_url: &str) -> Option<KeyPageInfo> {
        let manager = KeyManager::new(&self.client, key_page_url);
        match manager.get_key_page_state().await {
            Ok(state) => Some(KeyPageInfo {
                credits: state.credit_balance,
                version: state.version,
                threshold: state.accept_threshold,
                key_count: state.keys.len(),
            }),
            Err(_) => None,
        }
    }

    /// Create a token account under an ADI
    pub async fn create_token_account(&self, adi: &AdiInfo, account_name: &str) -> Result<TxResult, JsonRpcError> {
        let account_url = format!("{}/{}", adi.url, account_name);
        let mut signer = SmartSigner::new(&self.client, adi.keypair.clone(), &adi.key_page_url);

        let body = TxBody::create_token_account(&account_url, "acc://ACME");

        Ok(signer.sign_submit_and_wait(
            &adi.url,
            &body,
            Some("Create token account"),
            30,
        ).await)
    }

    /// Create a data account under an ADI
    pub async fn create_data_account(&self, adi: &AdiInfo, account_name: &str) -> Result<TxResult, JsonRpcError> {
        let account_url = format!("{}/{}", adi.url, account_name);
        let mut signer = SmartSigner::new(&self.client, adi.keypair.clone(), &adi.key_page_url);

        let body = TxBody::create_data_account(&account_url);

        Ok(signer.sign_submit_and_wait(
            &adi.url,
            &body,
            Some("Create data account"),
            30,
        ).await)
    }

    /// Write data to a data account
    pub async fn write_data(&self, adi: &AdiInfo, account_name: &str, entries: &[&str]) -> Result<TxResult, JsonRpcError> {
        let account_url = format!("{}/{}", adi.url, account_name);
        let mut signer = SmartSigner::new(&self.client, adi.keypair.clone(), &adi.key_page_url);

        let body = TxBody::write_data(entries);

        Ok(signer.sign_submit_and_wait(
            &account_url,
            &body,
            Some("Write data"),
            30,
        ).await)
    }

    /// Add a key to the ADI's key page
    pub async fn add_key_to_adi(&self, adi: &AdiInfo, new_keypair: &SigningKey) -> Result<TxResult, JsonRpcError> {
        let mut signer = SmartSigner::new(&self.client, adi.keypair.clone(), &adi.key_page_url);
        Ok(signer.add_key(&new_keypair.verifying_key().to_bytes()).await)
    }

    /// Set multi-sig threshold for the ADI's key page
    pub async fn set_multi_sig_threshold(&self, adi: &AdiInfo, threshold: u64) -> Result<TxResult, JsonRpcError> {
        let mut signer = SmartSigner::new(&self.client, adi.keypair.clone(), &adi.key_page_url);
        Ok(signer.set_threshold(threshold).await)
    }

    /// Close the client (for cleanup)
    pub fn close(&self) {
        // HTTP client cleanup is automatic in Rust
    }
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Derive lite identity URL from public key
///
/// Format: acc://[40 hex key hash][8 hex checksum]
/// - Key hash: SHA256(publicKey)[0..20] as hex (40 chars)
/// - Checksum: SHA256(keyHashHex)[28..32] as hex (8 chars)
///
/// Note: Lite addresses do NOT have .acme suffix!
pub fn derive_lite_identity_url(public_key: &[u8; 32]) -> String {
    // Get first 20 bytes of SHA256(publicKey)
    let hash = sha256_hash(public_key);
    let key_hash_20 = &hash[0..20];

    // Convert to hex string
    let key_hash_hex = hex::encode(key_hash_20);

    // Compute checksum: SHA256(keyHashHex)[28..32]
    let checksum_full = sha256_hash(key_hash_hex.as_bytes());
    let checksum_hex = hex::encode(&checksum_full[28..32]);

    // Format: acc://[keyHash][checksum]
    format!("acc://{}{}", key_hash_hex, checksum_hex)
}

/// Derive lite token account URL from public key
///
/// Format: acc://[40 hex key hash][8 hex checksum]/ACME
pub fn derive_lite_token_account_url(public_key: &[u8; 32]) -> String {
    let lite_identity = derive_lite_identity_url(public_key);
    format!("{}/ACME", lite_identity)
}

/// SHA-256 hash helper
pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Extract transaction ID from submit response
///
/// The V3 API returns a List with two entries:
/// - [0] = transaction result with txID like acc://hash@account/path
/// - [1] = signature result with txID like acc://hash@account
///
/// We prefer the second entry (signature tx) which doesn't have path suffix.
fn extract_txid(response: &Value) -> Option<String> {
    // Try array format first - this is the V3 format
    if let Some(arr) = response.as_array() {
        // Prefer second entry (signature tx) if available
        if arr.len() > 1 {
            if let Some(status) = arr[1].get("status") {
                if let Some(txid) = status.get("txID").and_then(|t| t.as_str()) {
                    return Some(txid.to_string());
                }
            }
        }
        // Fall back to first entry
        if let Some(first) = arr.first() {
            if let Some(status) = first.get("status") {
                if let Some(txid) = status.get("txID").and_then(|t| t.as_str()) {
                    return Some(txid.to_string());
                }
            }
        }
    }

    // Try direct format
    response.get("txid")
        .or_else(|| response.get("transactionHash"))
        .and_then(|t| t.as_str())
        .map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_lite_identity_url() {
        let public_key = [1u8; 32];
        let url = derive_lite_identity_url(&public_key);
        assert!(url.starts_with("acc://"));
        // Lite URLs do NOT have .acme suffix - they have 40 hex chars + 8 hex checksum
        assert!(!url.ends_with(".acme"));
        // Format: acc://[40 hex][8 hex checksum]
        let path = url.strip_prefix("acc://").unwrap();
        assert_eq!(path.len(), 48); // 40 + 8 hex chars
    }

    #[test]
    fn test_tx_body_add_credits() {
        let body = TxBody::add_credits("acc://test.acme/credits", "1000000", 5000);
        assert_eq!(body["type"], "addCredits");
        assert_eq!(body["recipient"], "acc://test.acme/credits");
    }

    #[test]
    fn test_tx_body_send_tokens() {
        let body = TxBody::send_tokens_single("acc://bob.acme/tokens", "100");
        assert_eq!(body["type"], "sendTokens");
    }

    #[test]
    fn test_tx_body_create_identity() {
        let body = TxBody::create_identity(
            "acc://test.acme",
            "acc://test.acme/book",
            "0123456789abcdef",
        );
        assert_eq!(body["type"], "createIdentity");
        assert_eq!(body["url"], "acc://test.acme");
    }

    #[test]
    fn test_wallet_creation() {
        let keypair = AccumulateClient::generate_keypair();
        let public_key = keypair.verifying_key().to_bytes();
        let lite_identity = derive_lite_identity_url(&public_key);
        let lite_token_account = derive_lite_token_account_url(&public_key);

        assert!(lite_identity.starts_with("acc://"));
        assert!(lite_token_account.contains("/ACME"));
    }
}
