//! Example 11: QuickStart Demo
//!
//! Demonstrates the ultra-simple QuickStart API
//!
//! BEFORE (hundreds of lines):
//!   - Create keypairs
//!   - Derive lite identity/token URLs
//!   - Call faucet multiple times
//!   - Wait for processing
//!   - Query oracle price
//!   - Calculate credit amounts
//!   - Build transaction body
//!   - Build context
//!   - Sign with correct version
//!   - Submit and extract txId
//!   - ... repeat for each operation
//!
//! AFTER (just a few lines per operation):
//!   let acc = QuickStart::kermit().await?;
//!   let wallet = acc.create_wallet();
//!   acc.fund_wallet(&wallet, 5).await?;
//!   let adi = acc.setup_adi(&wallet, "my-adi").await?;
//!
//! Run with: cargo run --example example_11_quickstart_demo

use accumulate_client::{AccumulateClient, QuickStart};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(60));
    println!("  QuickStart Demo - Ultra-Simple Accumulate SDK Usage");
    println!("{}\n", "=".repeat(60));

    // ============================================================
    // STEP 1: Connect to Kermit Testnet (one line!)
    // ============================================================
    println!(">>> Step 1: Connect to Kermit Testnet");
    let acc = QuickStart::kermit().await?;
    println!("    Connected to Kermit Testnet\n");

    // ============================================================
    // STEP 2: Create a wallet (one line!)
    // ============================================================
    println!(">>> Step 2: Create Wallet");
    let wallet = acc.create_wallet();
    println!("    Lite Identity:      {}", wallet.lite_identity);
    println!("    Lite Token Account: {}\n", wallet.lite_token_account);

    // ============================================================
    // STEP 3: Fund wallet from faucet (one line!)
    // ============================================================
    println!(">>> Step 3: Fund Wallet (faucet x5, wait 15s)");
    acc.fund_wallet(&wallet, 5).await?;
    let balance = acc.get_balance(&wallet).await;
    println!("    Balance: {:?} ACME tokens\n", balance);

    // ============================================================
    // STEP 4: Create ADI with one call (automatically handles
    //         credits, key hashing, transaction building, signing)
    // ============================================================
    println!(">>> Step 4: Create ADI (one call does everything!)");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis();
    let adi_name = format!("quickstart-{}", timestamp);
    let adi = acc.setup_adi(&wallet, &adi_name).await?;
    println!("    ADI URL:      {}", adi.url);
    println!("    Key Book:     {}", adi.key_book_url);
    println!("    Key Page:     {}\n", adi.key_page_url);

    // ============================================================
    // STEP 5: Buy credits for ADI (auto-fetches oracle!)
    // ============================================================
    println!(">>> Step 5: Buy Credits for ADI (auto-oracle)");
    let result = acc.buy_credits_for_adi(&wallet, &adi, 500).await?;
    if result.success {
        println!("    Credits purchased successfully!");
    } else {
        println!("    Failed: {:?}", result.error);
    }
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    // Query key page to see credits
    let key_page_info = acc.get_key_page_info(&adi.key_page_url).await;
    if let Some(info) = &key_page_info {
        println!("    Credits: {}", info.credits);
        println!("    Version: {}", info.version);
        println!("    Threshold: {}\n", info.threshold);
    }

    // ============================================================
    // STEP 6: Create token account (one line!)
    // ============================================================
    println!(">>> Step 6: Create Token Account");
    let result = acc.create_token_account(&adi, "tokens").await?;
    if result.success {
        println!("    Created: {}/tokens", adi.url);
    } else {
        println!("    Failed: {:?}", result.error);
    }
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    println!();

    // ============================================================
    // STEP 7: Create data account and write data (two lines!)
    // ============================================================
    println!(">>> Step 7: Create Data Account & Write Data");
    let result = acc.create_data_account(&adi, "mydata").await?;
    if !result.success {
        println!("    Failed to create data account: {:?}", result.error);
    }
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    let result = acc.write_data(&adi, "mydata", &[
        "Hello from QuickStart!",
        "This is so easy!",
    ]).await?;
    if result.success {
        println!("    Created: {}/mydata", adi.url);
        println!("    Wrote 2 data entries\n");
    } else {
        println!("    Failed: {:?}\n", result.error);
    }

    // ============================================================
    // STEP 8: Add key and set multi-sig threshold (two lines!)
    // ============================================================
    println!(">>> Step 8: Multi-Sig Setup");
    let second_key = AccumulateClient::generate_keypair();
    let result = acc.add_key_to_adi(&adi, &second_key).await?;
    if result.success {
        println!("    Added second key to key page");
    } else {
        println!("    Failed to add key: {:?}", result.error);
    }
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    let result = acc.set_multi_sig_threshold(&adi, 2).await?;
    if result.success {
        println!("    Set threshold to 2 (multi-sig)");
    } else {
        println!("    Failed to set threshold: {:?}", result.error);
    }
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    let updated_info = acc.get_key_page_info(&adi.key_page_url).await;
    if let Some(info) = &updated_info {
        println!("    Keys: {}", info.key_count);
        println!("    Threshold: {} (multi-sig!)\n", info.threshold);
    }

    // ============================================================
    // DONE!
    // ============================================================
    println!("{}", "=".repeat(60));
    println!("  DEMO COMPLETE!");
    println!("{}\n", "=".repeat(60));
    println!("Summary:");
    println!("  - Created wallet with lite accounts");
    println!("  - Funded from faucet");
    println!("  - Created ADI: {}", adi.url);
    println!("  - Bought credits (auto-oracle)");
    println!("  - Created token account: {}/tokens", adi.url);
    println!("  - Created data account: {}/mydata", adi.url);
    println!("  - Set up 2-of-2 multi-sig\n");
    println!("All done with minimal code using QuickStart!");

    acc.close();

    Ok(())
}
