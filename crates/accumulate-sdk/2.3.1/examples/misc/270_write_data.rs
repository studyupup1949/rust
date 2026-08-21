//! Example 270: Write Data to Data Account
//!
//! This example demonstrates:
//! - Creating data account transactions
//! - Writing structured data to the blockchain
//! - Data account management

use accumulate_client::{AccOptions, Accumulate, AccumulateClient};
use dotenvy::dotenv;
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    println!("Write Data to Data Account");
    println!("==========================");

    let client = Accumulate::devnet(AccOptions::default()).await?;

    // Generate keypair (returns SigningKey in ed25519-dalek v2)
    let keypair = AccumulateClient::generate_keypair();
    let public_key = keypair.verifying_key().to_bytes();
    let public_key_hex = hex::encode(public_key);

    let adi_url = format!("acc://data-demo-{}.acme", &public_key_hex[0..8]);
    let data_account_url = format!("{}/data", adi_url);

    println!("Account Information:");
    println!("   ADI URL: {}", adi_url);
    println!("   Data Account: {}", data_account_url);
    println!();

    // Create sample data
    println!("Preparing data to write...");
    let data_payload = json!({
        "timestamp": chrono::Utc::now().timestamp(),
        "event": "user_login",
        "user_id": "demo_user_123",
        "session_data": {
            "ip_address": "192.168.1.100",
            "user_agent": "Accumulate Rust SDK Example",
            "features_used": ["faucet", "data_write", "token_transfer"]
        },
        "metadata": {
            "version": "1.0",
            "schema": "user_event_v1",
            "source": "accumulate_rust_sdk"
        }
    });

    println!("   Data to write:");
    println!("{}", serde_json::to_string_pretty(&data_payload)?);
    println!();

    // Create write data transaction
    println!("Creating write data transaction...");
    let write_tx_body = json!({
        "type": "writeData",
        "data": data_payload,
        "account": data_account_url
    });

    println!("   [OK] Write data transaction prepared");
    println!("   Target account: {}", data_account_url);
    println!(
        "   Data size: {} bytes",
        serde_json::to_string(&data_payload)?.len()
    );
    println!("   Note: Would submit to network in full implementation");

    Ok(())
}
