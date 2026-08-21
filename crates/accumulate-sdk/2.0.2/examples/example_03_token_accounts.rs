//! Example 3: ADI Token Accounts (Kermit Testnet)
//!
//! This example demonstrates:
//! - Creating ADI ACME token accounts
//! - Sending tokens between lite and ADI accounts
//! - Using SmartSigner API with auto-version tracking
//!
//! Run with: cargo run --example example_03_token_accounts

use accumulate_client::{
    AccumulateClient, AccOptions, TxBody, SmartSigner,
    poll_for_balance, poll_for_credits, derive_lite_identity_url,
    KERMIT_V2, KERMIT_V3,
};
use sha2::{Digest, Sha256};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SDK Example 3: ADI Token Accounts ===\n");
    println!("Endpoint: {}\n", KERMIT_V3);

    // Connect to Kermit testnet
    let v2_url = Url::parse(KERMIT_V2)?;
    let v3_url = Url::parse(KERMIT_V3)?;
    let client = AccumulateClient::new_with_options(v2_url, v3_url, AccOptions::default()).await?;

    // =========================================================
    // Step 1: Generate key pairs
    // =========================================================
    println!("--- Step 1: Generate Key Pairs ---\n");

    let lite_keypair1 = AccumulateClient::generate_keypair();
    let lite_keypair2 = AccumulateClient::generate_keypair();
    let adi_keypair = AccumulateClient::generate_keypair();

    let lite_public_key1 = lite_keypair1.verifying_key().to_bytes();
    let lite_public_key2 = lite_keypair2.verifying_key().to_bytes();

    let lite_identity1 = derive_lite_identity_url(&lite_public_key1);
    let lite_token_account1 = format!("{}/ACME", lite_identity1);
    let lite_identity2 = derive_lite_identity_url(&lite_public_key2);
    let lite_token_account2 = format!("{}/ACME", lite_identity2);

    println!("Lite Account 1: {}", lite_token_account1);
    println!("Lite Account 2: {}\n", lite_token_account2);

    // =========================================================
    // Step 2: Fund the first lite account via faucet
    // =========================================================
    println!("--- Step 2: Fund Account via Faucet ---\n");

    println!("Requesting funds from faucet (5 times)...");
    for i in 1..=5 {
        let params = json!({"account": &lite_token_account1});
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
    let balance = poll_for_balance(&client, &lite_token_account1, 30).await;
    if balance.is_none() || balance == Some(0) {
        println!("ERROR: Account not funded. Stopping.");
        return Ok(());
    }
    println!("Balance confirmed: {:?}\n", balance);

    // =========================================================
    // Step 3: Add credits to lite identity
    // =========================================================
    println!("--- Step 3: Add Credits to Lite Identity ---\n");

    let mut lite_signer1 = SmartSigner::new(&client, lite_keypair1.clone(), &lite_identity1);

    let oracle = get_oracle_price(&client).await?;
    println!("Oracle price: {}", oracle);

    // Need 1000 credits for ADI creation and operations
    let credits = 1000u64;
    let amount = calculate_credits_amount(credits, oracle);
    println!("Buying {} credits for {} ACME sub-units", credits, amount);

    let add_credits_body = TxBody::add_credits(&lite_identity1, &amount.to_string(), oracle);

    let result = lite_signer1.sign_submit_and_wait(
        &lite_token_account1,
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
    let adi_name = format!("sdk-adi-{}", timestamp);
    let identity_url = format!("acc://{}.acme", adi_name);
    let book_url = format!("{}/book", identity_url);
    let key_page_url = format!("{}/1", book_url);

    let adi_public_key = adi_keypair.verifying_key().to_bytes();
    let adi_key_hash = sha256_hash(&adi_public_key);
    let adi_key_hash_hex = hex::encode(adi_key_hash);

    println!("ADI URL: {}", identity_url);
    println!("Key Page URL: {}\n", key_page_url);

    let create_adi_body = TxBody::create_identity(&identity_url, &book_url, &adi_key_hash_hex);

    let result = lite_signer1.sign_submit_and_wait(
        &lite_token_account1,
        &create_adi_body,
        Some("Create ADI via Rust SDK"),
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

    let key_page_credits = 500u64;
    let key_page_amount = calculate_credits_amount(key_page_credits, oracle);

    let add_key_page_credits_body = TxBody::add_credits(&key_page_url, &key_page_amount.to_string(), oracle);

    let result = lite_signer1.sign_submit_and_wait(
        &lite_token_account1,
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

    // Wait for key page credits to settle
    println!("Waiting for key page credits to settle...");
    let confirmed_credits = poll_for_credits(&client, &key_page_url, 30).await;
    if confirmed_credits.is_none() || confirmed_credits == Some(0) {
        println!("ERROR: Key page has no credits. Cannot proceed.");
        return Ok(());
    }
    println!("Key page credits confirmed: {:?}\n", confirmed_credits);

    // =========================================================
    // Step 6: Create ADI Token Accounts
    // =========================================================
    println!("--- Step 6: Create ADI Token Accounts ---\n");

    let mut adi_signer = SmartSigner::new(&client, adi_keypair.clone(), &key_page_url);

    let token_account_url1 = format!("{}/acme-account-1", identity_url);
    let token_account_url2 = format!("{}/acme-account-2", identity_url);

    // Create first token account
    let create_token1_body = TxBody::create_token_account(&token_account_url1, "acc://ACME");
    let result = adi_signer.sign_submit_and_wait(
        &identity_url,
        &create_token1_body,
        Some("Create first ADI token account"),
        30,
    ).await;

    if result.success {
        println!("CreateTokenAccount 1 SUCCESS - TxID: {:?}", result.txid);
    } else {
        println!("CreateTokenAccount 1 FAILED: {:?}", result.error);
    }

    // Create second token account
    let create_token2_body = TxBody::create_token_account(&token_account_url2, "acc://ACME");
    let result = adi_signer.sign_submit_and_wait(
        &identity_url,
        &create_token2_body,
        Some("Create second ADI token account"),
        30,
    ).await;

    if result.success {
        println!("CreateTokenAccount 2 SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("CreateTokenAccount 2 FAILED: {:?}", result.error);
    }

    // Wait for accounts to be created
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // =========================================================
    // Step 7: Send tokens from lite to ADI account
    // =========================================================
    println!("--- Step 7: Send Tokens from Lite to ADI ---\n");

    let send_amount1 = "500000000"; // 5 ACME (8 decimal places)
    println!("Sending 5 ACME from lite to {}", token_account_url1);

    let send_body1 = TxBody::send_tokens_single(&token_account_url1, send_amount1);

    let result = lite_signer1.sign_submit_and_wait(
        &lite_token_account1,
        &send_body1,
        Some("Send 5 ACME to ADI token account"),
        30,
    ).await;

    if result.success {
        println!("SendTokens SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("SendTokens FAILED: {:?}", result.error);
    }

    // Wait for tokens to arrive
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // =========================================================
    // Step 8: Send tokens from ADI to lite account
    // =========================================================
    println!("--- Step 8: Send Tokens from ADI to Lite ---\n");

    let send_amount2 = "200000000"; // 2 ACME
    println!("Sending 2 ACME from {} to {}", token_account_url1, lite_token_account2);

    let send_body2 = TxBody::send_tokens_single(&lite_token_account2, send_amount2);

    let result = adi_signer.sign_submit_and_wait(
        &token_account_url1,
        &send_body2,
        Some("Send 2 ACME to lite account"),
        30,
    ).await;

    if result.success {
        println!("SendTokens SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("SendTokens FAILED: {:?}", result.error);
    }

    // =========================================================
    // Step 9: Send tokens between ADI accounts
    // =========================================================
    println!("--- Step 9: Send Tokens Between ADI Accounts ---\n");

    let send_amount3 = "100000000"; // 1 ACME
    println!("Sending 1 ACME from {} to {}", token_account_url1, token_account_url2);

    let send_body3 = TxBody::send_tokens_single(&token_account_url2, send_amount3);

    let result = adi_signer.sign_submit_and_wait(
        &token_account_url1,
        &send_body3,
        Some("Send 1 ACME between ADI accounts"),
        30,
    ).await;

    if result.success {
        println!("SendTokens SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("SendTokens FAILED: {:?}", result.error);
    }

    // =========================================================
    // Summary
    // =========================================================
    println!("=== Summary ===\n");
    println!("Created ADI: {}", identity_url);
    println!("Token Account 1: {}", token_account_url1);
    println!("Token Account 2: {}", token_account_url2);
    println!("\nToken transfers:");
    println!("  - 5 ACME: lite -> ADI account 1");
    println!("  - 2 ACME: ADI account 1 -> lite account 2");
    println!("  - 1 ACME: ADI account 1 -> ADI account 2");
    println!("\nUsed SmartSigner API for all transactions!");

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
    let result: Value = client.v3_client.call_v3("network-status", json!({})).await?;
    result.get("oracle")
        .and_then(|o| o.get("price"))
        .and_then(|p| p.as_u64())
        .ok_or_else(|| "Oracle price not found".into())
}

/// Calculate ACME amount for desired credits
fn calculate_credits_amount(credits: u64, oracle: u64) -> u64 {
    (credits as u128 * 10_000_000_000u128 / oracle as u128) as u64
}
