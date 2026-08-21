//! # Keygen Lite URLs Example
//!
//! This example demonstrates:
//! - Loading DevNet configuration from .env.local
//! - Generating ED25519 keypairs
//! - Deriving Accumulate account URLs
//! - Testing connectivity to V2/V3 APIs
//! - Basic URL validation
//!
//! ## Prerequisites
//! 1. Run: `cargo run --bin devnet_discovery` to generate .env.local
//! 2. Ensure DevNet is running: `cd devnet && docker compose up -d`
//!
//! ## Usage
//! ```bash
//! cargo run --example 100_keygen_lite_urls
//! ```

use accumulate_client::{AccOptions, AccumulateClient};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Accumulate Keygen & URL Example");
    println!("================================");

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

    // Step 2: Test connectivity
    println!("\nTesting DevNet connectivity...");
    test_connectivity(&v2_url, &v3_url).await?;

    // Step 3: Generate keypairs
    println!("\nGenerating ED25519 keypairs...");
    generate_keypairs().await?;

    // Step 4: Demonstrate URL patterns
    println!("\nAccumulate URL patterns...");
    demonstrate_url_patterns();

    // Step 5: Create client and test basic operations
    println!("\nTesting Accumulate client creation...");
    test_client_creation(&v2_url, &v3_url).await?;

    println!("\nSuccess: Keygen & URL example completed successfully!");
    println!("Hint: Next, run `cargo run --example 120_faucet_local_devnet`");

    Ok(())
}

async fn test_connectivity(v2_url: &str, v3_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    // Test V2 API
    print!("  Testing V2 API... ");
    match client.get(&format!("{}/status", v2_url)).send().await {
        Ok(response) => {
            if response.status().is_success() {
                println!("[OK] Connected ({})", response.status());
            } else {
                println!("[WARN] Got status {} but DevNet may still be starting", response.status());
            }
        }
        Err(e) => {
            println!("[ERROR] Failed to connect: {}", e);
            println!("     Hint: Make sure DevNet is running:");
            println!("        cd devnet && docker compose up -d");
        }
    }

    // Test V3 API
    print!("  Testing V3 API... ");
    match client.get(&format!("{}/status", v3_url)).send().await {
        Ok(response) => {
            if response.status().is_success() {
                println!("[OK] Connected ({})", response.status());
            } else {
                println!("[WARN] Got status {} but DevNet may still be starting", response.status());
            }
        }
        Err(e) => {
            println!("[ERROR] Failed to connect: {}", e);
        }
    }

    Ok(())
}

async fn generate_keypairs() -> Result<(), Box<dyn std::error::Error>> {
    use accumulate_client::crypto::ed25519::{Ed25519Signer, sha256};

    // Generate a random keypair
    println!("  Generating random keypair...");
    let random_signer = Ed25519Signer::generate();

    let pub_key = random_signer.public_key_bytes();
    let priv_key = random_signer.private_key_bytes();

    println!("    Public key:  {}", hex::encode(&pub_key));
    println!("    Private key: {}", hex::encode(&priv_key));
    println!("    Key length:  {} bytes public, {} bytes private", pub_key.len(), priv_key.len());

    // Generate deterministic keypair from seed
    println!("\n  Generating deterministic keypair from seed...");
    let seed = [42u8; 32]; // Deterministic seed for testing
    let deterministic_signer = Ed25519Signer::from_seed(&seed)?;

    let det_pub = deterministic_signer.public_key_bytes();
    let det_priv = deterministic_signer.private_key_bytes();

    println!("    Seed:        {}", hex::encode(&seed));
    println!("    Public key:  {}", hex::encode(&det_pub));
    println!("    Private key: {}", hex::encode(&det_priv));

    // Test signing
    println!("\n  Testing message signing...");
    let message = b"Hello, Accumulate DevNet!";
    let signature = deterministic_signer.sign(message);

    println!("    Message:     {:?}", String::from_utf8_lossy(message));
    println!("    Signature:   {}", hex::encode(signature));
    println!("    Sig length:  {} bytes", signature.len());

    // Test message hashing
    println!("\n  Testing message hashing...");
    let message_hash = sha256(message);
    println!("    SHA-256:     {}", hex::encode(&message_hash));
    println!("    Hash length: {} bytes", message_hash.len());

    Ok(())
}

fn demonstrate_url_patterns() {
    println!("  Accumulate URL patterns and conventions:");

    // Identity URLs
    println!("\n    Identity URLs:");
    println!("      acc://alice.acme           → Identity root");
    println!("      acc://alice.acme/book      → Key book");
    println!("      acc://alice.acme/book/1    → Key page");

    // Token Account URLs
    println!("\n    Token Account URLs:");
    println!("      acc://alice.acme/tokens    → ACME token account");
    println!("      acc://alice.acme/USDC      → Custom token account");
    println!("      acc://company.acme/payroll → Corporate account");

    // System URLs
    println!("\n    System URLs:");
    println!("      acc://faucet.acme/ACME     → Faucet account (DevNet)");
    println!("      acc://oracle.acme          → Oracle service");
    println!("      acc://dn.acme              → Directory Network");

    // ADI Hierarchy
    println!("\n    ADI (Identity) Hierarchy:");
    println!("      acc://company.acme         → Root identity");
    println!("      acc://hr.company.acme     → Sub-identity");
    println!("      acc://alice.hr.company.acme → User under dept");

    // URL Validation
    println!("\n  URL Validation:");
    let test_urls = [
        "acc://alice.acme",
        "acc://alice.acme/tokens",
        "acc://faucet.acme/ACME",
        "invalid-url",
        "",
        "acc://valid.test/path",
    ];

    for url in &test_urls {
        let is_valid = url.starts_with("acc://") && url.contains('.');
        let status = if is_valid { "[OK]" } else { "[ERROR]" };
        println!("    {} {}", status, url);
    }
}

async fn test_client_creation(v2_url: &str, v3_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use url::Url;

    let v2_parsed = Url::parse(v2_url)?;
    let v3_parsed = Url::parse(v3_url)?;

    let options = AccOptions {
        timeout: Duration::from_secs(10),
        headers: std::collections::HashMap::new(),
    };

    println!("  Creating Accumulate client...");
    match AccumulateClient::from_endpoints(v2_parsed, v3_parsed, options).await {
        Ok(client) => {
            println!("    [OK] Client created successfully");

            let (client_v2, client_v3) = client.get_urls();
            println!("    Client V2 URL: {}", client_v2);
            println!("    Client V3 URL: {}", client_v3);

            // Test URL validation method
            println!("\n  Testing URL validation:");
            let test_urls = [
                "acc://alice.acme",
                "acc://alice.acme/tokens",
                "invalid-url",
            ];

            for url in &test_urls {
                let is_valid = AccumulateClient::validate_account_url(url);
                let status = if is_valid { "[OK]" } else { "[ERROR]" };
                println!("    {} {}", status, url);
            }
        }
        Err(e) => {
            println!("    [ERROR] Failed to create client: {}", e);
            println!("    Hint: Check that DevNet is running and accessible");
        }
    }

    Ok(())
}
