//! Example 10: Threshold Updates (Kermit Testnet)
//!
//! This example demonstrates:
//! - Setting up a key page with multiple keys
//! - Updating the accept threshold for multi-signature requirements
//! - Understanding threshold vs key count relationships
//! - Using SmartSigner API for threshold management
//!
//! Run with: cargo run --example example_10_threshold_updates

use accumulate_client::{
    AccumulateClient, AccOptions, TxBody, SmartSigner, KeyManager,
    poll_for_balance, poll_for_credits, derive_lite_identity_url,
    KERMIT_V2, KERMIT_V3,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SDK Example 10: Threshold Updates (Multi-Sig Setup) ===\n");
    println!("Endpoint: {}\n", KERMIT_V3);

    // Connect to Kermit testnet
    let v2_url = Url::parse(KERMIT_V2)?;
    let v3_url = Url::parse(KERMIT_V3)?;
    let client = AccumulateClient::new_with_options(v2_url, v3_url, AccOptions::default()).await?;

    // =========================================================
    // Step 1: Generate key pairs
    // =========================================================
    println!("--- Step 1: Generate Key Pairs ---\n");

    let lite_keypair = AccumulateClient::generate_keypair();
    let key1 = AccumulateClient::generate_keypair();
    let key2 = AccumulateClient::generate_keypair();
    let key3 = AccumulateClient::generate_keypair();

    let lite_public_key = lite_keypair.verifying_key().to_bytes();
    let lite_identity = derive_lite_identity_url(&lite_public_key);
    let lite_token_account = format!("{}/ACME", lite_identity);

    println!("Lite Identity: {}", lite_identity);
    println!("Lite Token Account: {}", lite_token_account);
    println!("Generated 4 key pairs (lite, key1, key2, key3)\n");

    // =========================================================
    // Step 2: Fund the lite account via faucet
    // =========================================================
    println!("--- Step 2: Fund Account via Faucet ---\n");

    println!("Requesting funds from faucet (5 times)...");
    for i in 1..=5 {
        let params = json!({"account": &lite_token_account});
        match client.v3_client.call_v3::<Value>("faucet", params).await {
            Ok(response) => {
                let txid = response.get("transactionHash")
                    .or_else(|| response.get("txid"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("submitted");
                println!("  Faucet {}/5: {}", i, txid);
            }
            Err(e) => println!("  Faucet {}/5 failed: {}", i, e),
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    // Wait for faucet transactions to settle
    println!("\nWaiting 10 seconds for faucet transactions to settle...");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Poll for balance
    println!("\nPolling for balance...");
    let balance = poll_for_balance(&client, &lite_token_account, 30).await;
    if balance.is_none() || balance == Some(0) {
        println!("ERROR: Account not funded. Stopping.");
        return Ok(());
    }
    println!("Balance confirmed: {:?}\n", balance);

    // =========================================================
    // Step 3: Add credits to lite identity
    // =========================================================
    println!("--- Step 3: Add Credits to Lite Identity ---\n");

    let mut lite_signer = SmartSigner::new(&client, lite_keypair.clone(), &lite_identity);

    let oracle = get_oracle_price(&client).await?;
    println!("Oracle price: {}", oracle);

    let credits = 3000u64; // Extra credits for multiple operations
    let amount = calculate_credits_amount(credits, oracle);

    let add_credits_body = TxBody::add_credits(&lite_identity, &amount.to_string(), oracle);

    let result = lite_signer.sign_submit_and_wait(
        &lite_token_account,
        &add_credits_body,
        Some("Add credits to lite identity"),
        30,
    ).await;

    if result.success {
        println!("AddCredits SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("AddCredits FAILED: {:?}", result.error);
        return Ok(());
    }

    // =========================================================
    // Step 4: Create an ADI with first key
    // =========================================================
    println!("--- Step 4: Create ADI ---\n");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis();
    let adi_name = format!("sdk-threshold-{}", timestamp);
    let identity_url = format!("acc://{}.acme", adi_name);
    let book_url = format!("{}/book", identity_url);
    let key_page_url = format!("{}/1", book_url);

    let key1_public = key1.verifying_key().to_bytes();
    let key1_hash = sha256_hash(&key1_public);
    let key1_hash_hex = hex::encode(key1_hash);

    println!("ADI URL: {}", identity_url);
    println!("Key Page URL: {}", key_page_url);
    println!("Initial key hash: {}...\n", &key1_hash_hex[0..16]);

    let create_adi_body = TxBody::create_identity(&identity_url, &book_url, &key1_hash_hex);

    let result = lite_signer.sign_submit_and_wait(
        &lite_token_account,
        &create_adi_body,
        Some("Create ADI for threshold demo"),
        30,
    ).await;

    if result.success {
        println!("CreateIdentity SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("CreateIdentity FAILED: {:?}", result.error);
        return Ok(());
    }

    // =========================================================
    // Step 5: Add credits to ADI key page
    // =========================================================
    println!("--- Step 5: Add Credits to ADI Key Page ---\n");

    let key_page_credits = 2000u64;
    let key_page_amount = calculate_credits_amount(key_page_credits, oracle);

    let add_key_page_credits_body = TxBody::add_credits(&key_page_url, &key_page_amount.to_string(), oracle);

    let result = lite_signer.sign_submit_and_wait(
        &lite_token_account,
        &add_key_page_credits_body,
        Some("Add credits to ADI key page"),
        30,
    ).await;

    if result.success {
        println!("AddCredits to key page SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("AddCredits to key page FAILED: {:?}", result.error);
        return Ok(());
    }

    // Poll for key page credits
    let credits_confirmed = poll_for_credits(&client, &key_page_url, 30).await;
    if credits_confirmed.is_none() || credits_confirmed == Some(0) {
        println!("ERROR: Key page has no credits. Cannot proceed.");
        return Ok(());
    }

    // =========================================================
    // Step 6: Add key2 and key3 to the key page
    // =========================================================
    println!("--- Step 6: Add Additional Keys ---\n");

    let mut adi_signer = SmartSigner::new(&client, key1.clone(), &key_page_url);

    // Add key2
    let key2_public = key2.verifying_key().to_bytes();
    let key2_hash = sha256_hash(&key2_public);
    let key2_hash_hex = hex::encode(key2_hash);
    println!("Adding key2 with hash: {}...", &key2_hash_hex[0..16]);

    let result = adi_signer.add_key(&key2_public).await;
    if result.success {
        println!("  AddKey (key2) SUCCESS");
    } else {
        println!("  AddKey (key2) FAILED: {:?}", result.error);
    }

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Add key3
    let key3_public = key3.verifying_key().to_bytes();
    let key3_hash = sha256_hash(&key3_public);
    let key3_hash_hex = hex::encode(key3_hash);
    println!("Adding key3 with hash: {}...", &key3_hash_hex[0..16]);

    let result = adi_signer.add_key(&key3_public).await;
    if result.success {
        println!("  AddKey (key3) SUCCESS\n");
    } else {
        println!("  AddKey (key3) FAILED: {:?}\n", result.error);
    }

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // =========================================================
    // Step 7: Query Initial Key Page State (Threshold = 1)
    // =========================================================
    println!("--- Step 7: Query Initial Key Page State ---\n");

    let key_manager = KeyManager::new(&client, &key_page_url);
    match key_manager.get_key_page_state().await {
        Ok(state) => {
            println!("Key Page: {}", key_page_url);
            println!("  Version: {}", state.version);
            println!("  Keys: {}", state.keys.len());
            println!("  Threshold: {} (single-sig mode)", state.accept_threshold);
            println!("  Meaning: Only 1 of {} keys required to sign", state.keys.len());
        }
        Err(e) => {
            println!("Error querying key page: {}", e);
        }
    }
    println!();

    // =========================================================
    // Step 8: Update Threshold to 2-of-3 (Multi-Sig)
    // =========================================================
    println!("--- Step 8: Update Threshold to 2-of-3 ---\n");

    println!("Setting threshold to 2 (requiring 2 of 3 signatures)...");

    let result = adi_signer.set_threshold(2).await;

    if result.success {
        println!("SetThreshold SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("SetThreshold FAILED: {:?}\n", result.error);
    }

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // =========================================================
    // Step 9: Query Updated Key Page State
    // =========================================================
    println!("--- Step 9: Query Updated Key Page State ---\n");

    match key_manager.get_key_page_state().await {
        Ok(state) => {
            println!("Key Page: {}", key_page_url);
            println!("  Version: {}", state.version);
            println!("  Keys: {}", state.keys.len());
            println!("  Threshold: {} (multi-sig mode!)", state.accept_threshold);
            println!("  Meaning: {} of {} keys required to sign", state.accept_threshold, state.keys.len());
        }
        Err(e) => {
            println!("Error querying key page: {}", e);
        }
    }
    println!();

    // =========================================================
    // Step 10: Update Threshold to 3-of-3 (Full Multi-Sig)
    // =========================================================
    println!("--- Step 10: Update Threshold to 3-of-3 ---\n");

    println!("Setting threshold to 3 (requiring ALL signatures)...");

    let result = adi_signer.set_threshold(3).await;

    if result.success {
        println!("SetThreshold SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("SetThreshold FAILED: {:?}\n", result.error);
    }

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // =========================================================
    // Step 11: Query Final Key Page State
    // =========================================================
    println!("--- Step 11: Query Final Key Page State ---\n");

    match key_manager.get_key_page_state().await {
        Ok(state) => {
            println!("Key Page: {}", key_page_url);
            println!("  Version: {}", state.version);
            println!("  Keys: {}", state.keys.len());
            println!("  Threshold: {} (unanimous mode!)", state.accept_threshold);
            println!("  Meaning: ALL {} keys required to sign", state.keys.len());
            println!("\nKey Details:");
            for (i, key) in state.keys.iter().enumerate() {
                let hash_preview = if key.key_hash.len() > 16 {
                    &key.key_hash[0..16]
                } else {
                    &key.key_hash
                };
                println!("    Key {}: {}...", i + 1, hash_preview);
            }
        }
        Err(e) => {
            println!("Error querying key page: {}", e);
        }
    }
    println!();

    // =========================================================
    // Summary
    // =========================================================
    println!("=== Summary ===\n");
    println!("Created ADI: {}", identity_url);
    println!("Key Page: {}", key_page_url);
    println!("\nThreshold Configuration History:");
    println!("  1. Initial: 1-of-1 (single key, single sig)");
    println!("  2. After adding keys: 1-of-3 (any single key)");
    println!("  3. Updated to: 2-of-3 (majority required)");
    println!("  4. Final: 3-of-3 (unanimous agreement)");
    println!("\nMulti-Sig Security Levels:");
    println!("  - 1-of-N: Lowest security (any key can sign)");
    println!("  - M-of-N (M<N): Balanced (majority required)");
    println!("  - N-of-N: Highest security (all keys required)");
    println!("\nUsed SmartSigner API for all operations!");

    Ok(())
}

/// SHA-256 hash helper
fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Get oracle price from network
async fn get_oracle_price(client: &AccumulateClient) -> Result<u64, Box<dyn std::error::Error>> {
    let result: serde_json::Value = client.v3_client.call_v3("network-status", serde_json::json!({})).await?;
    result.get("oracle")
        .and_then(|o| o.get("price"))
        .and_then(|p| p.as_u64())
        .ok_or_else(|| "Oracle price not found".into())
}

/// Calculate ACME amount for desired credits
fn calculate_credits_amount(credits: u64, oracle: u64) -> u64 {
    (credits as u128 * 10_000_000_000u128 / oracle as u128) as u64
}
