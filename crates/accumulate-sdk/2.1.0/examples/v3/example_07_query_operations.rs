//! Example 7: Query Operations (Kermit Testnet)
//!
//! This example demonstrates:
//! - Querying accounts (lite and ADI)
//! - Querying transactions by hash
//! - Querying network status
//! - Using the V3 API query methods
//!
//! Run with: cargo run --example example_07_query_operations

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
    println!("=== SDK Example 7: Query Operations ===\n");
    println!("Endpoint: {}\n", KERMIT_V3);

    // Connect to Kermit testnet
    let v2_url = Url::parse(KERMIT_V2)?;
    let v3_url = Url::parse(KERMIT_V3)?;
    let client = AccumulateClient::new_with_options(v2_url, v3_url, AccOptions::default()).await?;

    // =========================================================
    // Step 1: Query Network Status
    // =========================================================
    println!("--- Step 1: Query Network Status ---\n");

    let network_status: Value = client.v3_client.call_v3("network-status", json!({})).await?;

    if let Some(oracle) = network_status.get("oracle") {
        println!("Oracle Info:");
        println!("  Price: {}", oracle.get("price").and_then(|p| p.as_u64()).unwrap_or(0));
    }

    if let Some(globals) = network_status.get("globals") {
        println!("\nNetwork Globals:");
        println!("  Major Block Time: {:?}", globals.get("majorBlockTime"));
    }
    println!();

    // =========================================================
    // Step 2: Generate keys and fund account
    // =========================================================
    println!("--- Step 2: Setup Test Account ---\n");

    let lite_keypair = AccumulateClient::generate_keypair();
    let adi_keypair = AccumulateClient::generate_keypair();

    let lite_public_key = lite_keypair.verifying_key().to_bytes();
    let lite_identity = derive_lite_identity_url(&lite_public_key);
    let lite_token_account = format!("{}/ACME", lite_identity);

    println!("Lite Token Account: {}", lite_token_account);

    // Fund via faucet (5 times for sufficient balance)
    println!("Requesting funds from faucet...");
    for i in 1..=5 {
        let params = json!({"account": &lite_token_account});
        match client.v3_client.call_v3::<Value>("faucet", params).await {
            Ok(_) => println!("  Faucet {}/5: success", i),
            Err(e) => println!("  Faucet {}/5 failed: {}", i, e),
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    // Wait for settlement and poll for balance
    println!("\nWaiting for transactions to settle...");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    println!("Polling for balance...");
    let balance = poll_for_balance(&client, &lite_token_account, 30).await;
    if balance.is_none() || balance == Some(0) {
        println!("ERROR: Account not funded. Stopping.");
        return Ok(());
    }
    println!("Balance confirmed: {:?}\n", balance);

    // =========================================================
    // Step 3: Query Lite Token Account
    // =========================================================
    println!("--- Step 3: Query Lite Token Account ---\n");

    let query_params = json!({
        "scope": &lite_token_account,
        "query": {"queryType": "default"}
    });

    match client.v3_client.call_v3::<Value>("query", query_params).await {
        Ok(result) => {
            if let Some(account) = result.get("account") {
                println!("Account Query Result:");
                println!("  URL: {}", account.get("url").and_then(|u| u.as_str()).unwrap_or("unknown"));
                println!("  Type: {}", account.get("type").and_then(|t| t.as_str()).unwrap_or("unknown"));
                println!("  Balance: {}", account.get("balance").and_then(|b| b.as_str()).unwrap_or("0"));
                println!("  Token URL: {}", account.get("tokenUrl").and_then(|t| t.as_str()).unwrap_or("unknown"));
            }
        }
        Err(e) => println!("Query failed: {}", e),
    }

    // =========================================================
    // Step 4: Add Credits and Create ADI
    // =========================================================
    println!("\n--- Step 4: Create and Query ADI ---\n");

    let mut lite_signer = SmartSigner::new(&client, lite_keypair.clone(), &lite_identity);

    // Add credits
    let oracle = network_status.get("oracle")
        .and_then(|o| o.get("price"))
        .and_then(|p| p.as_u64())
        .unwrap_or(10000000);

    let credits = 500u64;
    let amount = (credits as u128 * 10_000_000_000u128 / oracle as u128) as u64;

    let add_credits_body = TxBody::add_credits(&lite_identity, &amount.to_string(), oracle);
    let result = lite_signer.sign_submit_and_wait(
        &lite_token_account,
        &add_credits_body,
        Some("Add credits"),
        30,
    ).await;

    if !result.success {
        println!("AddCredits failed: {:?}", result.error);
        return Ok(());
    }
    println!("Credits added successfully");

    // Wait for credits to be available
    println!("Waiting for credits to settle...");
    let credits_confirmed = poll_for_credits(&client, &lite_identity, 30).await;
    println!("Credits confirmed: {:?}", credits_confirmed);

    // Create ADI
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis();
    let adi_name = format!("sdk-query-{}", timestamp);
    let identity_url = format!("acc://{}.acme", adi_name);
    let book_url = format!("{}/book", identity_url);

    let adi_public_key = adi_keypair.verifying_key().to_bytes();
    let adi_key_hash = sha256_hash(&adi_public_key);
    let adi_key_hash_hex = hex::encode(adi_key_hash);

    let create_adi_body = TxBody::create_identity(&identity_url, &book_url, &adi_key_hash_hex);
    let result = lite_signer.sign_submit_and_wait(
        &lite_token_account,
        &create_adi_body,
        Some("Create ADI for query test"),
        30,
    ).await;

    if !result.success {
        println!("CreateIdentity failed: {:?}", result.error);
        return Ok(());
    }

    println!("ADI created: {}", identity_url);

    // Store txid for later query
    let txid = result.txid.clone().unwrap_or_default();

    // Wait and poll for ADI to be queryable
    println!("\nWaiting for ADI to be available...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Poll until ADI is available
    let mut adi_available = false;
    for attempt in 1..=30 {
        let query_params = json!({
            "scope": &identity_url,
            "query": {"queryType": "default"}
        });

        match client.v3_client.call_v3::<Value>("query", query_params).await {
            Ok(result) => {
                if result.get("account").is_some() {
                    adi_available = true;
                    break;
                }
            }
            Err(_) => {}
        }

        if attempt < 30 {
            println!("  ADI not available yet... (attempt {}/30)", attempt);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    if !adi_available {
        println!("WARNING: ADI not available after 30 attempts");
    }

    // =========================================================
    // Step 5: Query ADI
    // =========================================================
    println!("\n--- Step 5: Query ADI ---\n");

    let query_params = json!({
        "scope": &identity_url,
        "query": {"queryType": "default"}
    });

    match client.v3_client.call_v3::<Value>("query", query_params).await {
        Ok(result) => {
            if let Some(account) = result.get("account") {
                println!("ADI Query Result:");
                println!("  URL: {}", account.get("url").and_then(|u| u.as_str()).unwrap_or("unknown"));
                println!("  Type: {}", account.get("type").and_then(|t| t.as_str()).unwrap_or("unknown"));

                if let Some(authorities) = account.get("authorities") {
                    println!("  Authorities: {:?}", authorities);
                }
            }
        }
        Err(e) => println!("ADI query failed: {}", e),
    }

    // =========================================================
    // Step 6: Query Transaction
    // =========================================================
    println!("\n--- Step 6: Query Transaction ---\n");

    // Extract transaction hash from txid (format: acc://hash@url)
    if let Some(hash_part) = txid.strip_prefix("acc://") {
        if let Some(tx_hash) = hash_part.split('@').next() {
            println!("Querying transaction: {}", tx_hash);

            let tx_query_params = json!({
                "scope": &txid,
                "query": {"queryType": "default"}
            });

            match client.v3_client.call_v3::<Value>("query", tx_query_params).await {
                Ok(result) => {
                    println!("\nTransaction Query Result:");
                    if let Some(message) = result.get("message") {
                        if let Some(tx) = message.get("transaction") {
                            if let Some(header) = tx.get("header") {
                                println!("  Principal: {}",
                                    header.get("principal").and_then(|p| p.as_str()).unwrap_or("unknown"));
                                if let Some(memo) = header.get("memo") {
                                    println!("  Memo: {}", memo.as_str().unwrap_or(""));
                                }
                            }
                            if let Some(body) = tx.get("body") {
                                println!("  Body Type: {}",
                                    body.get("type").and_then(|t| t.as_str()).unwrap_or("unknown"));
                            }
                        }
                    }
                    if let Some(status) = result.get("status") {
                        println!("  Status Code: {}",
                            status.get("code").and_then(|c| c.as_u64()).unwrap_or(0));
                        println!("  Delivered: {}",
                            status.get("delivered").and_then(|d| d.as_bool()).unwrap_or(false));
                    }
                }
                Err(e) => println!("Transaction query failed: {}", e),
            }
        }
    }

    // =========================================================
    // Step 7: Query Key Book and Key Page
    // =========================================================
    println!("\n--- Step 7: Query Key Book and Key Page ---\n");

    let key_page_url = format!("{}/1", book_url);

    // Query key book
    let query_params = json!({
        "scope": &book_url,
        "query": {"queryType": "default"}
    });

    match client.v3_client.call_v3::<Value>("query", query_params).await {
        Ok(result) => {
            if let Some(account) = result.get("account") {
                println!("Key Book Query:");
                println!("  URL: {}", account.get("url").and_then(|u| u.as_str()).unwrap_or("unknown"));
                println!("  Type: {}", account.get("type").and_then(|t| t.as_str()).unwrap_or("unknown"));
                println!("  Page Count: {}", account.get("pageCount").and_then(|c| c.as_u64()).unwrap_or(0));
            }
        }
        Err(e) => println!("Key book query failed: {}", e),
    }

    // Query key page
    let query_params = json!({
        "scope": &key_page_url,
        "query": {"queryType": "default"}
    });

    match client.v3_client.call_v3::<Value>("query", query_params).await {
        Ok(result) => {
            if let Some(account) = result.get("account") {
                println!("\nKey Page Query:");
                println!("  URL: {}", account.get("url").and_then(|u| u.as_str()).unwrap_or("unknown"));
                println!("  Type: {}", account.get("type").and_then(|t| t.as_str()).unwrap_or("unknown"));
                println!("  Credit Balance: {}", account.get("creditBalance").and_then(|c| c.as_u64()).unwrap_or(0));
                println!("  Threshold: {}", account.get("acceptThreshold").and_then(|t| t.as_u64()).unwrap_or(0));

                if let Some(keys) = account.get("keys").and_then(|k| k.as_array()) {
                    println!("  Keys: {} key(s)", keys.len());
                    for (i, key) in keys.iter().enumerate() {
                        if let Some(hash) = key.get("publicKeyHash").and_then(|h| h.as_str()) {
                            println!("    Key {}: {}...", i + 1, &hash[..32.min(hash.len())]);
                        }
                    }
                }
            }
        }
        Err(e) => println!("Key page query failed: {}", e),
    }

    // =========================================================
    // Summary
    // =========================================================
    println!("\n=== Summary ===\n");
    println!("Demonstrated query operations:");
    println!("  - Network status query");
    println!("  - Lite token account query");
    println!("  - ADI identity query");
    println!("  - Transaction query by ID");
    println!("  - Key book and key page queries");
    println!("\nAll queries used the V3 API!");

    Ok(())
}

/// SHA-256 hash helper
fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}
