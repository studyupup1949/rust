//! # Zero to Hero - Complete Accumulate Flow
//!
//! This comprehensive example demonstrates the complete Accumulate workflow:
//! 1. DevNet discovery and configuration
//! 2. Key generation and lite identity creation
//! 3. Faucet funding and account verification
//! 4. Credit purchasing (preparation)
//! 5. ADI (Accumulate Digital Identity) preparation
//! 6. Token account preparation
//! 7. Data account preparation and writing
//! 8. Token transfers preparation
//!
//! ## Prerequisites
//! 1. Run: `cargo run --bin devnet_discovery` to generate .env.local
//! 2. Ensure DevNet is running: `cd devnet && docker compose up -d`
//!
//! ## Usage
//! ```bash
//! cargo run --example 999_zero_to_hero
//! ```
//!
//! This is the end-to-end demonstration of Accumulate capabilities.

use accumulate_client::{AccOptions, AccumulateClient};
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Accumulate Zero to Hero Example");
    println!("================================");
    println!("This example demonstrates the complete Accumulate workflow!");
    println!();

    // Step 1: Load environment configuration
    println!("Loading DevNet configuration...");
    dotenvy::dotenv().ok();

    let v2_url = std::env::var("ACC_RPC_URL_V2")
        .unwrap_or_else(|_| "http://localhost:26660/v2".to_string());
    let v3_url = std::env::var("ACC_RPC_URL_V3")
        .unwrap_or_else(|_| "http://localhost:26660/v3".to_string());
    let faucet_account = std::env::var("ACC_FAUCET_ACCOUNT")
        .unwrap_or_else(|_| "acc://faucet.acme/ACME".to_string());

    println!("Configuration:");
    println!("   V2 Endpoint: {}", v2_url);
    println!("   V3 Endpoint: {}", v3_url);
    println!("   Faucet: {}", faucet_account);
    println!();

    // Step 2: Connect to DevNet
    println!("Connecting to DevNet...");
    let client = create_client(&v2_url, &v3_url).await?;

    // Verify DevNet is running
    match test_devnet_status(&client).await {
        Ok(_) => println!("   [OK] DevNet is accessible"),
        Err(e) => {
            println!("   [ERROR] DevNet connectivity issue: {}", e);
            println!("   Hint: Make sure DevNet is running: cd devnet && docker compose up -d");
            return Err(e.into());
        }
    }
    println!();

    // =======================
    // STEP 3: Key Generation
    // =======================
    println!("STEP 3: Key Generation and Lite Identity");
    println!("-----------------------------------------");

    let test_account = generate_test_account().await?;

    // ==========================
    // STEP 4: Initial Funding
    // ==========================
    println!("STEP 4: Initial Faucet Funding");
    println!("------------------------------");

    request_faucet_tokens(&client, &test_account, &faucet_account).await?;

    // ========================
    // STEP 5: Buy Credits
    // ========================
    println!("STEP 5: Purchasing Credits (Preparation)");
    println!("-----------------------------------------");
    demonstrate_credit_purchase(&test_account).await?;

    // ==========================
    // STEP 6: Create ADI
    // ==========================
    println!("STEP 6: Creating ADI (Accumulate Digital Identity)");
    println!("---------------------------------------------------");
    let adi_url = demonstrate_adi_creation(&test_account).await?;

    // =============================
    // STEP 7: Create Token Account
    // =============================
    println!("STEP 7: Creating Token Account");
    println!("-------------------------------");
    let token_account_url = demonstrate_token_account_creation(&adi_url).await?;

    // ===========================
    // STEP 8: Create Data Account
    // ===========================
    println!("STEP 8: Creating Data Account");
    println!("------------------------------");
    let data_account_url = demonstrate_data_account_creation(&adi_url).await?;

    // =======================
    // STEP 9: Write Data
    // =======================
    println!("STEP 9: Writing Data");
    println!("--------------------");
    demonstrate_data_writing(&data_account_url).await?;

    // ===========================
    // STEP 10: Token Transfer
    // ===========================
    println!("STEP 10: Token Transfer (Lite to ADI)");
    println!("-------------------------------------");
    demonstrate_token_transfer(&test_account, &token_account_url).await?;

    // ================================
    // STEP 11: Summary and Next Steps
    // ================================
    print_summary(&test_account, &adi_url, &token_account_url, &data_account_url).await?;

    Ok(())
}

async fn create_client(v2_url: &str, v3_url: &str) -> Result<AccumulateClient, Box<dyn std::error::Error>> {
    use url::Url;

    let v2_parsed = Url::parse(v2_url)?;
    let v3_parsed = Url::parse(v3_url)?;

    let options = AccOptions {
        timeout: Duration::from_secs(15),
        headers: std::collections::HashMap::new(),
    };

    AccumulateClient::from_endpoints(v2_parsed, v3_parsed, options).await
        .map_err(|e| e.into())
}

async fn test_devnet_status(client: &AccumulateClient) -> Result<(), Box<dyn std::error::Error>> {
    match client.status().await {
        Ok(status) => {
            println!("   DevNet Status: {} ({})", status.network, status.version);
            Ok(())
        }
        Err(e) => {
            Err(format!("Failed to get DevNet status: {}", e).into())
        }
    }
}

#[derive(Debug)]
struct TestAccount {
    public_key: [u8; 32],
    lite_identity: String,
    acme_account: String,
    credits_account: String,
}

async fn generate_test_account() -> Result<TestAccount, Box<dyn std::error::Error>> {
    use accumulate_client::crypto::ed25519::Ed25519Signer;

    let signer = Ed25519Signer::generate();
    let public_key = signer.public_key_bytes();
    let lite_identity = derive_lite_identity_url(&public_key);
    let acme_account = format!("{}/ACME", lite_identity);
    let credits_account = format!("{}/credits", lite_identity);

    println!("   User Public Key: {}", hex::encode(&public_key));
    println!("   Lite Identity:   {}", lite_identity);
    println!("   ACME Account:    {}", acme_account);
    println!("   Credits Account: {}", credits_account);
    println!();

    Ok(TestAccount {
        public_key,
        lite_identity,
        acme_account,
        credits_account,
    })
}

async fn request_faucet_tokens(
    client: &AccumulateClient,
    test_account: &TestAccount,
    faucet_account: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   Requesting tokens from faucet...");
    match client.faucet(&test_account.acme_account).await {
        Ok(response) => {
            println!("   [OK] Faucet Success! TX: {}", response.txid);
            if !response.amount.is_empty() {
                println!("   Amount: {}", response.amount);
            }

            // Wait for transaction processing
            println!("   Waiting 3 seconds for processing...");
            tokio::time::sleep(Duration::from_secs(3)).await;

            // Verify funding
            match client.query_account(&test_account.acme_account).await {
                Ok(account) => {
                    println!("   [OK] Account funded successfully");
                    println!("   Account type: {}", account.account_type);
                }
                Err(e) => {
                    println!("   [WARN] Could not verify funding: {}", e);
                }
            }
        }
        Err(e) => {
            println!("   [ERROR] Faucet failed: {}", e);
            return Err(e.into());
        }
    }
    println!();
    Ok(())
}

async fn demonstrate_credit_purchase(test_account: &TestAccount) -> Result<(), Box<dyn std::error::Error>> {
    println!("   Creating credit purchase transaction...");
    let credit_amount = 10000u64;

    println!("   Transaction body would be created here");
    println!("   From: {}", test_account.acme_account);
    println!("   To:   {}", test_account.credits_account);
    println!("   Amount: {} ACME", credit_amount);
    println!("   Note: Transaction simulation (would submit to network)");
    println!();
    Ok(())
}

async fn demonstrate_adi_creation(test_account: &TestAccount) -> Result<String, Box<dyn std::error::Error>> {
    let adi_url = format!("acc://user-{}.acme", &hex::encode(&test_account.public_key)[0..8]);
    println!("   ADI URL: {}", adi_url);
    println!("   ADI creation transaction would be prepared here");
    println!("   Note: Transaction simulation (would submit to network)");
    println!();
    Ok(adi_url)
}

async fn demonstrate_token_account_creation(adi_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let token_account_url = format!("{}/tokens", adi_url);
    println!("   Token Account: {}", token_account_url);

    let create_token_account_tx = json!({
        "type": "createTokenAccount",
        "url": token_account_url,
        "tokenUrl": "acc://ACME",
        "keyBook": format!("{}/book", adi_url)
    });

    println!("   Token account creation transaction: {}", serde_json::to_string_pretty(&create_token_account_tx)?);
    println!("   Note: Transaction simulation (would submit to network)");
    println!();
    Ok(token_account_url)
}

async fn demonstrate_data_account_creation(adi_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let data_account_url = format!("{}/data", adi_url);
    println!("   Data Account: {}", data_account_url);

    let create_data_account_tx = json!({
        "type": "createDataAccount",
        "url": data_account_url,
        "keyBook": format!("{}/book", adi_url)
    });

    println!("   Data account creation transaction: {}", serde_json::to_string_pretty(&create_data_account_tx)?);
    println!("   Note: Transaction simulation (would submit to network)");
    println!();
    Ok(data_account_url)
}

async fn demonstrate_data_writing(data_account_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data_to_write = json!({
        "timestamp": chrono::Utc::now().timestamp(),
        "message": "Hello from Accumulate Rust SDK!",
        "version": "1.0",
        "metadata": {
            "example": "zero-to-hero",
            "language": "rust"
        }
    });

    println!("   Data to write: {}", serde_json::to_string_pretty(&data_to_write)?);

    let write_data_tx = json!({
        "type": "writeData",
        "data": data_to_write,
        "account": data_account_url
    });

    println!("   Write data transaction prepared");
    println!("   Note: Transaction simulation (would submit to network)");
    println!();
    Ok(())
}

async fn demonstrate_token_transfer(test_account: &TestAccount, token_account_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let transfer_amount = 1000u64;
    println!("   Transfer amount: {} ACME", transfer_amount);
    println!("   From: {} (Lite)", test_account.acme_account);
    println!("   To:   {} (ADI)", token_account_url);

    println!("   Token transfer transaction would be prepared here");
    println!("   Note: Transaction simulation (would submit to network)");
    println!();
    Ok(())
}

async fn print_summary(
    test_account: &TestAccount,
    adi_url: &str,
    token_account_url: &str,
    data_account_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Success: STEP 11: Zero to Hero Complete!");
    println!("----------------------------------------");
    println!();
    println!("Congratulations! You have successfully completed the Accumulate zero-to-hero flow!");
    println!();
    println!("Summary of what we accomplished:");
    println!("   1. [OK] Generated Ed25519 keypair");
    println!("   2. [OK] Created lite identity: {}", test_account.lite_identity);
    println!("   3. [OK] Funded account with faucet");
    println!("   4. [OK] Prepared credit purchase");
    println!("   5. [OK] Prepared ADI creation: {}", adi_url);
    println!("   6. [OK] Prepared token account: {}", token_account_url);
    println!("   7. [OK] Prepared data account: {}", data_account_url);
    println!("   8. [OK] Prepared data writing");
    println!("   9. [OK] Prepared token transfer");
    println!();
    println!("What's Next:");
    println!("   - Run individual examples (100, 120, 210) for detailed flows");
    println!("   - Explore V3 transaction submission");
    println!("   - Try advanced features like multi-sig and anchoring");
    println!("   - Build your own Accumulate applications!");
    println!();
    println!("Resources:");
    println!("   - Accumulate Documentation: https://docs.accumulatenetwork.io");
    println!("   - SDK Examples: ./examples/");
    println!("   - Test Suite: cargo test --all-features");
    println!();

    Ok(())
}

// Helper function for lite identity derivation (same as other examples)
fn derive_lite_identity_url(public_key: &[u8; 32]) -> String {
    use accumulate_client::crypto::ed25519::sha256;

    let hash = sha256(public_key);
    let lite_id_bytes = &hash[0..20];
    let lite_id_hex = hex::encode(lite_id_bytes);

    format!("acc://{}.acme", lite_id_hex)
}
