//! Example 13: ADI-to-ADI Token Transfer with Header Options (Kermit Testnet)
//!
//! This example demonstrates:
//! - Sending ACME tokens between ADI token accounts (ADI-to-ADI transfers)
//! - Using optional transaction header fields:
//!   - memo: Human-readable memo text
//!   - metadata: Binary metadata bytes
//!   - expire: Transaction expiration time (ExpireOptions)
//!   - hold_until: Scheduled execution (HoldUntilOptions)
//!   - authorities: Additional signing authorities
//!
//! Run with: cargo run --example example_13_adi_to_adi_transfer_with_header_options

use accumulate_client::{
    AccumulateClient, AccOptions, TxBody, SmartSigner, HeaderOptions,
    ExpireOptions, HoldUntilOptions,
    poll_for_balance, poll_for_credits, derive_lite_identity_url,
    KERMIT_V2, KERMIT_V3,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SDK Example 13: ADI-to-ADI Transfer with Header Options (Rust) ===\n");
    println!("Endpoint: {}\n", KERMIT_V3);

    // Connect to Kermit testnet
    let v2_url = Url::parse(KERMIT_V2)?;
    let v3_url = Url::parse(KERMIT_V3)?;
    let client = AccumulateClient::new_with_options(v2_url, v3_url, AccOptions::default()).await?;

    // Collect all TxIDs for the final report
    let mut tx_ids: Vec<(&str, String)> = Vec::new();

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

    println!("Requesting funds from faucet (10 times)...");
    for i in 1..=10 {
        let params = json!({"account": &lite_token_account});
        match client.v3_client.call_v3::<Value>("faucet", params).await {
            Ok(response) => {
                let txid = response.get("transactionHash")
                    .or_else(|| response.get("txid"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("submitted");
                println!("  Faucet {}/10: {}", i, txid);
            }
            Err(e) => println!("  Faucet {}/10 failed: {}", i, e),
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

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

    let credits = 2000u64; // More credits for multiple transactions
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
        if let Some(ref txid) = result.txid {
            tx_ids.push(("AddCredits (lite identity)", txid.clone()));
        }
    } else {
        println!("AddCredits FAILED: {:?}", result.error);
        return Ok(());
    }

    // Poll for lite identity credits
    println!("Polling for lite identity credits...");
    let lid_credits = poll_for_credits(&client, &lite_identity, 30).await;
    if lid_credits.is_none() || lid_credits == Some(0) {
        println!("ERROR: Lite identity has no credits. Stopping.");
        return Ok(());
    }
    println!("Lite identity credits confirmed: {:?}\n", lid_credits);

    // =========================================================
    // Step 4: Create an ADI
    // =========================================================
    println!("--- Step 4: Create ADI ---\n");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis();
    let adi_name = format!("sdk-hdropt-{}", timestamp);
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
        println!("CreateIdentity SUCCESS - TxID: {:?}", result.txid);
        if let Some(ref txid) = result.txid {
            tx_ids.push(("CreateIdentity", txid.clone()));
        }
    } else {
        println!("CreateIdentity FAILED: {:?}", result.error);
        return Ok(());
    }

    // Poll to confirm ADI exists
    println!("Polling to confirm ADI creation...");
    if !poll_for_account_exists(&client, &identity_url, 30).await {
        println!("ERROR: ADI not found after creation. Stopping.");
        return Ok(());
    }
    println!("ADI confirmed: {}\n", identity_url);

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
        println!("AddCredits to key page SUCCESS - TxID: {:?}", result.txid);
        if let Some(ref txid) = result.txid {
            tx_ids.push(("AddCredits (key page)", txid.clone()));
        }
    } else {
        println!("AddCredits to key page FAILED: {:?}", result.error);
        return Ok(());
    }

    // Poll for key page credits
    println!("Polling for key page credits...");
    let kp_credits = poll_for_credits(&client, &key_page_url, 30).await;
    if kp_credits.is_none() || kp_credits == Some(0) {
        println!("ERROR: Key page has no credits. Stopping.");
        return Ok(());
    }
    println!("Key page credits confirmed: {:?}\n", kp_credits);

    // =========================================================
    // Step 6: Create ADI Token Accounts
    // =========================================================
    println!("--- Step 6: Create ADI Token Accounts ---\n");

    let mut adi_signer = SmartSigner::new(&client, adi_keypair.clone(), &key_page_url);

    let tokens_account_url = format!("{}/tokens", identity_url);
    let staking_account_url = format!("{}/staking", identity_url);
    let savings_account_url = format!("{}/savings", identity_url);
    let reserve_account_url = format!("{}/reserve", identity_url);

    // Create multiple token accounts for demonstrating transfers
    for (account_url, account_name) in &[
        (&tokens_account_url, "tokens"),
        (&staking_account_url, "staking"),
        (&savings_account_url, "savings"),
        (&reserve_account_url, "reserve"),
    ] {
        println!("Creating {} account: {}", account_name, account_url);
        let create_body = TxBody::create_token_account(account_url, "acc://ACME");
        let result = adi_signer.sign_submit_and_wait(
            &identity_url,
            &create_body,
            Some(&format!("Create {} account", account_name)),
            30,
        ).await;

        if result.success {
            println!("  CreateTokenAccount ({}) SUCCESS - TxID: {:?}", account_name, result.txid);
            if let Some(ref txid) = result.txid {
                tx_ids.push(("CreateTokenAccount", txid.clone()));
            }
            // Poll to confirm account exists
            if poll_for_account_exists(&client, account_url, 30).await {
                println!("  {} account confirmed", account_name);
            } else {
                println!("  WARNING: {} account not confirmed after creation", account_name);
            }
        } else {
            println!("  CreateTokenAccount ({}) FAILED: {:?}", account_name, result.error);
            println!("ERROR: Token account creation failed. Stopping.");
            return Ok(());
        }
    }

    // =========================================================
    // Step 7: Fund ADI tokens account from lite account
    // =========================================================
    println!("\n--- Step 7: Fund ADI tokens account from lite ---\n");

    let fund_amount = 50 * 100_000_000u64; // 50 ACME
    println!("Sending 50 ACME from lite to {}", tokens_account_url);

    let fund_body = TxBody::send_tokens_single(&tokens_account_url, &fund_amount.to_string());

    let result = lite_signer.sign_submit_and_wait(
        &lite_token_account,
        &fund_body,
        Some("Fund ADI tokens account"),
        30,
    ).await;

    if result.success {
        println!("SendTokens SUCCESS - TxID: {:?}", result.txid);
        if let Some(ref txid) = result.txid {
            tx_ids.push(("SendTokens (lite to ADI)", txid.clone()));
        }
    } else {
        println!("SendTokens FAILED: {:?}", result.error);
        return Ok(());
    }

    // Poll for tokens account balance
    println!("Polling for tokens account balance...");
    let tokens_balance = poll_for_token_balance(&client, &tokens_account_url, 30).await;
    if tokens_balance == 0 {
        println!("ERROR: Tokens account has no balance. Stopping.");
        return Ok(());
    }
    println!("Tokens account balance confirmed: {}\n", tokens_balance);

    // =========================================================
    // Step 8: Transfer with MEMO (using sign_submit_and_wait_with_options)
    // =========================================================
    println!("--- Step 8: Transfer with MEMO Header Option ---\n");

    let transfer_amount_1 = 2 * 100_000_000u64; // 2 ACME
    let memo_text = "Payment for SDK example services - Invoice #12345";

    println!("Sending 2 ACME with memo: '{}'", memo_text);
    println!("From: {}", tokens_account_url);
    println!("To: {}\n", staking_account_url);

    let memo_options = HeaderOptions {
        memo: Some(memo_text.to_string()),
        ..Default::default()
    };

    let result = adi_signer.sign_submit_and_wait_with_options(
        &tokens_account_url,
        &TxBody::send_tokens_single(&staking_account_url, &transfer_amount_1.to_string()),
        &memo_options,
        30,
    ).await;

    if result.success {
        println!("Transfer with MEMO SUCCESS!");
        println!("TxID: {:?}\n", result.txid);
        if let Some(ref txid) = result.txid {
            tx_ids.push(("SendTokens (with memo)", txid.clone()));
        }
    } else {
        println!("Transfer with MEMO FAILED: {:?}\n", result.error);
    }

    tokio::time::sleep(Duration::from_secs(9)).await;

    // =========================================================
    // Step 9: Transfer with METADATA (binary metadata)
    // =========================================================
    println!("--- Step 9: Transfer with METADATA Header Option ---\n");

    let transfer_amount_2 = 2 * 100_000_000u64; // 2 ACME
    let metadata_bytes: Vec<u8> = b"Binary metadata: SDK Example 13 Rust".to_vec();

    println!("Sending 2 ACME with metadata: {:?}", String::from_utf8_lossy(&metadata_bytes));
    println!("From: {}", tokens_account_url);
    println!("To: {}\n", savings_account_url);

    let metadata_options = HeaderOptions {
        metadata: Some(metadata_bytes),
        ..Default::default()
    };

    let result = adi_signer.sign_submit_and_wait_with_options(
        &tokens_account_url,
        &TxBody::send_tokens_single(&savings_account_url, &transfer_amount_2.to_string()),
        &metadata_options,
        30,
    ).await;

    if result.success {
        println!("Transfer with METADATA SUCCESS!");
        println!("TxID: {:?}\n", result.txid);
        if let Some(ref txid) = result.txid {
            tx_ids.push(("SendTokens (with metadata)", txid.clone()));
        }
    } else {
        println!("Transfer with METADATA FAILED: {:?}\n", result.error);
    }

    tokio::time::sleep(Duration::from_secs(9)).await;

    // =========================================================
    // Step 10: Transfer with EXPIRE option (expires in 1 hour)
    // =========================================================
    println!("--- Step 10: Transfer with EXPIRE Header Option ---\n");

    let transfer_amount_3 = 2 * 100_000_000u64; // 2 ACME
    let expire_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() + 3600; // 1 hour from now

    println!("Sending 2 ACME with expire time: {} (unix)", expire_time);
    println!("From: {}", tokens_account_url);
    println!("To: {}\n", reserve_account_url);

    let expire_options = HeaderOptions {
        expire: Some(ExpireOptions { at_time: Some(expire_time) }),
        ..Default::default()
    };

    let result = adi_signer.sign_submit_and_wait_with_options(
        &tokens_account_url,
        &TxBody::send_tokens_single(&reserve_account_url, &transfer_amount_3.to_string()),
        &expire_options,
        30,
    ).await;

    if result.success {
        println!("Transfer with EXPIRE SUCCESS!");
        println!("TxID: {:?}\n", result.txid);
        if let Some(ref txid) = result.txid {
            tx_ids.push(("SendTokens (with expire)", txid.clone()));
        }
    } else {
        println!("Transfer with EXPIRE FAILED: {:?}\n", result.error);
    }

    tokio::time::sleep(Duration::from_secs(9)).await;

    // =========================================================
    // Step 11: Transfer with HOLD_UNTIL option (delayed execution)
    // =========================================================
    println!("--- Step 11: Transfer with HOLD_UNTIL Header Option ---\n");

    let transfer_amount_4 = 2 * 100_000_000u64; // 2 ACME
    let hold_block: u64 = 1_000_000; // Example future block number

    println!("Sending 2 ACME with hold_until block: {}", hold_block);
    println!("From: {}", tokens_account_url);
    println!("To: {}", staking_account_url);
    println!("(Transaction will be held until the specified minor block)\n");

    let hold_options = HeaderOptions {
        hold_until: Some(HoldUntilOptions { minor_block: Some(hold_block) }),
        ..Default::default()
    };

    let result = adi_signer.sign_submit_and_wait_with_options(
        &tokens_account_url,
        &TxBody::send_tokens_single(&staking_account_url, &transfer_amount_4.to_string()),
        &hold_options,
        30,
    ).await;

    if result.success {
        println!("Transfer with HOLD_UNTIL SUCCESS!");
        println!("TxID: {:?}\n", result.txid);
        if let Some(ref txid) = result.txid {
            tx_ids.push(("SendTokens (with hold_until)", txid.clone()));
        }
    } else {
        println!("Transfer with HOLD_UNTIL FAILED: {:?}\n", result.error);
    }

    tokio::time::sleep(Duration::from_secs(9)).await;

    // =========================================================
    // Step 12: Transfer with AUTHORITIES option
    // =========================================================
    println!("--- Step 12: Transfer with AUTHORITIES Header Option ---\n");

    let transfer_amount_5 = 2 * 100_000_000u64; // 2 ACME

    // Using same key page for demonstration
    let authorities = vec![key_page_url.clone()];

    println!("Sending 2 ACME with authorities: {:?}", authorities);
    println!("From: {}", tokens_account_url);
    println!("To: {}\n", savings_account_url);

    let auth_options = HeaderOptions {
        authorities: Some(authorities),
        ..Default::default()
    };

    let result = adi_signer.sign_submit_and_wait_with_options(
        &tokens_account_url,
        &TxBody::send_tokens_single(&savings_account_url, &transfer_amount_5.to_string()),
        &auth_options,
        30,
    ).await;

    if result.success {
        println!("Transfer with AUTHORITIES SUCCESS!");
        println!("TxID: {:?}\n", result.txid);
        if let Some(ref txid) = result.txid {
            tx_ids.push(("SendTokens (with authorities)", txid.clone()));
        }
    } else {
        println!("Transfer with AUTHORITIES FAILED: {:?}\n", result.error);
    }

    tokio::time::sleep(Duration::from_secs(9)).await;

    // =========================================================
    // Step 13: Transfer with ALL header options combined
    // =========================================================
    println!("--- Step 13: Transfer with ALL Header Options Combined ---\n");

    let transfer_amount_6 = 2 * 100_000_000u64; // 2 ACME
    let combined_memo = "Complete transaction with all header options";
    let combined_metadata: Vec<u8> = b"Full featured transaction metadata".to_vec();
    let combined_expire_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() + 7200; // 2 hours from now

    println!("Sending 2 ACME with ALL header options:");
    println!("  - memo: '{}'", combined_memo);
    println!("  - metadata: {:?}", String::from_utf8_lossy(&combined_metadata));
    println!("  - expire: {} (unix)", combined_expire_time);
    println!("From: {}", tokens_account_url);
    println!("To: {}\n", reserve_account_url);

    let all_options = HeaderOptions {
        memo: Some(combined_memo.to_string()),
        metadata: Some(combined_metadata),
        expire: Some(ExpireOptions { at_time: Some(combined_expire_time) }),
        ..Default::default()
    };

    let result = adi_signer.sign_submit_and_wait_with_options(
        &tokens_account_url,
        &TxBody::send_tokens_single(&reserve_account_url, &transfer_amount_6.to_string()),
        &all_options,
        30,
    ).await;

    if result.success {
        println!("Transfer with ALL OPTIONS SUCCESS!");
        println!("TxID: {:?}\n", result.txid);
        if let Some(ref txid) = result.txid {
            tx_ids.push(("SendTokens (all options)", txid.clone()));
        }
    } else {
        println!("Transfer with ALL OPTIONS FAILED: {:?}\n", result.error);
    }

    // =========================================================
    // Step 14: Verify balances
    // =========================================================
    println!("--- Step 14: Verify Balances ---\n");

    tokio::time::sleep(Duration::from_secs(15)).await;

    for (account_url, account_name) in &[
        (&tokens_account_url, "Tokens"),
        (&staking_account_url, "Staking"),
        (&savings_account_url, "Savings"),
        (&reserve_account_url, "Reserve"),
    ] {
        match query_balance(&client, account_url).await {
            Ok(bal) => println!("{} account balance: {}", account_name, bal),
            Err(e) => println!("Could not query {} balance: {}", account_name, e),
        }
    }

    // =========================================================
    // Summary
    // =========================================================
    println!("\n=== Summary ===\n");
    println!("Created ADI: {}", identity_url);
    println!("Token Accounts: tokens, staking, savings, reserve");
    println!("\nToken transfers demonstrated with header options:");
    println!("  - MEMO: Human-readable transaction memo");
    println!("  - METADATA: Binary metadata bytes");
    println!("  - EXPIRE: Transaction expiration time");
    println!("  - HOLD_UNTIL: Scheduled execution at specific block");
    println!("  - AUTHORITIES: Additional signing authorities");
    println!("  - ALL COMBINED: Multiple header options together");

    // =========================================================
    // TxID Report
    // =========================================================
    println!("\n=== TRANSACTION IDs FOR VERIFICATION ===\n");
    for (tx_name, txid) in &tx_ids {
        println!("  {}: {}", tx_name, txid);
    }
    println!("\nTotal transactions: {}", tx_ids.len());
    println!("Example 13 COMPLETED SUCCESSFULLY!");

    Ok(())
}

// =============================================================================
// Helper Functions
// =============================================================================

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

/// Query token account balance
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

/// Poll for an account to exist on the network
async fn poll_for_account_exists(client: &AccumulateClient, account_url: &str, max_attempts: u32) -> bool {
    for i in 0..max_attempts {
        let params = json!({
            "scope": account_url,
            "query": {"queryType": "default"}
        });
        match client.v3_client.call_v3::<Value>("query", params).await {
            Ok(result) => {
                if result.get("account").is_some() {
                    return true;
                }
            }
            Err(_) => {}
        }

        if i < max_attempts - 1 {
            println!("  Waiting for account... (attempt {}/{})", i + 1, max_attempts);
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
    false
}

/// Poll for token account balance
async fn poll_for_token_balance(client: &AccumulateClient, account_url: &str, max_attempts: u32) -> u64 {
    for i in 0..max_attempts {
        match query_balance(client, account_url).await {
            Ok(balance_str) => {
                if let Ok(bal) = balance_str.parse::<u64>() {
                    if bal > 0 {
                        return bal;
                    }
                }
            }
            Err(_) => {}
        }

        if i < max_attempts - 1 {
            println!("  Waiting for balance... (attempt {}/{})", i + 1, max_attempts);
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
    0
}
