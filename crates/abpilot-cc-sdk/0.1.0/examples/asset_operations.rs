use abpilot_cc_sdk::AbpilotClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get credentials from environment
    let app_id = std::env::var("APP_ID").expect("APP_ID not set");
    let app_secret = std::env::var("APP_SECRET").expect("APP_SECRET not set");
    let world_id = std::env::var("WORLD_ID").expect("WORLD_ID not set");
    let world_secret = std::env::var("WORLD_SECRET").expect("WORLD_SECRET not set");
    
    let client = AbpilotClient::new();
    let device_id = "device_001";
    
    // Create a device token
    println!("Creating device token...");
    let device_info = json!({
        "platform": "ios",
        "version": "1.0",
        "device_model": "iPhone 14"
    });
    
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
    
    println!("Device token created: {}", device_token.token);
    println!("Available world nodes:");
    for node in &device_token.items {
        println!("  - {} (tags: {})", node.base_url, node.tags);
    }
    
    // List all assets for the device
    println!("\nListing assets...");
    let assets = client.app()
        .list_assets(&app_id, &app_secret, device_id, &world_id)
        .await?;
    
    if assets.is_empty() {
        println!("No assets found.");
    } else {
        for asset in &assets {
            println!("  - {} {} = {}", asset.r#type, asset.id, asset.value);
        }
    }
    
    // Add gold to the device
    println!("\nAdding 100 gold...");
    let updated_asset = client.app()
        .add_asset(&world_id, &world_secret, device_id, "gold", "001", 100)
        .await?;
    println!("New gold balance: {}", updated_asset.value);
    
    // Get specific asset
    println!("\nGetting gold asset...");
    let gold_asset = client.app()
        .get_asset(&app_id, &app_secret, device_id, &world_id, "gold", "001")
        .await?;
    println!("Gold: {}", gold_asset.value);
    
    // Deduct gold (negative delta)
    println!("\nDeducting 50 gold...");
    let updated_asset = client.app()
        .add_asset(&world_id, &world_secret, device_id, "gold", "001", -50)
        .await?;
    println!("New gold balance: {}", updated_asset.value);
    
    // Update world node
    println!("\nUpdating world node...");
    let world_node = client.app()
        .update_world_node(
            &world_id,
            &world_secret,
            "https://node1.example.com",
            "cn|us"
        )
        .await?;
    println!("World node updated: {} (tags: {})", world_node.base_url, world_node.tags);
    
    // Get device info by token
    println!("\nGetting device info by token...");
    let device = client.app()
        .get_device_info(&world_id, &world_secret, &device_token.token)
        .await?;
    println!("Device ID: {}", device.device_id);
    println!("Device info: {}", device.info);
    
    Ok(())
}
