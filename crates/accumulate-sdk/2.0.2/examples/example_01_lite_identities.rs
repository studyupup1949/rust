//! Example 1: Lite Identities (Kermit Testnet)
//!
//! This example demonstrates:
//! - Creating lite identities and token accounts
//! - Using the SmartSigner API for auto-version tracking
//! - Funding accounts via faucet
//! - Adding credits and sending tokens
//!
//! Run with: cargo run --example example_01_lite_identities

use accumulate_client::{
    AccumulateClient, AccOptions, TxBody, SmartSigner,
    poll_for_balance, derive_lite_identity_url,
    KERMIT_V2, KERMIT_V3,
};
use serde_json::{json, Value};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SDK Example 1: Lite Identities ===\n");
    println!("Endpoint: {}\n", KERMIT_V3);

    // Connect to Kermit testnet
    let v2_url = Url::parse(KERMIT_V2)?;
    let v3_url = Url::parse(KERMIT_V3)?;
    let client = AccumulateClient::new_with_options(v2_url, v3_url, AccOptions::default()).await?;

    // =========================================================
    // Step 1: Generate key pairs for two lite identities
    // =========================================================
    println!("--- Step 1: Generate Key Pairs ---\n");

    let kp1 = AccumulateClient::generate_keypair();
    let kp2 = AccumulateClient::generate_keypair();

    // Derive lite identity and token account URLs
    let pk1 = kp1.verifying_key().to_bytes();
    let pk2 = kp2.verifying_key().to_bytes();

    let lid1 = derive_lite_identity_url(&pk1);
    let lta1 = format!("{}/ACME", lid1);
    let lid2 = derive_lite_identity_url(&pk2);
    let lta2 = format!("{}/ACME", lid2);

    println!("Lite Identity 1: {}", lid1);
    println!("Lite Token Account 1: {}", lta1);
    println!("Public Key 1: {}\n", hex::encode(&pk1[0..16]));

    println!("Lite Identity 2: {}", lid2);
    println!("Lite Token Account 2: {}", lta2);
    println!("Public Key 2: {}\n", hex::encode(&pk2[0..16]));

    // =========================================================
    // Step 2: Fund the first lite account via faucet
    // =========================================================
    println!("--- Step 2: Fund Account via Faucet ---\n");

    println!("Requesting funds from faucet (5 times)...");
    for i in 1..=5 {
        // Use V3 faucet API for better response handling
        let params = json!({
            "account": lta1
        });
        match client.v3_client.call_v3::<Value>("faucet", params).await {
            Ok(response) => {
                // V3 faucet returns different format - extract txid from various possible fields
                let txid = response.get("transactionHash")
                    .or_else(|| response.get("txid"))
                    .or_else(|| response.get("hash"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("submitted");
                println!("  Faucet {}/5: {}", i, txid);
            }
            Err(e) => {
                println!("  Faucet {}/5 failed: {}", i, e);
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    // Wait for faucet transactions to settle
    println!("\nWaiting 10 seconds for faucet transactions to settle...");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Poll for balance (with more attempts)
    println!("\nPolling for balance...");
    let balance = poll_for_balance(&client, &lta1, 60).await;
    if balance.is_none() || balance == Some(0) {
        println!("ERROR: Account not funded after 60 attempts. Stopping.");
        return Ok(());
    }
    println!("Balance confirmed: {:?}\n", balance);

    // =========================================================
    // Step 3: Add credits to lite identity using SmartSigner
    // =========================================================
    println!("--- Step 3: Add Credits (using SmartSigner) ---\n");

    // Create SmartSigner - auto-queries signer version!
    let mut signer1 = SmartSigner::new(&client, kp1.clone(), &lid1);

    // Get oracle price
    let oracle = get_oracle_price(&client).await?;
    println!("Oracle price: {}", oracle);

    // Calculate amount for 1000 credits
    let credits = 1000u64;
    let amount = calculate_credits_amount(credits, oracle);
    println!("Buying {} credits for {} ACME sub-units", credits, amount);

    // Use SmartSigner to sign and submit - no manual version tracking!
    let add_credits_body = TxBody::add_credits(&lid1, &amount.to_string(), oracle);

    let result = signer1.sign_submit_and_wait(
        &lta1,
        &add_credits_body,
        Some("Add credits to lite identity"),
        30,
    ).await;

    if result.success {
        println!("AddCredits SUCCESS - TxID: {:?}", result.txid);
    } else {
        println!("AddCredits FAILED: {:?}", result.error);
        println!("Continuing anyway to demonstrate API...");
    }

    // Verify credits were added
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    match query_credits(&client, &lid1).await {
        Ok(credits) => println!("Lite identity credit balance: {}\n", credits),
        Err(e) => println!("Could not query credit balance: {}\n", e),
    }

    // =========================================================
    // Step 4: Send tokens from lta1 to lta2
    // =========================================================
    println!("--- Step 4: Send Tokens ---\n");

    let send_amount = "100000000"; // 1 ACME (8 decimal places)
    println!("Sending 1 ACME from {} to {}", lta1, lta2);

    let send_body = TxBody::send_tokens_single(&lta2, send_amount);

    let result = signer1.sign_submit_and_wait(
        &lta1,
        &send_body,
        Some("Send 1 ACME"),
        30,
    ).await;

    if result.success {
        println!("SendTokens SUCCESS - TxID: {:?}", result.txid);
    } else {
        println!("SendTokens FAILED: {:?}", result.error);
    }

    // Check recipient balance
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    match query_balance(&client, &lta2).await {
        Ok(balance) => println!("Recipient balance: {}\n", balance),
        Err(e) => println!("Could not query recipient: {}\n", e),
    }

    // =========================================================
    // Summary
    // =========================================================
    println!("=== Summary ===\n");
    println!("Created two lite identities:");
    println!("  1. {}", lid1);
    println!("  2. {}", lid2);
    println!("\nUsed SmartSigner API which:");
    println!("  - Automatically queries signer version");
    println!("  - Provides sign_submit_and_wait()");
    println!("  - No manual version tracking needed!");

    Ok(())
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

/// Query credit balance
async fn query_credits(client: &AccumulateClient, url: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let params = json!({
        "scope": url,
        "query": {"queryType": "default"}
    });
    let result: Value = client.v3_client.call_v3("query", params).await?;
    result.get("account")
        .and_then(|a| a.get("creditBalance"))
        .and_then(|c| c.as_u64())
        .ok_or_else(|| "Credit balance not found".into())
}

/// Query token balance
async fn query_balance(client: &AccumulateClient, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let params = json!({
        "scope": url,
        "query": {"queryType": "default"}
    });
    let result: Value = client.v3_client.call_v3("query", params).await?;
    result.get("account")
        .and_then(|a| a.get("balance"))
        .and_then(|b| b.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Balance not found".into())
}
