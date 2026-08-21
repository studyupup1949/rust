//! Example 220: Create ADI using V3 API
//!
//! This example demonstrates:
//! - Creating an Accumulate Digital Identity (ADI)
//! - Using V3 transaction envelope format
//! - Setting up key books and pages

use accumulate_client::{AccOptions, Accumulate, AccumulateClient};
use dotenvy::dotenv;
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    println!("Create ADI using V3 API");
    println!("=======================");

    let client = Accumulate::devnet(AccOptions::default()).await?;

    // Generate keypair for ADI (returns SigningKey in ed25519-dalek v2)
    let keypair = AccumulateClient::generate_keypair();
    let public_key = keypair.verifying_key().to_bytes();
    let public_key_hex = hex::encode(public_key);

    // Create unique ADI URL
    let adi_url = format!("acc://demo-{}.acme", &public_key_hex[0..8]);

    println!("ADI Information:");
    println!("   Public Key: {}", public_key_hex);
    println!("   ADI URL: {}", adi_url);
    println!();

    // Create ADI transaction body
    println!("Creating ADI transaction...");
    let adi_tx_body = json!({
        "type": "createIdentity",
        "url": adi_url,
        "keyBook": {
            "publicKeyHash": public_key_hex
        },
        "keyPage": {
            "keys": [{
                "publicKeyHash": public_key_hex
            }]
        }
    });

    // Create signed envelope
    println!("Creating signed envelope...");
    let envelope = client.create_envelope(&adi_tx_body, &keypair)?;

    println!("   [OK] Envelope created successfully");
    println!("   Signatures: {}", envelope.signatures.len());
    if !envelope.signatures.is_empty() {
        println!("   Signature bytes length: {}", envelope.signatures[0].signature.len());
    }

    // Submit to V3 API
    println!("Submitting to V3 API...");
    match client.submit(&envelope).await {
        Ok(response) => {
            println!("   [OK] Transaction submitted successfully!");
            println!("   Transaction ID: {:?}", response);
        }
        Err(e) => {
            println!("   [ERROR] Failed to submit transaction: {}", e);
            println!("   This is expected on DevNet if the ADI already exists or keys are invalid");
        }
    }

    Ok(())
}

