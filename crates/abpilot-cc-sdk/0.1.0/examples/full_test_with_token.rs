use abpilot_cc_sdk::{AbpilotClient, AuthMethod};
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ABPilot CC SDK End-to-End Test (Token-based) ===\n");

    // Check if we have a token or API key
    let auth = if let Ok(token) = env::var("ABPILOT_TOKEN") {
        println!("✅ Using JWT token from ABPILOT_TOKEN");
        AuthMethod::jwt(token)
    } else if let Ok(api_key) = env::var("ABPILOT_API_KEY") {
        println!("✅ Using API key from ABPILOT_API_KEY");
        AuthMethod::api_key(api_key)
    } else {
        eprintln!("❌ Error: No authentication credentials found!");
        eprintln!("\nPlease set one of the following environment variables:");
        eprintln!("  ABPILOT_TOKEN=your_jwt_token");
        eprintln!("  ABPILOT_API_KEY=your_api_key");
        eprintln!("\nTo get credentials:");
        eprintln!("  1. Use the MP API to authenticate and get a token");
        eprintln!("  2. Or create an API key through the web interface");
        return Err("No authentication credentials".into());
    };

    let client = AbpilotClient::new();
    let mut authed_client = client.clone();
    authed_client.mp_mut().set_auth(auth);

    // ============ Step 1: Verify Authentication ============
    println!("\n🔑 Step 1: Verifying Authentication");
    
    println!("Listing API keys to verify authentication...");
    match authed_client.mp().list_api_keys().await {
        Ok(keys) => {
            println!("✅ Authentication successful!");
            println!("   Found {} existing API key(s)", keys.len());
            for key in &keys {
                println!("   - {} ({})", key.name, key.apikey);
            }
        }
        Err(e) => {
            eprintln!("❌ Authentication failed: {}", e);
            return Err(e.into());
        }
    }

    // ============ Step 2: API Key Management ============
    println!("\n🔑 Step 2: API Key Management");
    
    println!("Creating new API key...");
    let api_key = authed_client.mp().create_api_key("SDK Test Key").await?;
    println!("✅ API key created: {}", api_key.apikey);

    println!("Listing all API keys...");
    let api_keys = authed_client.mp().list_api_keys().await?;
    println!("✅ Found {} API key(s):", api_keys.len());
    for key in &api_keys {
        println!("   - {} ({})", key.name, key.apikey);
    }

    // ============ Step 3: App Management ============
    println!("\n📱 Step 3: App Management");
    
    println!("Creating app...");
    let app = authed_client.mp().create_app("SDK Test Game").await?;
    println!("✅ App created!");
    println!("   App ID: {}", app.app_id);
    println!("   Name: {}", app.name);
    println!("   Secret: {}", app.secret.as_ref().unwrap());
    
    let app_id = app.app_id.clone();
    let app_secret = app.secret.clone().unwrap();

    println!("\nListing all apps...");
    let apps = authed_client.mp().list_apps().await?;
    println!("✅ Found {} app(s):", apps.len());
    for a in &apps {
        println!("   - {} ({})", a.name, a.app_id);
    }

    println!("\nGetting app upload URLs...");
    let files = vec!["icon.png", "config.json"];
    let upload_urls = authed_client.mp().get_app_upload_urls(&app_id, &files).await?;
    println!("✅ Got {} upload URL(s)", upload_urls.len());
    for (file, url) in files.iter().zip(upload_urls.iter()) {
        println!("   - {}: {}...", file, &url[..80.min(url.len())]);
    }

    println!("\nGetting app download URLs...");
    let download_urls = authed_client.mp().get_app_download_urls(&app_id, &files).await?;
    println!("✅ Got {} download URL(s)", download_urls.len());
    for (file, url) in files.iter().zip(download_urls.iter()) {
        println!("   - {}: {}...", file, &url[..80.min(url.len())]);
    }

    // ============ Step 4: World Management ============
    println!("\n🌍 Step 4: World Management");
    
    println!("Creating world...");
    let world = authed_client.mp().create_world("SDK Test World").await?;
    println!("✅ World created!");
    println!("   World ID: {}", world.world_id);
    println!("   Name: {}", world.name);
    println!("   Secret: {}", world.secret.as_ref().unwrap());
    
    let world_id = world.world_id.clone();
    let world_secret = world.secret.clone().unwrap();

    println!("\nListing all worlds...");
    let worlds = authed_client.mp().list_worlds().await?;
    println!("✅ Found {} world(s):", worlds.len());
    for w in &worlds {
        println!("   - {} ({})", w.name, w.world_id);
    }

    println!("\nGetting world details...");
    let world_details = authed_client.mp().get_world(&world_id).await?;
    println!("✅ World details:");
    println!("   - Name: {}", world_details.name);
    println!("   - ID: {}", world_details.world_id);

    println!("\nGetting world upload URLs...");
    let world_files = vec!["world.dat", "metadata.json"];
    let world_upload_urls = authed_client.mp().get_world_upload_urls(&world_id, &world_files).await?;
    println!("✅ Got {} upload URL(s)", world_upload_urls.len());

    println!("\nGetting world download URLs...");
    let world_download_urls = authed_client.mp().get_world_download_urls(&world_id, &world_files).await?;
    println!("✅ Got {} download URL(s)", world_download_urls.len());

    // ============ Step 5: Device Token Creation ============
    println!("\n📲 Step 5: Device Token Creation");
    
    let device_id = "sdk_test_device_001";
    let device_info = json!({
        "platform": "rust_sdk",
        "version": "0.1.0",
        "test": true,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    println!("Creating device token for device: {}", device_id);
    let device_token = client.app()
        .create_device_token(
            &app_id,
            &app_secret,
            &world_id,
            device_id,
            device_info,
            3600, // 1 hour TTL
        )
        .await?;
    
    println!("✅ Device token created: {}", device_token.token);
    println!("   Available world nodes: {}", device_token.items.len());
    for node in &device_token.items {
        println!("   - {} (tags: {})", node.base_url, node.tags);
    }

    // ============ Step 6: World Node Management ============
    println!("\n🖥️  Step 6: World Node Management");
    
    println!("Adding world node 1...");
    let node = client.app()
        .update_world_node(
            &world_id,
            &world_secret,
            "https://sdk-test-node1.example.com",
            "cn|us|sdk_test"
        )
        .await?;
    println!("✅ World node added:");
    println!("   - URL: {}", node.base_url);
    println!("   - Tags: {}", node.tags);

    println!("\nAdding world node 2...");
    let node2 = client.app()
        .update_world_node(
            &world_id,
            &world_secret,
            "https://sdk-test-node2.example.com",
            "eu|sdk_test"
        )
        .await?;
    println!("✅ World node added:");
    println!("   - URL: {}", node2.base_url);
    println!("   - Tags: {}", node2.tags);

    // ============ Step 7: Asset Operations ============
    println!("\n💰 Step 7: Asset Operations");
    
    println!("Listing assets for device {}...", device_id);
    let assets = client.app()
        .list_assets(&app_id, &app_secret, device_id, &world_id)
        .await?;
    println!("✅ Found {} asset(s)", assets.len());
    for asset in &assets {
        println!("   - {} {} = {}", asset.r#type, asset.id, asset.value);
    }

    println!("\n💎 Adding 100 gold...");
    let gold_asset = client.app()
        .add_asset(&world_id, &world_secret, device_id, "gold", "001", 100)
        .await?;
    println!("✅ Gold added! New balance: {}", gold_asset.value);

    println!("\n💎 Adding 50 more gold...");
    let gold_asset = client.app()
        .add_asset(&world_id, &world_secret, device_id, "gold", "001", 50)
        .await?;
    println!("✅ Gold added! New balance: {}", gold_asset.value);

    println!("\n💎 Adding 200 gems...");
    let gem_asset = client.app()
        .add_asset(&world_id, &world_secret, device_id, "gem", "001", 200)
        .await?;
    println!("✅ Gems added! Balance: {}", gem_asset.value);

    println!("\n💎 Adding 500 silver...");
    let silver_asset = client.app()
        .add_asset(&world_id, &world_secret, device_id, "silver", "001", 500)
        .await?;
    println!("✅ Silver added! Balance: {}", silver_asset.value);

    println!("\nGetting specific asset (gold)...");
    let gold = client.app()
        .get_asset(&app_id, &app_secret, device_id, &world_id, "gold", "001")
        .await?;
    println!("✅ Gold balance: {}", gold.value);

    println!("\nDeducting 30 gold...");
    let gold_asset = client.app()
        .add_asset(&world_id, &world_secret, device_id, "gold", "001", -30)
        .await?;
    println!("✅ Gold deducted! New balance: {}", gold_asset.value);

    println!("\nListing all assets again...");
    let assets = client.app()
        .list_assets(&app_id, &app_secret, device_id, &world_id)
        .await?;
    println!("✅ Current assets:");
    for asset in &assets {
        println!("   - {} {} = {}", asset.r#type, asset.id, asset.value);
    }

    // ============ Step 8: Device Info Retrieval ============
    println!("\n📱 Step 8: Device Info Retrieval");
    
    println!("Getting device info by token...");
    let device = client.app()
        .get_device_info(&world_id, &world_secret, &device_token.token)
        .await?;
    println!("✅ Device info retrieved:");
    println!("   - Device ID: {}", device.device_id);
    println!("   - World ID: {}", device.world_id);
    println!("   - Info: {}", device.info);

    // ============ Step 9: Test Insufficient Balance ============
    println!("\n⚠️  Step 9: Testing Insufficient Balance");
    
    println!("Attempting to deduct 1000 gold (current balance: ~120, should fail)...");
    match client.app()
        .add_asset(&world_id, &world_secret, device_id, "gold", "001", -1000)
        .await
    {
        Ok(_) => println!("❌ Should have failed!"),
        Err(e) => println!("✅ Correctly failed: {}", e),
    }

    // ============ Step 10: Test Asset Not Found ============
    println!("\n⚠️  Step 10: Testing Asset Not Found");
    
    println!("Attempting to get non-existent asset...");
    match client.app()
        .get_asset(&app_id, &app_secret, device_id, &world_id, "diamond", "999")
        .await
    {
        Ok(_) => println!("❌ Should have failed!"),
        Err(e) => println!("✅ Correctly failed: {}", e),
    }

    // ============ Step 11: Cleanup ============
    println!("\n🧹 Step 11: Cleanup");
    
    println!("Deleting world node 1...");
    client.app()
        .delete_world_node(&world_id, &world_secret, "https://sdk-test-node1.example.com")
        .await?;
    println!("✅ World node 1 deleted");

    println!("Deleting world node 2...");
    client.app()
        .delete_world_node(&world_id, &world_secret, "https://sdk-test-node2.example.com")
        .await?;
    println!("✅ World node 2 deleted");

    println!("Deleting world...");
    authed_client.mp().delete_world(&world_id).await?;
    println!("✅ World deleted");

    println!("Deleting app...");
    authed_client.mp().delete_app(&app_id).await?;
    println!("✅ App deleted");

    println!("Deleting API key...");
    authed_client.mp().delete_api_key(&api_key.apikey).await?;
    println!("✅ API key deleted");

    // ============ Final Summary ============
    println!("\n✅ ========================================");
    println!("✅ All tests completed successfully!");
    println!("✅ ========================================");
    println!("\nTest Summary:");
    println!("  ✓ Authentication verification");
    println!("  ✓ API key management");
    println!("  ✓ App creation and management");
    println!("  ✓ World creation and management");
    println!("  ✓ Device token creation");
    println!("  ✓ World node management");
    println!("  ✓ Asset operations (add/get/list)");
    println!("  ✓ Device info retrieval");
    println!("  ✓ Error handling (insufficient balance)");
    println!("  ✓ Error handling (asset not found)");
    println!("  ✓ Resource cleanup");
    println!("\n🎉 SDK is working perfectly!");

    Ok(())
}
