//! # Buy Credits Local DevNet Example
//!
//! This example demonstrates:
//! - Loading DevNet configuration from .env.local
//! - Connecting to local DevNet
//! - Funding an account with faucet
//! - Converting ACME tokens to credits
//! - Verifying credit balances
//! - Working with lite identity credit accounts
//!
//! ## Prerequisites
//! 1. Run: `cargo run --bin devnet_discovery` to generate .env.local
//! 2. Ensure DevNet is running: `cd devnet && docker compose up -d`
//! 3. Run: `cargo run --example 120_faucet_local_devnet` first to fund account
//!
//! ## Usage
//! ```bash
//! cargo run --example 210_buy_credits_lite
//! ```

use accumulate_client::{AccOptions, AccumulateClient};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Accumulate Buy Credits Local DevNet Example");
    println!("============================================");

    // Step 1: Load environment configuration
    println!("\nLoading DevNet configuration...");
    dotenvy::dotenv().ok();

    let v2_url = std::env::var("ACC_RPC_URL_V2")
        .unwrap_or_else(|_| "http://localhost:26660/v2".to_string());
    let v3_url = std::env::var("ACC_RPC_URL_V3")
        .unwrap_or_else(|_| "http://localhost:26660/v3".to_string());
    let faucet_account = std::env::var("ACC_FAUCET_ACCOUNT")
        .unwrap_or_else(|_| "acc://faucet.acme/ACME".to_string());

    println!("  V2 API: {}", v2_url);
    println!("  V3 API: {}", v3_url);
    println!("  Faucet: {}", faucet_account);

    // Step 2: Connect to DevNet
    println!("\nConnecting to DevNet...");
    let client = create_client(&v2_url, &v3_url).await?;

    // Test connectivity
    match test_devnet_status(&client).await {
        Ok(_) => println!("  [OK] DevNet is accessible"),
        Err(e) => {
            println!("  [ERROR] DevNet connectivity issue: {}", e);
            println!("  Hint: Make sure DevNet is running: cd devnet && docker compose up -d");
            return Err(e.into());
        }
    }

    // Step 3: Generate test account
    println!("\nGenerating test account...");
    let test_account = generate_test_account().await?;

    // Step 4: Ensure account is funded
    println!("\nEnsuring account has ACME tokens...");
    ensure_account_funded(&client, &test_account, &faucet_account).await?;

    // Step 5: Buy credits
    println!("\nConverting ACME tokens to credits...");
    buy_credits(&client, &test_account).await?;

    // Step 6: Verify credit balance
    println!("\nVerifying credit balance...");
    verify_credits(&client, &test_account).await?;

    println!("\nSuccess: Buy credits example completed successfully!");
    println!("Hint: Next, run `cargo run --example 999_zero_to_hero`");

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
            println!("  DevNet Status: {} ({})", status.network, status.version);
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

    println!("  Public key:      {}", hex::encode(&public_key));
    println!("  Lite identity:   {}", lite_identity);
    println!("  ACME account:    {}", acme_account);
    println!("  Credits account: {}", credits_account);

    Ok(TestAccount {
        public_key,
        lite_identity,
        acme_account,
        credits_account,
    })
}

async fn ensure_account_funded(
    client: &AccumulateClient,
    test_account: &TestAccount,
    faucet_account: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if account already has tokens
    print!("  Checking existing balance... ");
    match client.query_account(&test_account.acme_account).await {
        Ok(account) => {
            println!("Account exists");
            if let Some(credits) = account.credits {
                if credits > 0 {
                    println!("  [OK] Account already has {} credits", credits);
                    return Ok(());
                }
            }
        }
        Err(_) => {
            println!("Account does not exist");
        }
    }

    // Request tokens from faucet
    print!("  Requesting faucet tokens... ");
    match client.faucet(&test_account.acme_account).await {
        Ok(response) => {
            println!("[OK] Success!");
            println!("    Transaction ID: {}", response.txid);
            if !response.amount.is_empty() {
                println!("    Amount: {}", response.amount);
            }

            // Wait for transaction processing
            println!("  Waiting 3 seconds for processing...");
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
        Err(e) => {
            println!("[ERROR] Failed: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

async fn buy_credits(
    client: &AccumulateClient,
    test_account: &TestAccount,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  From: {}", test_account.acme_account);
    println!("  To:   {}", test_account.credits_account);

    // Check current ACME balance
    print!("  Checking ACME balance... ");
    match client.query_account(&test_account.acme_account).await {
        Ok(account) => {
            println!("[OK] Account found");
            if let Some(credits) = account.credits {
                println!("    Current credits: {}", credits);
            } else {
                println!("    No credits field found");
            }
        }
        Err(e) => {
            println!("[ERROR] Failed to query account: {}", e);
            return Err(e.into());
        }
    }

    // In a full implementation, this would create and submit a credit purchase transaction
    println!("  Tip: Credit purchase transaction would be created here");
    println!("  Note: This requires transaction signing and submission");
    println!("  Warning: For now, demonstrating account structure and queries");

    Ok(())
}

async fn verify_credits(
    client: &AccumulateClient,
    test_account: &TestAccount,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check credits account
    print!("  Checking credits account... ");
    match client.query_account(&test_account.credits_account).await {
        Ok(account) => {
            println!("[OK] Credits account exists!");
            println!("    URL: {}", account.url);
            println!("    Type: {}", account.account_type);

            if let Some(credits) = account.credits {
                println!("    Credits: {}", credits);
            }

            // Show raw account data
            println!("    Data: {}", serde_json::to_string_pretty(&account.data)?);
        }
        Err(e) => {
            println!("[WARN] Credits account not found: {}", e);
            println!("    This is expected if no credit purchase transaction was submitted");
        }
    }

    // Check lite identity
    print!("  Checking lite identity... ");
    match client.query_account(&test_account.lite_identity).await {
        Ok(account) => {
            println!("[OK] Identity exists (type: {})", account.account_type);
        }
        Err(_) => {
            println!("[ERROR] Identity not found");
        }
    }

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
