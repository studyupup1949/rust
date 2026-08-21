use abpilot_cc_sdk::{AbpilotClient, AuthMethod};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Assume we already have a JWT token or API key
    let token = std::env::var("ABPILOT_TOKEN")
        .expect("ABPILOT_TOKEN environment variable not set");
    
    let mut client = AbpilotClient::new();
    client.mp_mut().set_auth(AuthMethod::jwt(token));
    
    // Create a new world
    println!("Creating world...");
    let world = client.mp().create_world("My World").await?;
    println!("World created!");
    println!("  World ID: {}", world.world_id);
    println!("  Name: {}", world.name);
    println!("  Secret: {}", world.secret.as_ref().unwrap());
    
    // List all worlds
    println!("\nListing all worlds...");
    let worlds = client.mp().list_worlds().await?;
    for w in &worlds {
        println!("  - {} ({})", w.name, w.world_id);
    }
    
    // Get world details
    println!("\nGetting world details...");
    let world_details = client.mp().get_world(&world.world_id).await?;
    println!("  World: {} ({})", world_details.name, world_details.world_id);
    
    // Get upload URLs for world files
    println!("\nGetting upload URLs...");
    let files = vec!["world.dat", "config.json"];
    let upload_urls = client.mp()
        .get_world_upload_urls(&world.world_id, &files)
        .await?;
    
    for (file, url) in files.iter().zip(upload_urls.iter()) {
        println!("  Upload {} to: {}", file, url);
    }
    
    // Get download URLs for world files
    println!("\nGetting download URLs...");
    let download_urls = client.mp()
        .get_world_download_urls(&world.world_id, &files)
        .await?;
    
    for (file, url) in files.iter().zip(download_urls.iter()) {
        println!("  Download {} from: {}", file, url);
    }
    
    // Reset world secret
    println!("\nResetting world secret...");
    let updated_world = client.mp().reset_world_secret(&world.world_id).await?;
    println!("New secret: {}", updated_world.secret.as_ref().unwrap());
    
    // Delete the world (cleanup)
    println!("\nDeleting world...");
    client.mp().delete_world(&world.world_id).await?;
    println!("World deleted!");
    
    Ok(())
}
