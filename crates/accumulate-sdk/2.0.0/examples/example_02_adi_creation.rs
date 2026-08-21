//! Example 2: ADI Creation (Kermit Testnet)
//!
//! This example demonstrates:
//! - Creating lite identities and token accounts
//! - Creating ADIs (Accumulate Digital Identities)
//! - Adding credits to lite identities and key pages
//! - Using SmartSigner API for auto-version tracking
//!
//! Run with: cargo run --example example_02_adi_creation

use accumulate_client::{
    AccumulateClient, AccOptions, TxBody, SmartSigner,
    poll_for_balance, derive_lite_identity_url,
    KERMIT_V2, KERMIT_V3,
};
use sha2::{Digest, Sha256};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SDK Example 2: ADI Creation ===\n");
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
    println!("Lite Token Account: {}", lite_token_account);
    println!("Public Key Hash: {}\n", hex::encode(&sha256_hash(&lite_public_key)[0..16]));

    // =========================================================
    // Step 2: Fund the lite account via faucet
    // =========================================================
    println!("--- Step 2: Fund Account via Faucet ---\n");

    println!("Requesting funds from faucet (3 times)...");
    for i in 1..=3 {
        let params = json!({"account": &lite_token_account});
        match client.v3_client.call_v3::<Value>("faucet", params).await {
            Ok(response) => {
                let txid = response.get("transactionHash")
                    .or_else(|| response.get("txid"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("submitted");
                println!("  Faucet {}/3: {}", i, txid);
            }
            Err(e) => println!("  Faucet {}/3 failed: {}", i, e),
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

    // Need 500 credits for ADI creation
    let credits = 500u64;
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
        println!("AddCredits SUCCESS - TxID: {:?}", result.txid);
    } else {
        println!("AddCredits FAILED: {:?}", result.error);
        println!("Continuing anyway to demonstrate API...");
    }

    // Verify credits were added
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    match query_credits(&client, &lite_identity).await {
        Ok(credits) => println!("Lite identity credit balance: {}\n", credits),
        Err(e) => println!("Could not query credit balance: {}\n", e),
    }

    // =========================================================
    // Step 4: Create an ADI
    // =========================================================
    println!("--- Step 4: Create ADI ---\n");

    // Generate unique ADI name with timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis();
    let adi_name = format!("sdk-adi-{}", timestamp);
    let identity_url = format!("acc://{}.acme", adi_name);
    let book_url = format!("{}/book", identity_url);
    let key_page_url = format!("{}/1", book_url);

    // Get key hash for ADI key
    let adi_public_key = adi_keypair.verifying_key().to_bytes();
    let adi_key_hash = sha256_hash(&adi_public_key);
    let adi_key_hash_hex = hex::encode(adi_key_hash);

    println!("ADI URL: {}", identity_url);
    println!("Key Book URL: {}", book_url);
    println!("ADI Key Hash: {}\n", &adi_key_hash_hex[0..32]);

    let create_adi_body = TxBody::create_identity(&identity_url, &book_url, &adi_key_hash_hex);

    let result = lite_signer.sign_submit_and_wait(
        &lite_token_account,
        &create_adi_body,
        Some("Create ADI via Rust SDK"),
        30,
    ).await;

    if result.success {
        println!("CreateIdentity SUCCESS - TxID: {:?}", result.txid);
    } else {
        println!("CreateIdentity FAILED: {:?}", result.error);
        return Ok(());
    }

    // Verify ADI was created
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    match query_account(&client, &identity_url).await {
        Ok(account) => {
            println!("ADI created: {}", account.get("url").and_then(|u| u.as_str()).unwrap_or("unknown"));
            println!("ADI type: {}\n", account.get("type").and_then(|t| t.as_str()).unwrap_or("unknown"));
        }
        Err(e) => println!("Could not verify ADI: {}\n", e),
    }

    // =========================================================
    // Step 5: Add credits to ADI key page
    // =========================================================
    println!("--- Step 5: Add Credits to ADI Key Page ---\n");

    println!("Key Page URL: {}", key_page_url);

    // Calculate amount for 200 credits
    let key_page_credits = 200u64;
    let key_page_amount = calculate_credits_amount(key_page_credits, oracle);
    println!("Buying {} credits for {} ACME sub-units", key_page_credits, key_page_amount);

    let add_key_page_credits_body = TxBody::add_credits(&key_page_url, &key_page_amount.to_string(), oracle);

    let result = lite_signer.sign_submit_and_wait(
        &lite_token_account,
        &add_key_page_credits_body,
        Some("Add credits to ADI key page"),
        30,
    ).await;

    if result.success {
        println!("AddCredits to key page SUCCESS - TxID: {:?}", result.txid);
    } else {
        println!("AddCredits to key page FAILED: {:?}", result.error);
    }

    // Verify credits were added to key page
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    match query_credits(&client, &key_page_url).await {
        Ok(credits) => println!("Key page credit balance: {}\n", credits),
        Err(e) => println!("Could not query key page: {}\n", e),
    }

    // =========================================================
    // Summary
    // =========================================================
    println!("=== Summary ===\n");
    println!("Created lite identity: {}", lite_identity);
    println!("Created ADI: {}", identity_url);
    println!("ADI Key Book: {}", book_url);
    println!("ADI Key Page: {}", key_page_url);
    println!("\nUsed SmartSigner API for all transactions!");
    println!("No manual version tracking needed.");

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

/// Query account info
async fn query_account(client: &AccumulateClient, url: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let params = json!({
        "scope": url,
        "query": {"queryType": "default"}
    });
    let result: Value = client.v3_client.call_v3("query", params).await?;
    result.get("account")
        .cloned()
        .ok_or_else(|| "Account not found".into())
}
