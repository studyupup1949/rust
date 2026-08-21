//! Example 9: Key Management (Kermit Testnet)
//!
//! This example demonstrates:
//! - Adding keys to a key page
//! - Removing keys from a key page
//! - Querying key page state
//! - Using SmartSigner and KeyManager APIs
//!
//! Run with: cargo run --example example_09_key_management

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
    println!("=== SDK Example 9: Key Management ===\n");
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
    let adi_keypair = AccumulateClient::generate_keypair();
    let second_keypair = AccumulateClient::generate_keypair();
    let third_keypair = AccumulateClient::generate_keypair();

    let lite_public_key = lite_keypair.verifying_key().to_bytes();
    let lite_identity = derive_lite_identity_url(&lite_public_key);
    let lite_token_account = format!("{}/ACME", lite_identity);

    println!("Lite Identity: {}", lite_identity);
    println!("Lite Token Account: {}", lite_token_account);
    println!("Generated 4 key pairs (lite, ADI, second, third)\n");

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

    let credits = 2000u64; // Extra credits for key management operations
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
    // Step 4: Create an ADI
    // =========================================================
    println!("--- Step 4: Create ADI ---\n");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis();
    let adi_name = format!("sdk-keymgmt-{}", timestamp);
    let identity_url = format!("acc://{}.acme", adi_name);
    let book_url = format!("{}/book", identity_url);
    let key_page_url = format!("{}/1", book_url);

    let adi_public_key = adi_keypair.verifying_key().to_bytes();
    let adi_key_hash = sha256_hash(&adi_public_key);
    let adi_key_hash_hex = hex::encode(adi_key_hash);

    println!("ADI URL: {}", identity_url);
    println!("Key Page URL: {}\n", key_page_url);

    let create_adi_body = TxBody::create_identity(&identity_url, &book_url, &adi_key_hash_hex);

    let result = lite_signer.sign_submit_and_wait(
        &lite_token_account,
        &create_adi_body,
        Some("Create ADI for key management demo"),
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

    let key_page_credits = 1000u64;
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
    // Step 6: Query Initial Key Page State
    // =========================================================
    println!("--- Step 6: Query Initial Key Page State ---\n");

    let key_manager = KeyManager::new(&client, &key_page_url);
    match key_manager.get_key_page_state().await {
        Ok(state) => {
            println!("Key Page: {}", key_page_url);
            println!("  Version: {}", state.version);
            println!("  Credits: {}", state.credit_balance);
            println!("  Threshold: {}", state.accept_threshold);
            println!("  Keys: {}", state.keys.len());
            for key in &state.keys {
                let hash_preview = if key.key_hash.len() > 16 {
                    &key.key_hash[0..16]
                } else {
                    &key.key_hash
                };
                println!("    - {}...", hash_preview);
            }
        }
        Err(e) => {
            println!("Error querying key page: {}", e);
        }
    }
    println!();

    // =========================================================
    // Step 7: Add Second Key
    // =========================================================
    println!("--- Step 7: Add Second Key ---\n");

    let mut adi_signer = SmartSigner::new(&client, adi_keypair.clone(), &key_page_url);

    let second_public_key = second_keypair.verifying_key().to_bytes();
    let second_key_hash = sha256_hash(&second_public_key);
    let second_key_hash_hex = hex::encode(second_key_hash);

    println!("Adding second key with hash: {}...", &second_key_hash_hex[0..16]);

    let result = adi_signer.add_key(&second_public_key).await;

    if result.success {
        println!("AddKey SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("AddKey FAILED: {:?}\n", result.error);
    }

    // Wait for key page update
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // =========================================================
    // Step 8: Add Third Key
    // =========================================================
    println!("--- Step 8: Add Third Key ---\n");

    let third_public_key = third_keypair.verifying_key().to_bytes();
    let third_key_hash = sha256_hash(&third_public_key);
    let third_key_hash_hex = hex::encode(third_key_hash);

    println!("Adding third key with hash: {}...", &third_key_hash_hex[0..16]);

    let result = adi_signer.add_key(&third_public_key).await;

    if result.success {
        println!("AddKey SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("AddKey FAILED: {:?}\n", result.error);
    }

    // Wait for key page update
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // =========================================================
    // Step 9: Query Updated Key Page State
    // =========================================================
    println!("--- Step 9: Query Updated Key Page State ---\n");

    match key_manager.get_key_page_state().await {
        Ok(state) => {
            println!("Key Page: {}", key_page_url);
            println!("  Version: {}", state.version);
            println!("  Credits: {}", state.credit_balance);
            println!("  Threshold: {}", state.accept_threshold);
            println!("  Keys: {} (should be 3)", state.keys.len());
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
    // Step 10: Remove Third Key
    // =========================================================
    println!("--- Step 10: Remove Third Key ---\n");

    println!("Removing third key with hash: {}...", &third_key_hash_hex[0..16]);

    let remove_key_body = TxBody::update_key_page_remove_key(&third_key_hash);

    let result = adi_signer.sign_submit_and_wait(
        &key_page_url,
        &remove_key_body,
        Some("Remove third key"),
        30,
    ).await;

    if result.success {
        println!("RemoveKey SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("RemoveKey FAILED: {:?}\n", result.error);
    }

    // Wait for key page update
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // =========================================================
    // Step 11: Query Final Key Page State
    // =========================================================
    println!("--- Step 11: Query Final Key Page State ---\n");

    match key_manager.get_key_page_state().await {
        Ok(state) => {
            println!("Key Page: {}", key_page_url);
            println!("  Version: {}", state.version);
            println!("  Credits: {}", state.credit_balance);
            println!("  Threshold: {}", state.accept_threshold);
            println!("  Keys: {} (should be 2)", state.keys.len());
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
    println!("\nKey Management Operations:");
    println!("  1. Started with 1 key (ADI key)");
    println!("  2. Added second key");
    println!("  3. Added third key");
    println!("  4. Removed third key");
    println!("  5. Final state: 2 keys");
    println!("\nUsed SmartSigner and KeyManager APIs!");

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
