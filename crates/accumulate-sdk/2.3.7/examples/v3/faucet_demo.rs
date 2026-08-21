use accumulate_client::{AccOptions, Accumulate, AccumulateClient};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get account URL from command line args or use default
    let account_url = env::args()
        .nth(1)
        .unwrap_or_else(|| "acc://demo-account".to_string());

    println!("Faucet Demo - Requesting tokens for: {}", account_url);

    // Connect to DevNet (faucet only available on DevNet/TestNet)
    let client = Accumulate::devnet(AccOptions::default()).await?;

    println!("Connected to DevNet");

    // Check if the account URL is valid format
    if !AccumulateClient::validate_account_url(&account_url) {
        eprintln!(
            "⚠ Warning: Account URL '{}' may not be in the correct format",
            account_url
        );
        println!("Expected format: acc://account-name or account/path");
    }

    // Request tokens from faucet
    println!("Requesting tokens from faucet...");
    match client.faucet(&account_url).await {
        Ok(response) => {
            println!("✓ Faucet request successful!");
            println!("  Transaction ID: {}", response.txid);
            println!("  Account: {}", response.account);
            println!("  Amount: {}", response.amount);
            if !response.link.is_empty() {
                println!("  Link: {}", response.link);
            }

            // Wait a moment and try to query the account
            println!("\nWaiting 3 seconds before querying account...");
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            match client.query_account(&account_url).await {
                Ok(account) => {
                    println!("✓ Account query successful!");
                    println!("  URL: {}", account.url);
                    println!("  Type: {}", account.account_type);
                    if let Some(credits) = account.credits {
                        println!("  Credits: {}", credits);
                    }
                    if let Some(nonce) = account.nonce {
                        println!("  Nonce: {}", nonce);
                    }
                }
                Err(e) => {
                    println!("⚠ Could not query account (may not exist yet): {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Faucet request failed: {}", e);
            eprintln!("Make sure DevNet is running and the faucet is enabled");
            return Err(e.into());
        }
    }

    println!("\nFaucet demo completed!");

    Ok(())
}
