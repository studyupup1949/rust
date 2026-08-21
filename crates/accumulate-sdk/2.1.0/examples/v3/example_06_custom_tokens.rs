//! Example 6: Custom Tokens (Kermit Testnet)
//!
//! This example demonstrates:
//! - Creating custom tokens with custom precision
//! - Creating token accounts for custom tokens
//! - Issuing tokens to accounts
//! - Transferring custom tokens between accounts
//!
//! Run with: cargo run --example example_06_custom_tokens

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
    println!("=== SDK Example 6: Custom Tokens ===\n");
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

    let lite_public_key = lite_keypair.verifying_key().to_bytes();
    let lite_identity = derive_lite_identity_url(&lite_public_key);
    let lite_token_account = format!("{}/ACME", lite_identity);

    println!("Lite Identity: {}", lite_identity);
    println!("Lite Token Account: {}\n", lite_token_account);

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

    // Need 1000 credits for ADI creation and operations
    let credits = 1000u64;
    let amount = calculate_credits_amount(credits, oracle);
    println!("Buying {} credits for {} ACME sub-units", credits, amount);

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
    let adi_name = format!("sdk-token-{}", timestamp);
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

    // Wait for key page credits to settle
    println!("Waiting for key page credits to settle...");
    let confirmed_credits = poll_for_credits(&client, &key_page_url, 30).await;
    if confirmed_credits.is_none() || confirmed_credits == Some(0) {
        println!("ERROR: Key page has no credits. Cannot proceed.");
        return Ok(());
    }
    println!("Key page credits confirmed: {:?}\n", confirmed_credits);

    // =========================================================
    // Step 6: Create Custom Token
    // =========================================================
    println!("--- Step 6: Create Custom Token ---\n");

    let mut adi_signer = SmartSigner::new(&client, adi_keypair.clone(), &key_page_url);

    // Create a custom token with 2 decimal places (like cents)
    let custom_token_url = format!("{}/my-token", identity_url);
    let symbol = "RUST";
    let precision = 2u64;

    println!("Creating custom token:");
    println!("  URL: {}", custom_token_url);
    println!("  Symbol: {}", symbol);
    println!("  Precision: {} (like cents)\n", precision);

    let create_token_body = TxBody::create_token(&custom_token_url, symbol, precision, None);

    let result = adi_signer.sign_submit_and_wait(
        &identity_url,
        &create_token_body,
        Some(&format!("Create custom token: {}", symbol)),
        30,
    ).await;

    if result.success {
        println!("CreateToken SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("CreateToken FAILED: {:?}", result.error);
    }

    // Wait for token creation
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // =========================================================
    // Step 7: Create Token Accounts
    // =========================================================
    println!("--- Step 7: Create Custom Token Accounts ---\n");

    let account1_url = format!("{}/wallet1", identity_url);
    let account2_url = format!("{}/wallet2", identity_url);

    // Create first token account
    println!("Creating wallet1: {}", account1_url);
    let create_account1_body = TxBody::create_token_account(&account1_url, &custom_token_url);
    let result = adi_signer.sign_submit_and_wait(
        &identity_url,
        &create_account1_body,
        Some("Create custom token wallet 1"),
        30,
    ).await;

    if result.success {
        println!("CreateTokenAccount 1 SUCCESS - TxID: {:?}", result.txid);
    } else {
        println!("CreateTokenAccount 1 FAILED: {:?}", result.error);
    }

    // Create second token account
    println!("Creating wallet2: {}", account2_url);
    let create_account2_body = TxBody::create_token_account(&account2_url, &custom_token_url);
    let result = adi_signer.sign_submit_and_wait(
        &identity_url,
        &create_account2_body,
        Some("Create custom token wallet 2"),
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
    // Step 8: Issue Tokens
    // =========================================================
    println!("--- Step 8: Issue Custom Tokens ---\n");

    // Issue 10000 tokens (100.00 with 2 decimal precision)
    let issue_amount = "10000"; // 100.00 RUST
    println!("Issuing {} tokens (100.00 {}) to {}", issue_amount, symbol, account1_url);

    let issue_body = TxBody::issue_tokens_single(&account1_url, issue_amount);

    let result = adi_signer.sign_submit_and_wait(
        &custom_token_url,
        &issue_body,
        Some("Issue custom tokens"),
        30,
    ).await;

    if result.success {
        println!("IssueTokens SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("IssueTokens FAILED: {:?}", result.error);
    }

    // Wait for issuance
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // =========================================================
    // Step 9: Transfer Tokens
    // =========================================================
    println!("--- Step 9: Send Custom Tokens ---\n");

    // Send 2500 tokens (25.00 with 2 decimal precision)
    let send_amount = "2500"; // 25.00 RUST
    println!("Sending {} tokens (25.00 {}) from wallet1 to wallet2", send_amount, symbol);

    let send_body = TxBody::send_tokens_single(&account2_url, send_amount);

    let result = adi_signer.sign_submit_and_wait(
        &account1_url,
        &send_body,
        Some("Send custom tokens"),
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
    println!("Created custom token: {}", custom_token_url);
    println!("  Symbol: {}", symbol);
    println!("  Precision: {}", precision);
    println!("Wallet 1: {}", account1_url);
    println!("Wallet 2: {}", account2_url);
    println!("\nOperations:");
    println!("  - Issued 100.00 {} to Wallet 1", symbol);
    println!("  - Transferred 25.00 {} to Wallet 2", symbol);
    println!("  - Wallet 1 balance: 75.00 {}", symbol);
    println!("  - Wallet 2 balance: 25.00 {}", symbol);
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
