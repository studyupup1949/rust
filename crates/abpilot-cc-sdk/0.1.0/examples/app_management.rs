use abpilot_cc_sdk::{AbpilotClient, AuthMethod};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Assume we already have a JWT token or API key
    let token = std::env::var("ABPILOT_TOKEN")
        .expect("ABPILOT_TOKEN environment variable not set");
    
    let mut client = AbpilotClient::new();
    client.mp_mut().set_auth(AuthMethod::jwt(token));
    
    // Create a new app
    println!("Creating app...");
    let app = client.mp().create_app("My Game").await?;
    println!("App created!");
    println!("  App ID: {}", app.app_id);
    println!("  Name: {}", app.name);
    println!("  Secret: {}", app.secret.as_ref().unwrap());
    
    // List all apps
    println!("\nListing all apps...");
    let apps = client.mp().list_apps().await?;
    for app in &apps {
        println!("  - {} ({})", app.name, app.app_id);
    }
    
    // Get upload URLs for app files
    println!("\nGetting upload URLs...");
    let files = vec!["icon.png", "config.json"];
    let upload_urls = client.mp()
        .get_app_upload_urls(&app.app_id, &files)
        .await?;
    
    for (file, url) in files.iter().zip(upload_urls.iter()) {
        println!("  Upload {} to: {}", file, url);
    }
    
    // Get download URLs for app files
    println!("\nGetting download URLs...");
    let download_urls = client.mp()
        .get_app_download_urls(&app.app_id, &files)
        .await?;
    
    for (file, url) in files.iter().zip(download_urls.iter()) {
        println!("  Download {} from: {}", file, url);
    }
    
    // Reset app secret
    println!("\nResetting app secret...");
    let updated_app = client.mp().reset_app_secret(&app.app_id).await?;
    println!("New secret: {}", updated_app.secret.as_ref().unwrap());
    
    // Delete the app (cleanup)
    println!("\nDeleting app...");
    client.mp().delete_app(&app.app_id).await?;
    println!("App deleted!");
    
    Ok(())
}
