use accumulate_client::{AccOptions, Accumulate, AccumulateClient};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("Transaction Demo - Creating and signing transactions");

    // Connect to DevNet
    let client = Accumulate::devnet(AccOptions::default()).await?;
    println!("Connected to DevNet");

    // Generate a new keypair for demonstration (returns SigningKey in ed25519-dalek v2)
    let keypair = AccumulateClient::generate_keypair();
    let public_key_hex = hex::encode(keypair.verifying_key().to_bytes());
    println!("Generated keypair with public key: {}", public_key_hex);

    // Create a simple token transfer transaction body
    let from_account = env::args()
        .nth(1)
        .unwrap_or_else(|| "acc://alice".to_string());
    let to_account = env::args()
        .nth(2)
        .unwrap_or_else(|| "acc://bob".to_string());
    let amount = env::args()
        .nth(3)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(100);

    println!("Creating token transfer:");
    println!("  From: {}", from_account);
    println!("  To: {}", to_account);
    println!("  Amount: {}", amount);

    let tx_body = client.create_token_transfer(&from_account, &to_account, amount, None);
    println!("✓ Transaction body created");

    // Create signed transaction envelope
    let envelope = client.create_envelope(&tx_body, &keypair)?;
    println!("✓ Transaction envelope created and signed");

    println!("\nTransaction details:");
    println!("  Signatures: {}", envelope.signatures.len());
    for (i, sig) in envelope.signatures.iter().enumerate() {
        println!("  Signature {}: {} bytes", i + 1, sig.signature.len());
        println!("    Public Key: {}", hex::encode(&sig.public_key));
        println!("    Timestamp: {}", sig.timestamp);
    }

    // Note: In a real scenario, you would submit the transaction:
    // let result = client.submit(&envelope).await?;
    // println!("Transaction submitted with hash: {}", result.hash);

    println!("\n⚠ Note: Transaction was created and signed but not submitted to the network");
    println!(
        "To submit, uncomment the submission code and ensure you have proper account permissions"
    );

    // Demonstrate account creation transaction
    println!("\nDemonstrating account creation transaction:");
    let new_account_url = "acc://demo-new-account";
    let account_tx = client.create_account(new_account_url, &keypair.verifying_key().to_bytes(), "identity");
    println!(
        "✓ Account creation transaction body created for: {}",
        new_account_url
    );

    let account_envelope = client.create_envelope(&account_tx, &keypair)?;
    println!("✓ Account creation envelope signed");

    println!("\nTransaction demo completed!");
    println!("Generated transactions are ready for submission to the network");

    Ok(())
}
