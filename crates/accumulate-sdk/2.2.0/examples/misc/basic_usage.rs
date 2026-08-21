use accumulate_client::{AccOptions, Accumulate};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get network from environment or default to DevNet
    let network = env::var("ACCUMULATE_NETWORK").unwrap_or_else(|_| "devnet".to_string());

    println!("Connecting to Accumulate {} network...", network);

    // Create client based on network
    let client = match network.as_str() {
        "devnet" => Accumulate::devnet(AccOptions::default()).await?,
        "testnet" => Accumulate::testnet(AccOptions::default()).await?,
        "mainnet" => Accumulate::mainnet(AccOptions::default()).await?,
        url => Accumulate::custom(url, AccOptions::default()).await?,
    };

    // Get network status
    println!("Getting network status...");
    match client.status().await {
        Ok(status) => {
            println!("✓ Network: {}", status.network);
            println!("✓ Version: {}", status.version);
            println!("✓ Commit: {}", status.commit);
            println!("✓ Node ID: {}", status.node_info.id);
            println!("✓ Listen Address: {}", status.node_info.listen_addr);
        }
        Err(e) => {
            eprintln!("✗ Failed to get status: {}", e);
            return Err(e.into());
        }
    }

    // Get client URLs for reference
    let (v2_url, v3_url) = client.get_urls();
    println!("✓ V2 API: {}", v2_url);
    println!("✓ V3 API: {}", v3_url);

    println!("\nBasic connectivity test completed successfully!");

    Ok(())
}
