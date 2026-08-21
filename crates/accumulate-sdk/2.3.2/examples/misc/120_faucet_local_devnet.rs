//! # Faucet Local DevNet Example
//!
//! This example demonstrates:
//! - Loading DevNet configuration from .env.local
//! - Connecting to local DevNet (or Kermit testnet as fallback)
//! - Requesting tokens from the faucet
//! - Checking account balances
//! - Querying transaction status
//! - Working with lite token accounts
//!
//! ## Prerequisites
//! 1. Run: `cargo run --bin devnet_discovery` to generate .env.local
//! 2. Ensure DevNet is running: `cd devnet && docker compose up -d`
//! 3. Or use Kermit testnet if DevNet is unavailable
//!
//! ## Usage
//! ```bash
//! cargo run --example 120_faucet_local_devnet
//! ```

use accumulate_client::{AccOptions, AccumulateClient, KERMIT_V2, KERMIT_V3};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Accumulate Faucet Example");
    println!("=========================");

    // Step 1: Load environment configuration or use defaults
    println!("\nLoading configuration...");
    dotenvy::dotenv().ok();

    let v2_url = std::env::var("ACC_RPC_URL_V2")
        .unwrap_or_else(|_| "http://localhost:26660/v2".to_string());
    let v3_url = std::env::var("ACC_RPC_URL_V3")
        .unwrap_or_else(|_| "http://localhost:26660/v3".to_string());

    println!("  Trying local DevNet first...");
    println!("  V2 API: {}", v2_url);
    println!("  V3 API: {}", v3_url);

    // Step 2: Connect to DevNet, fallback to Kermit if unavailable
    println!("\nConnecting to network...");
    let (client, network_name) = match create_client(&v2_url, &v3_url).await {
        Ok(c) => {
            // Test if local DevNet is actually responding
            match test_network_status(&c).await {
                Ok(_) => {
                    println!("  [OK] Local DevNet is accessible");
                    (c, "DevNet")
                }
                Err(_) => {
                    println!("  [WARN] Local DevNet not responding, trying Kermit testnet...");
                    let kermit_client = create_client(KERMIT_V2, KERMIT_V3).await?;
                    match test_network_status(&kermit_client).await {
                        Ok(_) => {
                            println!("  [OK] Connected to Kermit testnet");
                            (kermit_client, "Kermit")
                        }
                        Err(e) => {
                            println!("  [ERROR] Both DevNet and Kermit unavailable");
                            return Err(e.into());
                        }
                    }
                }
            }
        }
        Err(_) => {
            println!("  [WARN] Local DevNet unavailable, trying Kermit testnet...");
            let kermit_client = create_client(KERMIT_V2, KERMIT_V3).await?;
            match test_network_status(&kermit_client).await {
                Ok(_) => {
                    println!("  [OK] Connected to Kermit testnet");
                    (kermit_client, "Kermit")
                }
                Err(e) => {
                    println!("  [ERROR] Both DevNet and Kermit unavailable");
                    return Err(e.into());
                }
            }
        }
    };

    println!("  Using network: {}", network_name);

    // Step 3: Generate a test account
    println!("\nGenerating test account...");
    let test_account = generate_test_account().await?;

    // Step 4: Request tokens from faucet (multiple times for sufficient balance)
    println!("\nRequesting tokens from faucet...");
    request_faucet_tokens(&client, &test_account).await?;

    // Step 5: Verify funding and test queries
    println!("\nVerifying funding and testing queries...");
    verify_funding(&client, &test_account).await?;

    println!("\nSuccess: Faucet funding example completed successfully!");
    println!("Hint: Next, run `cargo run --example 210_buy_credits_lite`");

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

async fn test_network_status(client: &AccumulateClient) -> Result<(), Box<dyn std::error::Error>> {
    // Use V3 network-status API which is more reliable
    let result: serde_json::Value = client.v3_client.call_v3("network-status", serde_json::json!({})).await?;

    if result.get("oracle").is_some() {
        println!("  Network status OK (oracle available)");
        Ok(())
    } else {
        Err("Network status missing oracle".into())
    }
}

#[derive(Debug)]
struct TestAccount {
    public_key: [u8; 32],
    lite_identity: String,
    acme_account: String,
}

async fn generate_test_account() -> Result<TestAccount, Box<dyn std::error::Error>> {
    use accumulate_client::crypto::ed25519::Ed25519Signer;

    let signer = Ed25519Signer::generate();
    let public_key = signer.public_key_bytes();
    let lite_identity = derive_lite_identity_url(&public_key);
    let acme_account = format!("{}/ACME", lite_identity);

    println!("  Public key:    {}", hex::encode(&public_key));
    println!("  Lite identity: {}", lite_identity);
    println!("  ACME account:  {}", acme_account);

    Ok(TestAccount {
        public_key,
        lite_identity,
        acme_account,
    })
}

async fn request_faucet_tokens(
    client: &AccumulateClient,
    test_account: &TestAccount,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  Target account: {}", test_account.acme_account);

    // Check initial balance
    print!("  Checking initial balance... ");
    match client.query_account(&test_account.acme_account).await {
        Ok(_) => println!("Account already exists"),
        Err(_) => println!("Account does not exist (expected)"),
    }

    // Request tokens from faucet multiple times
    println!("  Requesting faucet tokens (5 times)...");
    for i in 1..=5 {
        print!("    Faucet request {}/5... ", i);
        match client.faucet(&test_account.acme_account).await {
            Ok(response) => {
                println!("[OK] TxID: {}", response.txid);
            }
            Err(e) => {
                println!("[WARN] {}", e);
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    // Wait for transactions to settle
    println!("  Waiting 10 seconds for transactions to settle...");
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Poll for balance
    println!("  Polling for balance...");
    for attempt in 1..=10 {
        match client.query_account(&test_account.acme_account).await {
            Ok(account) => {
                if let Some(balance) = account.data.get("balance").and_then(|b| b.as_str()) {
                    if balance != "0" {
                        println!("    [OK] Balance confirmed: {}", balance);
                        return Ok(());
                    }
                }
            }
            Err(_) => {}
        }
        if attempt < 10 {
            print!("    Attempt {}/10... ", attempt);
            println!("waiting 3s");
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }

    println!("    [WARN] Balance not confirmed after polling, but faucet requests were sent");
    Ok(())
}

async fn verify_funding(
    client: &AccumulateClient,
    test_account: &TestAccount,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if account was created and funded
    print!("  Checking funded account... ");
    match client.query_account(&test_account.acme_account).await {
        Ok(account) => {
            println!("[OK] Account exists!");
            println!("    URL: {}", account.url);
            println!("    Type: {}", account.account_type);

            if let Some(credits) = account.credits {
                println!("    Credits: {}", credits);
            }

            if let Some(nonce) = account.nonce {
                println!("    Nonce: {}", nonce);
            }

            // Show raw account data
            println!("    Data: {}", serde_json::to_string_pretty(&account.data)?);
        }
        Err(e) => {
            println!("[WARN] Account not found: {}", e);
            println!("    This might indicate:");
            println!("    - Transaction still processing");
            println!("    - Faucet request failed");
            println!("    - DevNet synchronization issues");
        }
    }

    // Test querying the lite identity root
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

// Helper function for lite identity derivation (same as example 100)
fn derive_lite_identity_url(public_key: &[u8; 32]) -> String {
    use accumulate_client::crypto::ed25519::sha256;

    let hash = sha256(public_key);
    let lite_id_bytes = &hash[0..20];
    let lite_id_hex = hex::encode(lite_id_bytes);

    format!("acc://{}.acme", lite_id_hex)
}
