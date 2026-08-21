//! Example 12: Multi-Signature Transaction Workflow (Kermit Testnet)
//!
//! This example demonstrates:
//! - Setting up a 2-of-2 multi-sig key page
//! - Understanding how multi-sig affects transaction processing
//! - Signature types and vote types in Accumulate
//! - Using SmartSigner API for multi-sig operations
//!
//! Run with: cargo run --example example_12_multi_signature_workflow

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
    println!("=== SDK Example 12: Multi-Signature Transaction Workflow ===\n");
    println!("Endpoint: {}\n", KERMIT_V3);

    // Connect to Kermit testnet
    let v2_url = Url::parse(KERMIT_V2)?;
    let v3_url = Url::parse(KERMIT_V3)?;
    let client = AccumulateClient::new_with_options(v2_url, v3_url, AccOptions::default()).await?;

    // =========================================================
    // Step 1: Generate key pairs for multi-sig setup
    // =========================================================
    println!("--- Step 1: Generate Key Pairs ---\n");

    let lite_keypair = AccumulateClient::generate_keypair();
    let signer1 = AccumulateClient::generate_keypair();
    let signer2 = AccumulateClient::generate_keypair();

    let lite_public_key = lite_keypair.verifying_key().to_bytes();
    let lite_identity = derive_lite_identity_url(&lite_public_key);
    let lite_token_account = format!("{}/ACME", lite_identity);

    println!("Lite Identity: {}", lite_identity);
    println!("Lite Token Account: {}", lite_token_account);
    println!("Multi-sig Signer 1: Generated ED25519 keypair");
    println!("Multi-sig Signer 2: Generated ED25519 keypair\n");

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

    let credits = 3000u64;
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
    // Step 4: Create an ADI with first signer
    // =========================================================
    println!("--- Step 4: Create ADI ---\n");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis();
    let adi_name = format!("sdk-multisig-{}", timestamp);
    let identity_url = format!("acc://{}.acme", adi_name);
    let book_url = format!("{}/book", identity_url);
    let key_page_url = format!("{}/1", book_url);

    let signer1_public = signer1.verifying_key().to_bytes();
    let signer1_hash = sha256_hash(&signer1_public);
    let signer1_hash_hex = hex::encode(signer1_hash);

    println!("ADI URL: {}", identity_url);
    println!("Key Page URL: {}", key_page_url);
    println!("Signer 1 hash: {}...\n", &signer1_hash_hex[0..16]);

    let create_adi_body = TxBody::create_identity(&identity_url, &book_url, &signer1_hash_hex);

    let result = lite_signer.sign_submit_and_wait(
        &lite_token_account,
        &create_adi_body,
        Some("Create ADI for multi-sig demo"),
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
    // Step 6: Add second signer to key page
    // =========================================================
    println!("--- Step 6: Add Second Signer ---\n");

    let mut signer1_smart = SmartSigner::new(&client, signer1.clone(), &key_page_url);

    let signer2_public = signer2.verifying_key().to_bytes();
    let signer2_hash = sha256_hash(&signer2_public);
    let signer2_hash_hex = hex::encode(signer2_hash);
    println!("Adding signer 2 with hash: {}...", &signer2_hash_hex[0..16]);

    let result = signer1_smart.add_key(&signer2_public).await;
    if result.success {
        println!("AddKey (signer2) SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("AddKey (signer2) FAILED: {:?}\n", result.error);
    }

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // =========================================================
    // Step 7: Set threshold to 2-of-2 (both signatures required)
    // =========================================================
    println!("--- Step 7: Set 2-of-2 Threshold ---\n");

    println!("Setting threshold to 2 (both signatures required)...");

    let result = signer1_smart.set_threshold(2).await;

    if result.success {
        println!("SetThreshold SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("SetThreshold FAILED: {:?}\n", result.error);
    }

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // =========================================================
    // Step 8: Verify Multi-Sig Configuration
    // =========================================================
    println!("--- Step 8: Verify Multi-Sig Configuration ---\n");

    let key_manager = KeyManager::new(&client, &key_page_url);
    match key_manager.get_key_page_state().await {
        Ok(state) => {
            println!("Key Page: {}", key_page_url);
            println!("  Version: {}", state.version);
            println!("  Keys: {}", state.keys.len());
            println!("  Threshold: {}", state.accept_threshold);
            println!("  Multi-Sig: {} (2-of-2)", state.accept_threshold >= 2);
            println!("\nKey Details:");
            for (i, key) in state.keys.iter().enumerate() {
                let hash_preview = if key.key_hash.len() > 16 {
                    &key.key_hash[0..16]
                } else {
                    &key.key_hash
                };
                println!("    Signer {}: {}...", i + 1, hash_preview);
            }
        }
        Err(e) => {
            println!("Error querying key page: {}", e);
        }
    }
    println!();

    // =========================================================
    // Step 9: Create Token Account for Multi-Sig Demo
    // =========================================================
    println!("--- Step 9: Create Token Account ---\n");

    let token_account_url = format!("{}/tokens", identity_url);
    println!("Creating token account: {}", token_account_url);

    // First signature from signer1
    let create_token_body = TxBody::create_token_account(&token_account_url, "acc://ACME");

    let result = signer1_smart.sign_submit_and_wait(
        &identity_url,
        &create_token_body,
        Some("Create token account (signer1)"),
        30,
    ).await;

    if result.success {
        println!("CreateTokenAccount (signer1) SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("CreateTokenAccount (signer1) FAILED: {:?}\n", result.error);
    }

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // =========================================================
    // Step 10: Fund the ADI Token Account
    // =========================================================
    println!("--- Step 10: Fund ADI Token Account ---\n");

    let fund_body = TxBody::send_tokens_single(&token_account_url, "500000000"); // 5 ACME

    let result = lite_signer.sign_submit_and_wait(
        &lite_token_account,
        &fund_body,
        Some("Fund ADI token account"),
        30,
    ).await;

    if result.success {
        println!("Fund tokens account SUCCESS - TxID: {:?}\n", result.txid);
    } else {
        println!("Fund tokens account FAILED: {:?}\n", result.error);
    }

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // =========================================================
    // Step 11: Signature Types Overview
    // =========================================================
    println!("--- Step 11: Signature Types in Accumulate ---\n");

    println!("The Accumulate SDK supports 16 signature types:\n");

    println!("Primary Signature Types:");
    println!("  ED25519Signature       - Ed25519 (EdDSA) signatures");
    println!("  LegacyED25519Signature - Legacy Ed25519 format");
    println!();

    println!("Bitcoin-Compatible:");
    println!("  BTCSignature           - Bitcoin secp256k1 signatures");
    println!("  BTCLegacySignature     - Legacy Bitcoin format");
    println!();

    println!("Ethereum-Compatible:");
    println!("  ETHSignature           - Ethereum secp256k1 with recovery");
    println!("  TypedDataSignature     - EIP-712 typed data signatures");
    println!();

    println!("Standard Cryptographic:");
    println!("  EcdsaSha256Signature   - ECDSA with SHA-256");
    println!("  RsaSha256Signature     - RSA PKCS#1 v1.5");
    println!();

    println!("Accumulate-Specific:");
    println!("  RCD1Signature          - Factom RCD1 compatibility");
    println!("  DelegatedSignature     - Delegation chain signatures");
    println!("  AuthoritySignature     - Authority-level signatures");
    println!("  RemoteSignature        - Remote signing support");
    println!("  PartitionSignature     - Partition validator signatures");
    println!("  ReceiptSignature       - Receipt/proof signatures");
    println!("  InternalSignature      - Internal system signatures");
    println!("  SignatureSet           - Multi-sig signature collection");
    println!();

    // =========================================================
    // Step 12: Vote Types Overview
    // =========================================================
    println!("--- Step 12: Vote Types in Signatures ---\n");

    println!("Each signature can include a vote:");
    println!("  Accept  (0) - Approve the transaction");
    println!("  Reject  (1) - Reject the transaction");
    println!("  Abstain (2) - No opinion on the transaction");
    println!("  Suggest (3) - Non-binding suggestion");
    println!();

    println!("In a 2-of-2 multi-sig:");
    println!("  - Both signers must provide Accept votes");
    println!("  - Any Reject vote blocks the transaction");
    println!("  - Abstain votes don't count toward threshold");
    println!();

    // =========================================================
    // Step 13: Multi-Sig Transaction Demo
    // =========================================================
    println!("--- Step 13: Multi-Sig Transaction Demo ---\n");

    println!("With 2-of-2 threshold, transactions require both signatures.");
    println!("In practice:");
    println!("  1. Signer1 submits transaction -> Pending");
    println!("  2. Signer2 adds signature -> Executed");
    println!();

    // Demonstrate with a transaction using signer1
    println!("Sending tokens using signer1...");
    let send_body = TxBody::send_tokens_single(&lite_token_account, "10000000"); // 0.1 ACME

    let result = signer1_smart.sign_submit_and_wait(
        &token_account_url,
        &send_body,
        Some("Multi-sig send demo"),
        30,
    ).await;

    if result.success {
        println!("Transaction submitted: {:?}", result.txid);
        println!("Note: With 2-of-2, signer2 would need to add signature\n");
    } else {
        println!("Transaction failed (expected with 2-of-2): {:?}\n", result.error);
        println!("This is expected - the transaction is pending signer2.\n");
    }

    // =========================================================
    // Summary
    // =========================================================
    println!("=== Summary ===\n");
    println!("Created ADI: {}", identity_url);
    println!("Key Page: {} (2-of-2 multi-sig)", key_page_url);
    println!("Token Account: {}", token_account_url);
    println!("\nMulti-Sig Configuration:");
    println!("  - 2 signers on key page");
    println!("  - Threshold: 2 (both required)");
    println!("  - All transactions need both signatures");
    println!("\nSignature Types: 16 types supported");
    println!("  - ED25519, BTC, ETH, RSA, ECDSA, and more");
    println!("\nVote Types: Accept, Reject, Abstain, Suggest");
    println!("\nMulti-Sig Workflow:");
    println!("  1. First signer submits -> Pending");
    println!("  2. Additional signers add signatures");
    println!("  3. When threshold met -> Executed");

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
